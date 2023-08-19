use anyhow::Context;
use comrak::{arena_tree::Node, nodes::Ast, *};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

mod document;
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

fn document_children(file: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if file.is_dir() {
        let mut res = Vec::new();
        for entry in file.read_dir()? {
            let entry = entry?;
            let name = entry.file_name();
            if name == "." || name == ".." || name == "index.md" {
                continue;
            }
            res.push(entry.path());
        }
        res.sort();
        Ok(res)
    } else {
        Ok(Vec::new())
    }
}

fn get_md_path(file: &Path) -> anyhow::Result<Cow<Path>> {
    Ok(if file.is_dir() {
        Cow::from(file.join("index.md"))
    } else {
        Cow::from(file)
    })
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
        .map(|p| import_module(p))
        .collect::<anyhow::Result<Vec<_>>>()?;

    for module in &modules {
        generated += &module.to_string();
        generated += "\n";
    }

    generated += &runner::Runner { modules: &modules }.to_string();

    std::fs::write(out.as_ref().join("lib.rs"), generated)?;
    Ok(())
}
