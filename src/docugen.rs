//! # Documentation generation
//!
//! You start by running [`collect_help_info`] or [`collect_parser_help_info`] depending if you
//! want to generate documentation for the whole parser or for a fragmet of it. From
//! there you can either rely in bpaf to generate the whole documentation for you or to
//! [`split`](HelpInfo::split) [`HelpInfo`] into smaller bits and compose them with extra text.
//!
//! ```rust
//! # use bpaf::*;
//! # use std::path::PathBuf;
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)]
//! /// List directory contents
//! ///
//! ///
//! /// List information about the FILEs (the current directory by default).
//! /// Prints name only unless `--long` is specified
//! ///
//! ///     Exit status:
//! ///       0: if OK
//! ///       1: if requested FILEs does not exist
//! struct Options {
//!     /// use a long listing format
//!     #[bpaf(short, long)]
//!     long: bool,
//!     /// use specific paths instead of current directory
//!     #[bpaf(positional("FILE"))]
//!     files: Vec<PathBuf>
//! }
//!
//! use bpaf::docugen::{
//!     collect_help_info,
//!     roff::semantic::Semantic,
//!     roff::write_updated,
//!     roff::man::{Manpage, Section},
//! };
//!
//! // generate semantic document
//! let mut doc = Semantic::default();
//! doc += collect_help_info(options(), "ls");
//!
//! // render to markdown and save to file
//! let markdown = doc.render_to_markdown();
//! # let path = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("tests").join("sample.md");
//! # let mut files_updated =
//! write_updated(path, markdown.as_bytes()).unwrap();
//!
//! // render to groff and save to file
//! let man = Manpage::new("ls", Section::General,
//!     &["1 Jan 2023", "rust toolbox", "File lister 2000"]);
//! let groff = doc.render_to_manpage(man);
//! # let path = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("tests").join("sample.1");
//! # files_updated |=
//! write_updated(path, groff.as_bytes()).unwrap();
//! # assert!(!files_updated, "Generated docs are updated, commit the files");
//! ```
//!<details>
//!<summary>Generated markdown</summary>
//!
//!```markdown
#![doc = include_str!("../tests/sample.md")]
//!```
//!
//!</details>
//!
//!<details>
//!<summary>Rendered markdown</summary>
//!
#![doc = include_str!("../tests/sample.md")]
//!
//!</details>
//!
//!<details>
//!<summary>Generated ROFF</summary>
//!
//!```text
#![doc = include_str!("../tests/sample.1")]
//!```
//!
//!</details>

use crate::{
    info::Info,
    item::ShortLong,
    meta_help::{HelpItem, HelpItems},
    meta_usage::UsageMeta,
    *,
};
pub use roff;
pub use roff::semantic::*;

/// Help information collected off a parser
///
/// You can create this from [`OptionParser`]
#[derive(Debug, Clone)]
pub struct HelpInfo {
    meta: Meta,
    info: Option<Info>,
    name: Option<&'static str>,
}

impl SemWrite for HelpInfo {
    fn sem_write(self, to: &mut Semantic) {
        if let Some(t) = self.info.as_ref().and_then(|i| i.descr) {
            match self.name {
                Some(name) => {
                    to.section("Name");
                    to.paragraph([mono(name), text(" - "), text(t)]);
                }
                None => {
                    to.section("Summary");
                    to.paragraph(text(t));
                }
            }
        }

        if let Some(t) = self.info.as_ref().and_then(|i| i.header) {
            to.section("Description");
            to.paragraph(text(t));
        }

        *to += self.split();

        if let Some(t) = self.info.as_ref().and_then(|i| i.footer) {
            to.paragraph(text(t));
        }
    }
}

#[derive(Debug, Clone)]
/// A set of help items
///
/// Designed mostly to be a named type with [`SemWrite`] implementation
pub struct Items<'a>(Vec<HelpItem<'a>>);

impl<'a> Items<'a> {
    /// Returns `true` if there's items inside.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns number of elements inside.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// [`HelpInfo`] [`split`](HelpInfo::split) into flags, positional items and commands
#[derive(Debug, Clone)]
pub struct UsageItems<'a> {
    /// Collection of all the flags (`--flag`)
    pub flags: Items<'a>,
    /// Collection of all the positional items (`<FILE>`)
    pub positionals: Items<'a>,
    /// Collection of all the commands (`build`)
    pub commands: Items<'a>,
}

impl SemWrite for Items<'_> {
    fn sem_write(self, to: &mut Semantic) {
        to.definition_list(self.0);
    }
}

pub fn collect_parser_help_info<P, T>(parser: &P) -> HelpInfo
where
    P: Parser<T>,
{
    HelpInfo {
        meta: parser.meta(),
        info: None,
        name: None,
    }
}

pub fn collect_help_info<T>(parser: OptionParser<T>, name: &'static str) -> HelpInfo {
    HelpInfo {
        meta: parser.inner.meta(),
        info: Some(parser.info),
        name: Some(name),
    }
}

pub fn usage<P, T>(parser: &P) -> impl SemWrite + '_
where
    P: Parser<T>,
{
    collect_parser_help_info(parser)
    //    write_with(|doc| {
    //    })
}

