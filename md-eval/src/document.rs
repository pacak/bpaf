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
            Block::Ignore => {}
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
                writeln!(
                    &mut res,
                    "#[allow(unused_imports)] use crate::{{*, parsers::*}};"
                )?;
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
