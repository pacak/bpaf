use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};

use pulldown_cmark::{CowStr, Event};

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
        /// entry point to the file. "load" this to get the document again
        file: PathBuf,
        /// Original file names, used for diagnostics
        files: Vec<PathBuf>,
    },
}

// workflow:
// - inside the build script - read md, render to rust code
// - inside the runner - read md, render to md

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub code: String,
}

impl Document {
    pub fn name(&self) -> &str {
        match self {
            Document::Page { name, .. } | Document::Pages { name, .. } => name.as_str(),
        }
    }

    pub fn ext(&self) -> &str {
        match self {
            Document::Page { .. } => "md",
            Document::Pages { .. } => "rs",
        }
    }

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let name = file2mod(path);
        let file = path.to_owned();
        if path.is_file() {
            Ok(Self::Page {
                name,
                contents: std::fs::read_to_string(path)?,
                file,
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
                file,
            })
        }
    }
    fn read_from(&self) -> &Path {
        match self {
            Document::Page { file, .. } | Document::Pages { file, .. } => file,
        }
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = (&Path, Event)> + '_> {
        use pulldown_cmark::Parser;
        match self {
            Document::Page { contents, file, .. } => {
                Box::new(std::iter::repeat(file.as_path()).zip(Parser::new(contents)))
            }
            Document::Pages { pages, files, .. } => Box::new(
                files
                    .iter()
                    .zip(pages.iter())
                    .flat_map(|(file, s)| std::iter::repeat(file.as_path()).zip(Parser::new(s))),
            ),
        }
    }

    pub fn render_rust(&self) -> anyhow::Result<Mod> {
        use pulldown_cmark::{CodeBlockKind, Tag};
        use std::fmt::Write;

        let mut fence = Upcoming::default();
        let mut modules = String::new();
        let mut execs = String::new();
        let mut mapping = BTreeMap::new();
        let mut cur_file = PathBuf::new();
        let mut ix = 0usize;
        for (file, t) in self.tokens() {
            if file != cur_file {
                mapping.clear();
                cur_file = file.to_owned();
            }

            match t {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f))) => {
                    fence = Upcoming::parse_fence(&f)?;
                    match &fence {
                        Upcoming::Code {
                            title: _,
                            id: Some(id),
                        } => {
                            ix += 1;
                            writeln!(&mut modules, "mod r{ix} {{ #![allow(dead_code)]")?;
                            if mapping.insert(*id, ix).is_some() {
                                anyhow::bail!("Duplicate mapping {id}");
                            }
                        }
                        Upcoming::Code { id: None, .. } => {
                            ix += 1;
                            writeln!(&mut modules, "mod t{ix} {{ #![allow(dead_code)]")?;
                        }
                        Upcoming::Exec { .. } => {}
                        Upcoming::Ignore => {}
                    }
                }

                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_f))) => {
                    match fence {
                        Upcoming::Code { .. } => writeln!(&mut modules, "}}")?,
                        Upcoming::Exec { .. } | Upcoming::Ignore => {}
                    }
                    fence = Upcoming::Ignore;
                }
                Event::Text(code) => match &fence {
                    &Upcoming::Code { .. } => {
                        for line in code
                            .lines()
                            .map(|l| l.trim_start().strip_prefix("# ").unwrap_or(l))
                        {
                            writeln!(&mut modules, "{}", line)?;
                        }
                    }
                    Upcoming::Exec { title: _, ids } => {
                        let args = shell_words::split(&code).unwrap();
                        for id in ids {
                            let Some(code_id) = mapping.get(id) else {
                                panic!("Unknown id {id} for code {code}");
                            };
                            writeln!(
                    &mut execs,
                    "out.push(crate::render_res(r{code_id}::options().run_inner(bpaf::Args::from(&{args:?}).set_name(\"app\"))));"
                )?;
                        }
                    }
                    Upcoming::Ignore => {}
                },
                _ => {}
            }
        }

        let read_from = self.read_from();
        let code = format!(
            "mod {name} {{
        {modules}
        pub fn run(output_dir: &std::path::Path) {{
            #[allow(unused_mut)]
            let mut out = Vec::new();
            {execs}

            let doc = md_eval::md::Document::load({read_from:?}).expect(\"Failed to read \\{read_from:?});
            let md = doc.render_markdown(&out).expect(\"Failed to render \\{read_from:?});

            let dest = std::path::PathBuf::from(output_dir).join(\"{name}.{ext}\");
            std::fs::write(dest, md.to_string()).unwrap();

        }}
    }}",
            name = self.name(),
            ext = self.ext(),
        );
        let name = self.name().to_string();
        Ok(Mod { name, code })
    }

    pub fn render_markdown(&self, out: &[String]) -> anyhow::Result<String> {
        let mut res = String::new();

        match self {
            Document::Page { contents, .. } => {
                let mut x = 0;

                let events = splice_output(contents, out, &mut x);
                pulldown_cmark_to_cmark::cmark(events, &mut res)?;
                Ok(res)
            }
            Document::Pages { pages, name, .. } => {
                use std::fmt::Write;
                let mut x = 0;

                let mut pag = Pagination {
                    pad: "//!",
                    index: name,
                    current: 0,
                    pages: pages.len(),
                };

                let mut buf = String::new();
                pulldown_cmark_to_cmark::cmark(splice_output(&pages[0], out, &mut x), &mut buf)?;

                for line in buf.lines() {
                    let line = line.trim_end();
                    if line.is_empty() {
                        writeln!(&mut res, "//!")?;
                    } else {
                        writeln!(&mut res, "//! {line}")?;
                    }
                }

                write!(&mut res, "{pag}")?;

                writeln!(
                    &mut res,
                    "#[allow(unused_imports)] use crate::{{*, parsers::*}};"
                )?;

                pag.pad = "///";
                for (page, child) in pages[1..].iter().enumerate() {
                    let page = page + 1;
                    buf.clear();

                    let mut buf = String::new();
                    pulldown_cmark_to_cmark::cmark(splice_output(child, out, &mut x), &mut buf)?;

                    writeln!(&mut res, "\n")?;
                    for line in buf.lines() {
                        let line = line.trim_end();
                        if line.is_empty() {
                            writeln!(&mut res, "///")?;
                        } else {
                            writeln!(&mut res, "/// {line}")?;
                        }
                    }

                    pag.current = page;
                    write!(&mut res, "{pag}")?;

                    writeln!(&mut res, "pub mod page_{page} {{}}")?;
                }

                Ok(res)
            }
        }
    }
}

