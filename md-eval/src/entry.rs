use comrak::{
    arena_tree::{Node, NodeEdge},
    nodes::{Ast, NodeValue},
    Arena,
};
use std::{cell::RefCell, path::Path};

use crate::{file2mod, read_comrak, Block};

pub type Md<'a> = Node<'a, RefCell<Ast>>;

#[derive(Debug)]
pub enum Entry<'a> {
    Singleton {
        name: String,
        body: &'a Md<'a>,
    },
    Siblings {
        name: String,
        index: Option<&'a Md<'a>>,
        siblings: Vec<&'a Md<'a>>,
    },
}

pub fn import<'a, 'i>(arena: &'a Arena<Md<'i>>, file: &Path) -> anyhow::Result<Entry<'i>>
where
    'a: 'i,
{
    let name = file2mod(file);
    if file.is_file() {
        Ok(Entry::Singleton {
            body: read_comrak(arena, file)?,
            name,
        })
    } else {
        let mut files = Vec::new();
        let mut index = None;
        for entry in file.read_dir()? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let name = entry.path();
                if name.file_name().unwrap() == "index.md" {
                    index = Some(name);
                } else {
                    files.push(name);
                }
            }
        }

        files.sort();
        let index = if let Some(n) = index {
            Some(read_comrak(arena, &n)?)
        } else {
            None
        };
        let mut siblings = Vec::new();
        for file in &files {
            siblings.push(read_comrak(arena, file)?);
        }

        Ok(Entry::Siblings {
            name,
            index,
            siblings,
        })
    }
}
