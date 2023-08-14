use crate::*;

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub name: String,
    pub body: String,
    pub execs: usize,
    pub children: Vec<Document>,
    pub prev_sibling: Option<String>,
    pub parent: Option<String>,
    pub next_sibling: Option<String>,
}

pub fn render_module(file: impl AsRef<Path>, results: &[String]) -> anyhow::Result<Document> {
    render_module_inner(file.as_ref(), results)
}

fn render_module_inner(file: &Path, results: &[String]) -> anyhow::Result<Document> {
    let arena = Arena::new();
    let root = read_comrak(&arena, &get_md_path(file)?)?;
    let name = file2mod(file);

    let mut execs = 0;
    for edge in root.traverse() {
        if let arena_tree::NodeEdge::Start(n) = edge {
            let ast = &mut n.data.borrow_mut();
            if let nodes::NodeValue::CodeBlock(code) = &mut ast.value {
                if let Ok(toks) = CodeTok::parse(code) {
                    if toks.get(0).copied() == Some(CodeTok::Runner) {
                        code.literal = results[execs].clone();
                        execs += 1;
                    }
                }
            }
        }
    }

    let mut wrote = Vec::new();
    let options = ComrakOptions::default();
    format_commonmark(root, &options, &mut wrote)?;
    let body = String::from_utf8(wrote)?;

    let mut document = Document {
        name,
        body,
        ..Document::default()
    };

    let mut children = Vec::new();
    for child_file in document_children(file)? {
        let mut child = render_module(&child_file, &results[execs..])?;
        child.parent = Some(document.name.clone());
        execs += child.execs;
        children.push(child);
    }

    if children.len() > 1 {
        for i in 1..children.len() - 1 {
            children[i - 1].next_sibling = Some(children[i].name.clone());
        }

        for i in 0..children.len() - 2 {
            children[i].next_sibling = Some(children[i + 1].name.clone());
        }
    }

    document.execs = execs;
    document.children = children;

    Ok(document)
}

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

        let pad = "//! ";

        writeln!(f, "{pad}&nbsp;")?;
        writeln!(f, "{pad}")?;
        writeln!(
            f,
            "{pad}<table width='100%' cellspacing='0' style='border: hidden;'><tr>"
        )?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: left;'>")?;
        if let Some(module) = self.left {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&larr; ](super::{module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 34%; text-align: center;'>")?;
        if let Some(module) = self.up {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&uarr;](super::super::{module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: right;'>")?;
        if let Some(module) = self.right {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad}[&rarr;](super::{module})")?;
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

            for line in self.body.lines() {
                writeln!(f, "//! {line}")?;
            }
            write!(f, "{}", nav)?;

            writeln!(f, "mod {} {{", self.name)?;

            for child in &self.children {
                writeln!(f, "{}", child)?;
            }

            writeln!(f, "}}")?;
        } else {
            writeln!(f, "{}", self.body)?;
        }

        Ok(())
    }
}
