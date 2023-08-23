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

fn codeblocks<'a>(
    file_id: usize,
    root: &'a Node<'a, RefCell<Ast>>,
) -> impl Iterator<
    Item = (
        usize,
        anyhow::Result<Block>,
        std::cell::RefMut<'a, NodeValue>,
    ),
> {
    //) -> impl Iterator<Item = (anyhow::Result<Block>, RefMut<&'a mut NodeCodeBlock)> {
    root.traverse().filter_map(move |edge| match edge {
        NodeEdge::Start(node) => {
            let mut ast = node.data.borrow_mut();
            let pos = ast.sourcepos;
            if let NodeValue::CodeBlock(code) = &mut ast.value {
                Some((
                    file_id,
                    Block::parse(pos, code),
                    std::cell::RefMut::map(ast, |a| &mut a.value),
                ))
            } else {
                None
            }
        }
        NodeEdge::End(_) => None,
    })
}

impl<'a> Entry<'a> {
    pub fn code_blocks(
        &'a self,
    ) -> Box<dyn Iterator<Item = (usize, anyhow::Result<Block>, std::cell::RefMut<NodeValue>)> + 'a>
    {
        match self {
            Entry::Singleton { body, .. } => Box::new(codeblocks(0, body)),
            Entry::Siblings {
                index: Some(index),
                siblings,
                ..
            } => Box::new(
                codeblocks(0, index).chain(
                    siblings
                        .iter()
                        .enumerate()
                        .flat_map(|(ix, i)| codeblocks(ix, i)),
                ),
            ),
            Entry::Siblings {
                index: None,
                siblings,
                ..
            } => Box::new(
                siblings
                    .iter()
                    .enumerate()
                    .flat_map(|(ix, i)| codeblocks(ix, i)),
            ),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::Singleton { name, .. } => name.as_str(),
            Entry::Siblings { name, .. } => name.as_str(),
        }
    }

    pub fn ext(&self) -> &str {
        match self {
            Entry::Singleton { .. } => "md",
            Entry::Siblings { .. } => "rs",
        }
    }
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
