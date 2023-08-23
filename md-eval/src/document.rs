use std::ops::DerefMut;

use crate::*;
use comrak::nodes::{NodeHtmlBlock, NodeValue};

const STYLE: &str = "padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;";

struct Nav<'a> {
    pad: &'a str,
    prev: Option<&'a str>,
    index: Option<&'a str>,
    next: Option<&'a str>,
}

impl std::fmt::Display for Nav<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Nav {
            pad,
            prev,
            index,
            next,
        } = self;
        if prev.is_none() && next.is_none() && index.is_none() {
            return Ok(());
        }
        writeln!(f, "{pad}")?;
        writeln!(
            f,
            "{pad}<table width='100%' cellspacing='0' style='border: hidden;'><tr>"
        )?;

        writeln!(f, "{pad}  <td style='width: 34%; text-align: left;'>")?;
        if let Some(module) = index {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&larr;&larr;]({module})")?;
            writeln!(f, "{pad}")?;
        }

        writeln!(f, "{pad}  </td>")?;

        writeln!(f, "{pad}  <td style='width: 33%; text-align: center;'>")?;
        if let Some(module) = prev {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&larr; ]({module})")?;
            writeln!(f, "{pad}")?;
        }

        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: right;'>")?;
        if let Some(module) = next {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&rarr;]({module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}</tr></table>")?;
        writeln!(f, "{pad}")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    /// Module name, derived from file name/path
    pub name: String,

    /// Markdown title, extracted from the first line
    pub title: Option<String>,

    /// Document body
    pub body: Option<String>,
    /// Number of executions - used to track insertions from children
    pub execs: usize,

    pub children: Vec<Document>,
    pub prev_sibling: Option<String>,
    pub parent: Option<String>,
    pub next_sibling: Option<String>,
}

pub fn render_module(file: impl AsRef<Path>, results: &[String]) -> anyhow::Result<String> {
    render_entry(file.as_ref(), results)
}

fn render_entry(file: &Path, results: &[String]) -> anyhow::Result<String> {
    let arena = Default::default();
    let entry = entry::import(&arena, file)?;

    let mut execs = 0;

    for (_file_id, code, mut ast) in entry.code_blocks() {
        match code? {
            Block::Code(_id, code) => {
                if let Some(title) = code.title.as_ref() {
                    if let NodeValue::CodeBlock(lit) = ast.deref_mut() {
                        *ast = html(fold_source(title, &lit.literal))
                    }
                }
            }
            Block::Exec(exec) => {
                *ast = html(match exec.title.as_ref() {
                    Some(title) => fold_html(title, &exec.line, &results[execs]),
                    None => format!(
                        "<div style=\"{STYLE}\">\n$ app {}<br />\n{}\n</div>",
                        &exec.line, &results[execs]
                    ),
                });
                execs += 1;
            }
        }
    }

    let mut wrote = Vec::new();
    let options = ComrakOptions::default();
    match &entry {
        entry::Entry::Singleton { name: _, body } => {
            format_commonmark(body, &options, &mut wrote)?;
            Ok(String::from_utf8(wrote)?)
        }
        entry::Entry::Siblings {
            name,
            index,
            siblings,
        } => {
            use std::fmt::Write;
            let mut res = String::new();
            if let Some(index) = index {
                let nav = Nav {
                    pad: "//! ",
                    prev: None,
                    index: None,
                    next: (!siblings.is_empty()).then_some("page_1"),
                };

                format_commonmark(index, &options, &mut wrote)?;
                for line in std::str::from_utf8(&wrote)?.lines() {
                    writeln!(&mut res, "//! {line}")?;
                }
                wrote.clear();

                writeln!(&mut res)?;
                write!(&mut res, "{nav}")?;
            }

            let index_link = format!("super::{name}");

            for (page, child) in siblings.iter().enumerate() {
                writeln!(&mut res)?; // \n/// &nbsp;")?;
                let page = page + 1;
                let prev_page = format!("page_{}", page - 1);
                let next_page = format!("page_{}", page + 1);
                let nav = Nav {
                    pad: "/// ",
                    prev: if page == 1 {
                        index.is_some().then_some(&index_link)
                    } else {
                        Some(&prev_page)
                    },
                    index: index.is_some().then_some(&index_link),
                    next: (page < siblings.len()).then_some(&next_page),
                };

                format_commonmark(child, &options, &mut wrote)?;
                for line in std::str::from_utf8(&wrote)?.lines() {
                    writeln!(&mut res, "/// {line}")?;
                }
                wrote.clear();

                write!(&mut res, "{nav}")?;

                writeln!(&mut res, "pub mod page_{page} {{}}")?;
            }

            Ok(res)
        }
    }
}

fn fold_html(title: &str, line: &str, contents: &str) -> String {
    format!(
        "<details><summary>{title}</summary><div style=\"{STYLE}\">\n$ app {line}<br />\n{contents}\n</div></details>",
    )
}

fn fold_source(title: &str, contents: &str) -> String {
    format!("<details><summary>{title}</summary>\n\n```rust\n{contents}```\n\n</details>")
}

fn html(literal: String) -> NodeValue {
    NodeValue::HtmlBlock(NodeHtmlBlock {
        literal,
        block_type: 0,
    })
}

