//! # Documentation generation
//!
//! You start by running [`collect_help_info`] or [`collect_parser_help_info`] depending if you
//! want to generate documentation for the whole parser or for a fragmet of it. From
//! there you can either rely in bpaf to generate the whole documentation for you or to
//! [`split`](HelpInfo::split) [`HelpInfo`] into smaller bits and compose them with extra text.
//!
//! ```rust
//! # use bpaf::{*, docugen::*};
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
//! use bpaf::docugen::*;
//!
//! // generate semantic document
//! let mut doc = Doc::default();
//! doc.push(collect_help_info(options(), "ls"));
//!
//! // render to markdown and save to file
//! let markdown = doc.render_to_markdown();
//! # let path = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("tests").join("sample.md");
//! # let mut files_updated =
//! write_updated(path, markdown.as_bytes()).unwrap();
//!
//! // render to groff and save to file
//! let groff = doc.render_to_manpage("ls", Section::General, &["1 Jan 2023", "rust toolbox"]);
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
pub use ::roff::*;

/// Help information collected off a parser
///
/// You can create this from [`OptionParser`]
#[derive(Debug, Clone)]
pub struct HelpInfo {
    meta: Meta,
    info: Option<Info>,
    name: Option<&'static str>,
    section_names: SectionName,
}

impl Write for HelpInfo {
    fn write(&self, to: &mut Doc) {
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

        to.push(&self.split());

        if let Some(t) = self.info.as_ref().and_then(|i| i.footer) {
            to.paragraph(text(t));
        }
    }
}

#[derive(Debug, Clone)]
/// A set of help items
///
/// Designed mostly to be a named type with [`Write`] implementation
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

/// Should subsection name be printed when rendering the usage?
///
/// Subsection - "Available Options", etc.
#[derive(Debug, Copy, Clone)]
pub enum SectionName {
    /// Never print section name
    Never,
    /// Print section name only when there's multiple populated sections
    Multiple,
    /// Always print section name
    Always,
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
    /// Should subsection names be printed?
    pub print_sections: bool,
}

impl Write for Items<'_> {
    fn write(&self, to: &mut Doc) {
        to.dlist(self.0.as_slice());
    }
}

pub fn collect_help_info<T>(parser: OptionParser<T>, name: &'static str) -> HelpInfo {
    HelpInfo {
        meta: parser.inner.meta(),
        info: Some(parser.info),
        name: Some(name),
        section_names: SectionName::Multiple,
    }
}

/// Extract and write usage synopsis
///

pub fn synopsis<T>(parser: &OptionParser<T>) -> impl Write + '_ {
    |doc: &mut Doc| {
        if let Some(meta) = parser.inner.meta().to_usage_meta() {
            doc.push(&meta);
        } else {
            doc.text("Parser takes no parameters");
        }
    }
}

pub trait HasMeta {
    fn meta(&self) -> Meta;
}