fn splice_output<'a>(
    contents: &'a str,
    outs: &'a [String],
    outs_used: &'a mut usize,
) -> impl Iterator<Item = Event<'a>> {
    Splicer {
        inner: pulldown_cmark::Parser::new(contents),
        outs,
        outs_used,
        queue: VecDeque::new(),
    }
}

struct Splicer<'a, I> {
    inner: I,
    outs: &'a [String],
    outs_used: &'a mut usize,
    queue: VecDeque<Event<'a>>,
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for Splicer<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use pulldown_cmark::{CodeBlockKind, Tag};
        if let Some(q) = self.queue.pop_front() {
            return Some(q);
        }

        match self.inner.next()? {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f))) => {
                let fence = Upcoming::parse_fence(&f).unwrap();
                match &fence {
                    Upcoming::Code { title, .. } => {
                        if let Some(title) = title {
                            self.queue.push_back(fold_open(title.as_str()));
                        }
                        let cb = Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()));
                        self.queue.push_back(Event::Start(cb));
                        // transfer rust code text and closing codeblock tag
                        self.queue.push_back(self.inner.next()?);
                        self.queue.push_back(self.inner.next()?);
                        if title.is_some() {
                            self.queue.push_back(Event::Html("</details>".into()));
                        }

                        self.queue.pop_front()
                    }
                    Upcoming::Exec { title, ids, .. } => {
                        if let Some(title) = title {
                            self.queue.push_back(fold_open(title));
                        }

                        let Event::Text(code) = self.inner.next()? else {
                            panic!("To run an example with no input leave an empty line between opening and closing ```");
                        };

                        if ids.len() > 1 {
                            for i in 1..ids.len() {
                                assert_eq!(
                                    &self.outs[*self.outs_used],
                                    &self.outs[*self.outs_used + i]
                                );
                            }
                        }

                        let snip = &self.outs[*self.outs_used];
                        let html = format!("\n\n```text\n$ app {code}{snip}```\n\n");
                        self.queue.push_back(Event::Html(html.into()));
                        *self.outs_used += ids.len();

                        self.inner.next()?;

                        if title.is_some() {
                            self.queue.push_back(Event::Html("</details>".into()));
                        }
                        self.queue.pop_front()
                    }
                    _ => Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f)))),
                }
            }

            // by default pulldown_cmark_to_cmark mangles [`foo`] to \[`foo`\], but not
            // [`foo`](foo), so this turns former into later...
            Event::Text(CowStr::Borrowed("[")) => {
                let text_or_code = self.inner.next()?;
                let closing = self.inner.next()?;
                if let (Event::Code(b) | Event::Text(b), Event::Text(CowStr::Borrowed("]"))) =
                    (&text_or_code, &closing)
                {
                    let link = Tag::Link(
                        pulldown_cmark::LinkType::Inline,
                        b.clone(),
                        CowStr::Borrowed(""),
                    );
                    self.queue.push_back(Event::Start(link.clone()));
                    self.queue.push_back(text_or_code.clone());
                    self.queue.push_back(Event::End(link));
                } else {
                    self.queue.push_back(Event::Text(CowStr::Borrowed("[")));
                    self.queue.push_back(text_or_code);
                    self.queue.push_back(closing);
                }
                self.queue.pop_front()
            }

            event => Some(event),
        }
    }
}

fn fold_open(title: &str) -> Event<'static> {
    Event::Html(format!("<details><summary>{title}</summary>").into())
}

#[derive(Clone, Copy)]
pub(crate) struct Pagination<'a> {
    pad: &'a str,
    index: &'a str,
    current: usize,
    pages: usize,
}

impl std::fmt::Display for Pagination<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pagination {
            pad,
            index,
            current,
            pages,
        } = *self;
        if pages == 1 {
            return Ok(());
        }
        writeln!(f, "{pad}")?;

        writeln!(
            f,
            "{pad} <table width='100%' cellspacing='0' style='border: hidden;'><tr>"
        )?;

        writeln!(f, "{pad}  <td style='text-align: center;'>")?;
        writeln!(f, "{pad}")?;

        match current {
            0 => {}
            1 => {
                writeln!(f, "{pad} [&larr;](super::{})", index)?;
            }
            _ => {
                writeln!(f, "{pad} [&larr;](page_{})", current - 1)?;
            }
        }

        for page in 0..pages {
            if page == current {
                writeln!(f, "{pad} **{}**", page + 1)?;
            } else if page == 0 {
                writeln!(f, "{pad} [1](super::{})", index)?;
            } else {
                writeln!(f, "{pad} [{}](page_{})", page + 1, page)?;
            }
        }

        if current + 1 < pages {
            writeln!(f, "{pad} [&rarr;](page_{})", current + 1)?;
        }

        writeln!(f, "{pad}")?;
        writeln!(f, "{pad}  </td>")?;

        writeln!(f, "{pad} </tr></table>")?;

        Ok(())
    }
}
