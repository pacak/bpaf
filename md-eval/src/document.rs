use crate::*;

const STYLE: &str = "padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;";

pub(crate) struct Nav<'a> {
    pub(crate) pad: &'a str,
    pub(crate) prev: Option<&'a str>,
    pub(crate) index: Option<&'a str>,
    pub(crate) next: Option<&'a str>,
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
            "{pad} <table width='100%' cellspacing='0' style='border: hidden;'><tr>"
        )?;

        writeln!(f, "{pad}  <td style='width: 34%; text-align: left;'>")?;
        if let Some(module) = index {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad} [&larr;&larr;]({module})")?;
            writeln!(f, "{pad}")?;
        }

        writeln!(f, "{pad}  </td>")?;

        writeln!(f, "{pad}  <td style='width: 33%; text-align: center;'>")?;
        if let Some(module) = prev {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad} [&larr; ]({module})")?;
            writeln!(f, "{pad}")?;
        }

        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad}  <td style='width: 33%; text-align: right;'>")?;
        if let Some(module) = next {
            writeln!(f, "{pad}")?;
            writeln!(f, "{pad} [&rarr;]({module})")?;
            writeln!(f, "{pad}")?;
        }
        writeln!(f, "{pad}  </td>")?;
        writeln!(f, "{pad} </tr></table>")?;
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

fn fold_html(title: &str, line: &str, contents: &str) -> String {
    format!(
        "<details><summary>{title}</summary><div style=\"{STYLE}\">\n$ app {line}<br />\n{contents}\n</div></details>",
    )
}

fn fold_source(title: &str, contents: &str) -> String {
    format!("<details><summary>{title}</summary>\n\n```rust\n{contents}```\n\n</details>")
}
