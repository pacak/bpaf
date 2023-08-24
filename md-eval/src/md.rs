use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::fence,
};

use pulldown_cmark::Event;

use crate::{file2mod, Upcoming};

/// Markdown document from data folder
pub enum Document {
    /// Single markdown document, used with `#[doc = ...`, renders to .md
    Page {
        /// markdown file to generate
        name: String,
        contents: String,
        file: PathBuf,
    },
    /// Multi page document, used with `mod`, renders to .rs
    Pages {
        /// rs file to generate
        name: String,
        /// sorted alphabetically
        pages: Vec<String>,
        /// Original file names, used for diagnostics
        files: Vec<PathBuf>,
    },
}

// workflow:
// - inside the build script - read md, render to rust code
// - inside the runner - read md, render to md

impl Document {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let name = file2mod(path);
        if path.is_file() {
            Ok(Self::Page {
                name,
                contents: std::fs::read_to_string(path)?,
                file: path.to_owned(),
            })
        } else {
            let mut files = Vec::new();
            for entry in path.read_dir()? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    files.push(entry.path());
                }
            }
            files.sort();
            Ok(Self::Pages {
                name,
                pages: files
                    .iter()
                    .map(std::fs::read_to_string)
                    .collect::<Result<Vec<_>, _>>()?,
                files,
            })
        }
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = (&Path, Event)> + '_> {
        use pulldown_cmark::Parser;
        match self {
            Document::Page { contents, file, .. } => {
                Box::new(std::iter::repeat(file.as_path()).zip(Parser::new(&contents)))
            }
            Document::Pages { pages, files, .. } => Box::new(
                files
                    .iter()
                    .zip(pages.iter())
                    .flat_map(|(file, s)| std::iter::repeat(file.as_path()).zip(Parser::new(s))),
            ),
        }
    }

    pub fn render_rust(&self) -> anyhow::Result<String> {
        use pulldown_cmark::{CodeBlockKind, Tag};
        use std::fmt::Write;
        let mut out = String::new();
        let mut fence = Upcoming::default();

        let mut modules = String::new();
        let mut typecheck = String::new();
        let mut execs = String::new();
        let mut mapping = BTreeMap::new();
        let mut cur_file = PathBuf::new();
        let mut ix = 0;
        for (file, t) in self.tokens() {
            if file != cur_file {
                mapping.clear();
                cur_file = file.to_owned();
            }

            match t {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f))) => {
                    fence = Upcoming::parse_fence(&f)?;
                    ix += 1;
                    continue;
                }

                Event::Text(code) => match &fence {
                    &Upcoming::Code {
                        title: _,
                        id: Some(id),
                    } => {
                        if mapping.insert(id, ix).is_some() {
                            anyhow::bail!("Duplicate mapping {id}");
                        }
                        writeln!(&mut modules, "mod r{ix} {{ #![allow(dead_code)]")?;
                        unhide(&mut modules, &code)?;
                        writeln!(&mut modules, "}}")?
                    }

                    Upcoming::Code { title: _, id: None } => {}
                    Upcoming::Exec { title: _, ids } => todo!("{fence:?}\n{ids:?}"),
                    Upcoming::Ignore => {}
                },
                _ => {}
            }
            fence = Upcoming::Ignore;
        }
        //        let mut out = String::new();
        //        pulldown_cmark_to_cmark::cmark(self.tokens(), &mut out)?;

        Ok(out)
    }
}

fn unhide(f: &mut String, code: &str) -> std::fmt::Result {
    Ok(())
}
