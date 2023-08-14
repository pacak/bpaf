use std::{error::Error, path::PathBuf};

use md_eval::*;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let mut items = Vec::new();

    for entry in std::fs::read_dir("./data")? {
        let entry = entry?;
        if entry.file_name() == "." || entry.file_name() == ".." {
            continue;
        }
        items.push(entry.path());
    }
    items.sort();

    let mut out = String::new();

    for item in items {
        out += &format!("{}\n", import_module(&item)?);
    }
    std::fs::write(path.join("lib.rs"), out)?;

    Ok(())
}
