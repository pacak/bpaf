use std::{fmt::Write, path::Path};

pub mod md;
mod runner;
mod types;

pub use crate::runner::options;
use crate::types::*;
// TODO:
//
// - generate environment and run completion tests

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
        .map(|p| crate::md::Document::load(p)?.render_rust())
        .collect::<anyhow::Result<Vec<_>>>()?;

    for module in &modules {
        generated += &module.code;
        generated += "\n";
    }

    generated += &Runner { modules: &modules }.to_string();

    let mut docs_rs = String::new();
    writeln!(&mut docs_rs, "//! All the custom documentation").unwrap();
    writeln!(
        &mut docs_rs,
        "// this file is generated with help of md-eval, changes will be removed"
    )
    .unwrap();

    for module in &modules {
        if module.is_rs {
            writeln!(&mut docs_rs, "pub mod {};", module.name).unwrap();
        }
    }

    writeln!(
        &mut generated,
        "pub const DOCS_RS: &'static str = {docs_rs:?};"
    )
    .unwrap();

    std::fs::write(out.as_ref().join("lib.rs"), generated)?;
    Ok(())
}

pub(crate) struct Runner<'a> {
    pub(crate) modules: &'a [crate::md::Mod],
}

impl std::fmt::Display for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub fn run_md_eval() {{")?;
        writeln!(f, "  let opts = md_eval::options().run();")?;
        writeln!(
            f,
            "  std::fs::create_dir_all(&opts.out_dir).expect(\"Couldn't create the output dir\");"
        )?;
        for module in self.modules {
            writeln!(f, "  {}::run(&opts.out_dir);", module.name)?;
        }
        writeln!(f, "let mod_file = opts.out_dir.join(\"mod.rs\");")?;
        writeln!(f, "std::fs::write(mod_file, DOCS_RS).unwrap()")?;

        writeln!(f, "}}")?;

        Ok(())
    }
}