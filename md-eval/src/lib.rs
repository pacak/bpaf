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
mod types;
pub use crate::{document::*, module::*, types::*};

// import needs to run twice - once to extract code snippets to run and once to substitute the
// results into final markdown file
//
// alternatively it can create a program that once executed markdown with everything in it...

// workflow: from documentation system run it on a directory of markdown files to generate a
// directory of sources to be included
// as tests

fn read_comrak<'a>(
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    file: &Path,
) -> anyhow::Result<&'a Node<'a, RefCell<Ast>>> {
    let input = std::fs::read_to_string(file)?;
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

fn write_updated(new_val: &str, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    use std::io::Read;
    use std::io::Seek;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;
    let mut current_val = String::new();
    file.read_to_string(&mut current_val)?;
    if current_val != new_val {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, new_val.as_bytes())?;
    }
    Ok(())
}