impl<T> HasMeta for OptionParser<T> {
    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Extract and write down full description for each first level command in this parser
///
/// Does not
pub fn write_commands<M, W>(parser: &M, prefix: Option<W>, doc: &mut Doc)
where
    M: HasMeta,
    W: Write,
{
    let mut hi = HelpItems::default();
    let meta = parser.meta();
    hi.classify(&meta);

    for cmd in hi.cmds {
        if let HelpItem::Command {
            name,
            short,
            help,
            meta,
            info,
        } = cmd
        {
            match short {
                Some(s) => doc.section(&format!("{}, {}", name, s)),
                None => doc.section(name),
            };

            if let Some(help) = help {
                doc.subsection("Description");
                doc.paragraph(help);
            }

            doc.subsection("Synopsis");
            doc.paragraph(|doc: &mut Doc| {
                if let Some(prefix) = &prefix {
                    prefix.write(doc);
                }
                if let Some(usage) = meta.to_usage_meta() {
                    doc.literal(name).mono(" ").push(&usage);
                } else {
                    doc.literal(name);
                }
            });

            doc.subsection("Usage");
            if let Some(descr) = info.descr {
                doc.paragraph(descr);
            }

            if let Some(header) = info.header {
                doc.paragraph(header);
            }

            doc.push(&HelpInfo {
                meta: meta.clone(),
                info: None,
                name: None,
                section_names: SectionName::Multiple,
            });

            if let Some(footer) = info.footer {
                doc.paragraph(footer);
            }
        }
    }

    //    let commands =
}

/*
#[derive(Debug, Clone)]
struct WriteCommands(Meta);

impl Write for WriteCommands {
    fn write(&self, to: &mut Doc) {
        let mut commands = Vec::new();
        match &self.0 {
            Meta::And(_) => todo!(),
            Meta::Or(xs) => todo!(),
            Meta::Optional(x) => todo!(),
            Meta::Item(_) => todo!(),
            Meta::Many(_) => todo!(),
            Meta::Decorated(x, _) => (WriteCommands(*x)).write(to),
            Meta::Skip => {}
            Meta::HideUsage(x) => {
                (WriteCommands(x)).write(to);
            }
        }
    }
}*/

/// Extract and write usage for command line options used by a parser
///
/// You can use this function to insert a list of items parser consumes along with help messages
///
/// ```rust
/// # use bpaf::{docugen::*, *};
/// fn dragon_type() -> impl Parser<bool> {
///     short('d').long("dragon").help("Is the dragon scary?").switch()
/// }
///
/// let mut doc = Doc::default();
/// doc.paragraph(|doc: &mut Doc| {
///     doc.text("You can customize how your dragon looks like with those options:")
///         .push(usage(&dragon_type(), SectionName::Never));
///     });
///
/// let expected = "<p>You can customize how your dragon looks like with those options:
///
/// <dl>
/// <dt><tt><b>-d</b></tt>, <tt><b>--dragon</b></tt></dt>
/// <dd>Is the dragon scary?</dd></dl></p>";
/// assert_eq!(doc.render_to_markdown(), expected);
/// ```
///
/// You can also [`split`](HelpInfo::split) into fields
pub fn usage<P, T>(parser: &P, section_names: SectionName) -> HelpInfo
where
    P: Parser<T>,
{
    HelpInfo {
        meta: parser.meta(),
        info: None,
        name: None,
        section_names,
    }
}

/// Extract and write comma separated flag or command names
///
/// You can use this function to refer to some parser in your documentation. Using
/// [`literal`](roff::literal) and similar methods also work but with this function you can
/// ensure that documentation is always up to date.
/// ```rust
/// # use bpaf::{docugen::*, *};
/// fn dragon_type() -> impl Parser<bool> {
///     short('d').long("dragon").help("Is the dragon scary?").switch()
/// }
///
/// let mut doc = Doc::default();
/// doc.paragraph(|doc: &mut Doc| {
///     doc.text("You can use ")
///         .push(names_only(&dragon_type()))
///         .text(" to unleash the dragon.");
///     });
///
/// let expected = "<p>You can use <tt><b>-d</b></tt>, <tt><b>--dragon</b></tt> to unleash the dragon.</p>";
/// assert_eq!(doc.render_to_markdown(), expected);
/// ```
pub fn names_only<P, T>(parser: &P) -> impl Write + '_
where
    P: Parser<T>,
{
    |doc: &mut Doc| {
        let info = usage(parser, SectionName::Never);
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
                doc.text(", ");
            }
            match item {
                HelpItem::Decor { help: _ } | HelpItem::BlankDecor => {}
                HelpItem::Positional {
                    strict: _,
                    metavar,
                    help: _,
                } => {
                    doc.push(metavar);
                }
                HelpItem::Command {
                    name,
                    short: _,
                    help: _,
                    meta: _,
                    info: _,
                } => {
                    doc.literal(name);
                }
                HelpItem::Flag { name, help: _ } => {
                    doc.push(&name.0);
                }
                HelpItem::Argument {
                    name,
                    metavar,
                    env: _,
                    help: _,
                } => {
                    doc.push(&name.0).mono(" ").push(metavar);
                }
            }
        }
    }
}