fn title(root: &entry::Md) -> Option<String> {
    let first = root.first_child()?;
    if let NodeValue::Heading(_) = &first.data.borrow().value {
        let mut wrote = Vec::new();
        format_commonmark(first, &Default::default(), &mut wrote).ok()?;
        let b = String::from_utf8(wrote).ok()?;
        Some(b.split_once(|c| c != '#').unwrap().1.trim().to_owned())
    } else {
        None
    }
}

// rules:
// top level md files in the data folder generate plain markdown for #[doc = include_str!()]
//
// directories serve as entry points to the multi page documents, each directory should contain
// multiple files that are listed in alphabetical order, if index.rs is present - it goes first and
// contain a list of all the pages. It cannot be nested
/*
fn render_module_inner(file: &Path, results: &[String]) -> anyhow::Result<Document> {
    let arena = Arena::new();
    let name = file2mod(file);

    let body;
    let mut title = None;
    let mut execs = 0;
    if file.exists() && file.is_file() {
        let root = read_comrak(&arena, &get_md_path(file)?)?;

        for (block, mut ast) in crate::module::codeblocks(root) {
            match block? {
                Block::Code(_id, code) => {
                    if let Some(title) = code.title.as_ref() {
                        if let NodeValue::CodeBlock(lit) = ast.deref_mut() {
                            *ast = html(fold_source(title, &lit.literal))
                        }
                    }
                }
                Block::Exec(exec) => {
                    *ast = html(match exec.title.as_ref() {
                        Some(title) => fold_html(title, &exec.line, &results[execs]),
                        None => format!(
                            "<div style=\"{STYLE}\">\n$ app {}<br />\n{}\n</div>",
                            &exec.line, &results[execs]
                        ),
                    });
                    execs += 1;
                }
            }
        }

        let mut wrote = Vec::new();
        let options = ComrakOptions::default();
        format_commonmark(root, &options, &mut wrote)?;
        body = Some(String::from_utf8(wrote)?);

        if let Some(x) = root.first_child() {
            if let NodeValue::Heading(_) = &x.data.borrow().value {
                let mut xx = Vec::new();
                format_commonmark(x, &Default::default(), &mut xx)?;
                let b = String::from_utf8(xx)?;
                title = Some(b.split_once(|c| c != '#').unwrap().1.trim().to_owned());
            }
        };
    } else {
        body = None;
    }

    let mut children = Vec::new();
    for child_file in document_children(file)? {
        let mut child = render_module(&child_file, &results[execs..])?;
        execs += child.execs;
        children.push(child);
    }

    let mut document = Document {
        title,
        name,
        body,
        execs,
        ..Document::default()
    };

    for child in children.iter_mut() {
        child.parent = Some(document.name.clone());
    }

    if children.len() > 1 {
        for i in 1..children.len() - 1 {
            children[i - 1].next_sibling = Some(children[i].name.clone());
            children[i].prev_sibling = Some(children[i - 1].name.clone());
        }
    }

    document.execs = execs;
    document.children = children;

    Ok(document)
}*/

struct Navigation<'a> {
    left: Option<&'a str>,
    up: Option<&'a str>,
    right: Option<&'a str>,
}

impl std::fmt::Display for Navigation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.left.is_none() && self.right.is_none() && self.up.is_none() {
            return Ok(());
        }

        let pad = "/// ";

        writeln!(f, "{pad}&nbsp;")?;
        writeln!(f, "{pad}")?;
        writeln!(
            f,
            "{pad}<table width='100%' cellspacing='0' style='border: hidden;'><tr>"
        )?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: left;'>")?;
        if let Some(module) = self.left {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&larr; ]({module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 34%; text-align: center;'>")?;
        if let Some(module) = self.up {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&uarr;](super::{module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: right;'>")?;
        if let Some(module) = self.right {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&rarr;]({module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}</tr></table>")?;
        writeln!(f, "{pad}")?;
        Ok(())
    }
}

impl Document {
    fn navigation(&self) -> Navigation {
        Navigation {
            left: self.prev_sibling.as_deref(),
            up: self.parent.as_deref(),
            right: self.next_sibling.as_deref(),
        }
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.parent.is_some() || !self.children.is_empty() {
            let nav = self.navigation();

            write!(f, "{}", nav)?;

            if let Some(body) = self.body.as_deref() {
                for line in body.lines() {
                    writeln!(f, "/// {line}")?;
                }
            }

            writeln!(f, "///")?;
            for (ix, child) in self.children.iter().enumerate() {
                writeln!(
                    f,
                    "/// {}. [{}]({}::{})",
                    ix + 1,
                    child.title.as_deref().unwrap_or(child.name.as_str()),
                    self.name,
                    child.name
                )?;
            }
            writeln!(f, "///")?;

            write!(f, "{}", nav)?;

            writeln!(f, "pub mod {} {{", self.name)?;

            for child in &self.children {
                writeln!(f, "{}", child)?;
            }

            writeln!(f, "}}")?;
        } else {
            writeln!(f, "{}", self.body.as_deref().unwrap())?;
        }

        Ok(())
    }
}