/// Extract and write comma separated flag or command names
///
/// Use this if you want to refer to some other parser in parts of your documentation
pub fn names_only<P, T>(parser: &P) -> impl SemWrite + '_
where
    P: Parser<T>,
{
    write_with(|doc| {
        let info = collect_parser_help_info(parser);
        let items = info.split();
        for (ix, item) in items
            .flags
            .0
            .iter()
            .chain(items.positionals.0.iter())
            .chain(items.commands.0.iter())
            .enumerate()
        {
            if ix > 0 {
                *doc += text(", ");
            }
            match item {
                HelpItem::Decor { help: _ } | HelpItem::BlankDecor => {}
                HelpItem::Positional {
                    strict: _,
                    metavar,
                    help: _,
                } => *doc += *metavar,
                HelpItem::Command {
                    name,
                    short: _,
                    help: _,
                    meta: _,
                    info: _,
                } => {
                    *doc += literal(*name);
                }
                HelpItem::Flag { name, help: _ } => {
                    *doc += name.0;
                }
                HelpItem::Argument {
                    name,
                    metavar,
                    env: _,
                    help: _,
                } => {
                    *doc += name.0;
                    *doc += mono(" ");
                    *doc += *metavar;
                }
            }
        }
    })
}

impl HelpInfo {
    /// Split [`HelpInfo`] into help info for flags, positionals and commands
    ///
    /// You can use this method if you want to customize rendering
    pub fn split(&self) -> UsageItems {
        let mut hi = HelpItems::default();
        hi.classify(&self.meta);

        UsageItems {
            flags: docugen::Items(hi.flgs),
            positionals: docugen::Items(hi.psns),
            commands: docugen::Items(hi.cmds),
        }
    }
}

impl SemWrite for UsageItems<'_> {
    fn sem_write(self, to: &mut Semantic) {
        if !self.positionals.is_empty() {
            to.subsection("Available positional items");
            *to += self.positionals;
        }

        if !self.flags.is_empty() {
            to.subsection("Available options");
            *to += self.flags;
        }

        if !self.commands.is_empty() {
            to.subsection("Available commands");
            *to += self.commands;
        }
    }
}

impl SemWrite for &UsageMeta {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            UsageMeta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        *to += mono(" ");
                    }
                    x.sem_write(to);
                }
            }
            UsageMeta::Or(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        *to += mono(" | ");
                    }
                    x.sem_write(to);
                }
            }
            UsageMeta::Required(req) => {
                *to += mono("(");
                req.sem_write(to);
                *to += mono(")");
            }
            UsageMeta::Optional(opt) => {
                *to += mono("[");
                opt.sem_write(to);
                *to += mono("]");
            }
            UsageMeta::Many(_) => todo!(),
            UsageMeta::ShortFlag(f) => {
                *to += [literal('-'), literal(*f)];
            }
            UsageMeta::ShortArg(f, m) => {
                *to += [literal('-'), literal(*f), mono('=')];
                *to += metavar(*m);
            }
            UsageMeta::LongFlag(f) => {
                *to += [literal("--"), literal(*f)];
            }
            UsageMeta::LongArg(f, m) => {
                *to += [literal("--"), literal(*f), mono("="), metavar(m)];
            }
            UsageMeta::Pos(m) => {
                *to += metavar(*m);
            }
            UsageMeta::StrictPos(m) => {
                *to += [mono("-- "), metavar(*m)];
            }

            UsageMeta::Command => {
                *to += [literal("COMMAND"), mono(" "), metavar("...")];
            }
        }
    }
}

impl SemWrite for ShortLong {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            ShortLong::Short(s) => *to += [literal('-'), literal(s)],
            ShortLong::Long(l) => *to += [literal("--"), literal(l)],
            ShortLong::ShortLong(s, l) => {
                *to += [literal('-'), literal(s)];
                *to += [text(", "), literal("--"), literal(l)];
            }
        }
    }
}

impl SemWrite for meta_help::Metavar {
    fn sem_write(self, to: &mut Semantic) {
        *to += metavar(self.0);
    }
}

impl SemWrite for HelpItem<'_> {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            HelpItem::Decor { help } => {
                to.item(text(help));
            }
            HelpItem::BlankDecor => {}
            HelpItem::Positional {
                strict: _,
                metavar,
                help,
            } => {
                to.term(metavar);
                if let Some(help) = help {
                    to.item(text(help));
                }
            }
            HelpItem::Command {
                name,
                short,
                help,
                meta: _,
                info: _,
            } => {
                match short {
                    Some(short) => to.term(write_with(|to| {
                        to.text(literal(short)).text([text(", "), literal(name)]);
                    })),
                    None => to.term(literal(name)),
                };
                if let Some(help) = help {
                    to.item(text(help));
                }
            }

            HelpItem::Flag { name, help } => {
                to.term(name.0);
                if let Some(help) = help {
                    to.item(text(help));
                }
            }
            HelpItem::Argument {
                name,
                metavar,
                env: _,
                help,
            } => {
                to.term(write_with(|to| {
                    to.text(name.0).text(mono("=")).text(metavar);
                }));

                if let Some(help) = help {
                    to.item(text(help));
                }
            }
        }
    }
}