impl HelpInfo {
    /// Split [`HelpInfo`] into help info for flags, positionals and commands
    ///
    /// You can use this method if you want to customize rendering
    pub fn split(&self) -> UsageItems {
        let mut hi = HelpItems::default();
        hi.classify(&self.meta);

        let mut filled = 0;
        for s in [&hi.flgs, &hi.psns, &hi.cmds] {
            if !s.is_empty() {
                filled += 1;
            }
        }
        let print_sections = match (self.section_names, filled >= 2) {
            (SectionName::Never, _) => false,
            (SectionName::Multiple, multiple) => multiple,
            (SectionName::Always, _) => true,
        };

        UsageItems {
            flags: docugen::Items(hi.flgs),
            positionals: docugen::Items(hi.psns),
            commands: docugen::Items(hi.cmds),
            print_sections,
        }
    }
}

impl Write for UsageItems<'_> {
    fn write(&self, doc: &mut Doc) {
        if !self.positionals.is_empty() {
            if self.print_sections {
                doc.subsection("Available positional items");
            }
            self.positionals.write(doc);
        }

        if !self.flags.is_empty() {
            if self.print_sections {
                doc.subsection("Available options");
            }
            self.flags.write(doc);
        }

        if !self.commands.is_empty() {
            if self.print_sections {
                doc.subsection("Available commands");
            }
            self.commands.write(doc);
        }
    }
}

impl Write for UsageMeta {
    fn write(&self, doc: &mut Doc) {
        match self {
            UsageMeta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        doc.mono(" ");
                    }
                    x.write(doc);
                }
            }
            UsageMeta::Or(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        doc.mono(" | ");
                    }
                    x.write(doc);
                }
            }
            UsageMeta::Required(req) => {
                doc.mono("(");
                req.write(doc);
                doc.mono(")");
            }
            UsageMeta::Optional(opt) => {
                doc.mono("[");
                opt.write(doc);
                doc.mono("]");
            }
            UsageMeta::Many(_) => todo!(),
            UsageMeta::ShortFlag(f) => {
                doc.push(&[
                    StyledChar(Style::Literal, '-'),
                    StyledChar(Style::Literal, *f),
                ]);
            }
            UsageMeta::ShortArg(f, m) => {
                doc.push(&[
                    StyledChar(Style::Literal, '-'),
                    StyledChar(Style::Literal, *f),
                ]);
                doc.mono("=").metavar(*m);
            }
            UsageMeta::LongFlag(f) => {
                doc.literal("--").literal(*f);
            }
            UsageMeta::LongArg(f, m) => {
                doc.push(&[literal("--"), literal(*f), mono("="), metavar(m)]);
            }
            UsageMeta::Pos(m) => {
                doc.metavar(*m);
            }
            UsageMeta::StrictPos(m) => {
                doc.push(&[mono("-- "), metavar(*m)]);
            }

            UsageMeta::Command => {
                doc.literal("COMMAND").mono(" ").metavar("...");
            }
        }
    }
}

impl Write for ShortLong {
    fn write(&self, doc: &mut Doc) {
        match self {
            ShortLong::Short(s) => {
                doc.push(&[
                    StyledChar(Style::Literal, '-'),
                    StyledChar(Style::Literal, *s),
                ]);
            }
            ShortLong::Long(l) => {
                doc.push(&[literal("--"), literal(l)]);
            }
            ShortLong::ShortLong(s, l) => {
                doc.push(&[
                    StyledChar(Style::Literal, '-'),
                    StyledChar(Style::Literal, *s),
                ]);
                doc.push(&[text(", "), literal("--"), literal(l)]);
            }
        }
    }
}

impl Write for meta_help::Metavar {
    fn write(&self, to: &mut Doc) {
        to.metavar(self.0);
    }
}

impl Write for HelpItem<'_> {
    fn write(&self, to: &mut Doc) {
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
                to.term(*metavar);
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
                    Some(short) => to.term(|to: &mut Doc| {
                        to.push(&StyledChar(Style::Literal, *short))
                            .text(", ")
                            .literal(name);
                    }),

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
                to.term(|to: &mut Doc| {
                    to.push(&name.0).push(&mono("=")).push(metavar);
                });

                if let Some(help) = help {
                    to.item(text(help));
                }
            }
        }
    }
}
