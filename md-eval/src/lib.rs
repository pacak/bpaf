use comrak::{arena_tree::Node, nodes::Ast, *};
use std::{cell::RefCell, collections::BTreeMap, path::Path};

mod document;
mod entry;
pub mod md;
mod module;
mod runner;
mod types;

pub use crate::{document::*, module::*, runner::*, types::*};

// TODO:
//
// - run it via binary instead of a test?
// - generate environment and run completion tests

fn read_comrak<'a>(
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    file: &Path,
) -> anyhow::Result<&'a Node<'a, RefCell<Ast>>> {
    let Ok(input) = std::fs::read_to_string(file) else {
        anyhow::bail!("Couldn't read markdown from {file:?}");
    };
    let options = ComrakOptions::default();
    Ok(parse_document(arena, &input, &options))
}

fn file2mod(file: &Path) -> String {
    (if file.is_dir() {
        file.file_name()
    } else {
        file.file_stem()
    })
    .expect("No file name?")
    .to_str()
    .unwrap()
    .to_string()
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) struct Runner<'a> {
    pub(crate) modules: &'a [Mod],
}

impl std::fmt::Display for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub fn run_md_eval() {{")?;
        writeln!(f, "  let opts = md_eval::options().run();")?;

        for module in self.modules {
            writeln!(f, "  {}::run(&opts.out_dir);", module.name)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

/// read markdowns from `source` directory, write a lib file into `out` file
pub fn process_directory(source: impl AsRef<Path>, out: impl AsRef<Path>) -> Result<()> {
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    let mut items = Vec::new();

    // read all the files and process them sorted
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name() == "." || entry.file_name() == ".." {
            continue;
        }
        items.push(entry.path());
    }
    items.sort();

    let mut generated = String::new();

    let modules = items
        .iter()
        .map(|p| construct_module(p))
        .collect::<anyhow::Result<Vec<_>>>()?;

    for module in &modules {
        generated += &module.code;
        generated += "\n";
    }

    generated += &Runner { modules: &modules }.to_string();

    std::fs::write(out.as_ref().join("lib.rs"), generated)?;
    Ok(())
}

pub fn process_directory2(source: impl AsRef<Path>, out: impl AsRef<Path>) -> Result<()> {
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
    let mut items = Vec::new();

    // read all the files and process them sorted
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name() == "." || entry.file_name() == ".." {
            continue;
        }
        items.push(entry.path());
    }
    items.sort();

    let mut generated = String::new();

    let modules = items
        .iter()
        .map(|p| crate::md::Document::load(p)?.render_rust())
        .collect::<anyhow::Result<Vec<_>>>()?;

    for module in &modules {
        generated += &module.code;
        generated += "\n";
    }

    generated += &Runner2 { modules: &modules }.to_string();

    std::fs::write(out.as_ref().join("lib.rs"), generated)?;
    Ok(())
}

pub(crate) struct Runner2<'a> {
    pub(crate) modules: &'a [crate::md::Mod],
}

impl std::fmt::Display for Runner2<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub fn run_md_eval() {{")?;
        writeln!(f, "  let opts = md_eval::options().run();")?;

        for module in self.modules {
            writeln!(f, "  {}::run(&opts.out_dir);", module.name)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
