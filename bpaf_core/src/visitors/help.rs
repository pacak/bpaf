//! Overall rendering takes 2 stages:
//! 1. collect info from the visitor. At this point we don't know what the tabstop is
//!    so can't really render it into the final version. Info is collected into
//!    strings: one per section group (named/pos/commands/global/sections).
//!    Plus descr/header/footer.
//! 2. convert it to final version with tabs expanded into spaces and colors applied
//!
//!
//! Text is separated with tab symbols into 3 virtual columns, 2 tabs.
//! 1st column - description and header text. Can grow up to MAX_WIDTH, obeys newline separation
//! 2nd column - flags with metavars. on rows with them 1st column must be empty (insert an \n if
//! it isn't) and contents are padded with 4 spaces, it's width, as long as it is under MAX_TAB
//! sets the tabstop, does not obey newline separation rules
//! 3rd column - starts after the tabstop

//! Second pass renders text to ANSI. All it needs to do is to
//! 1. expand tabs into spaces
//! 2. split
//! - for ANSI it expands tabs and applies
//! - for roff/

const T: &str = Style::Text.ansi();
const L: &str = Style::Literal.ansi();
const H: &str = Style::Header.ansi();
const M: &str = Style::Metavar.ansi();

use super::ShortLong;
use crate::{
    Flag, Item, Nest, VKind, Visited,
    console_writer::{MAX_TAB, Style, Styled, char_width},
    custom_help::Block,
    miniansi::Frag,
    visitors::{VisitGroup, Visitor, usage::Usage},
};

impl<'a> Help<'a> {
    fn new(path: &'a str, detailed: crate::Help) -> Help<'a> {
        Help {
            path,
            detailed,
            ..Help::default()
        }
    }

    fn render(mut self) -> Styled {
        self.prepare_output();
        Styled {
            raw: self.output,
            tab: self.max_tab + 2,
        }
    }
}

struct GlobalOnly<'h> {
    help: Help<'h>,
    stack: Vec<VisitGroup>,
    global: usize,
}
impl<'h> GlobalOnly<'h> {
    fn new(help: Help<'h>) -> Self {
        Self {
            help,
            stack: Vec::new(),
            global: 0,
        }
    }
}

impl<'a> Visitor<'a> for GlobalOnly<'a> {
    fn item<'t>(&mut self, item: Item<'a, 't>) {
        if self.global > 0 {
            self.help.item(item)
        }
        match item {
            Item::OptionParser { inner, .. } | Item::Command { inner, .. } => inner.vi(self),
            Item::Flag { .. }
            | Item::Arg { .. }
            | Item::Positional { .. }
            | Item::Section { .. }
            | Item::Nested { .. }
            | Item::Rendered { .. } => {}
        }
    }

    fn identify(&self) -> VKind {
        self.help.identify()
    }

    fn push_group(&mut self, group: VisitGroup) {
        if group == VisitGroup::Global {
            self.global += 1;
        }
        self.stack.push(group);
        if self.global > 0 {
            self.help.push_group(group);
        }
    }

    fn pop_group(&mut self) {
        if self.global > 0 {
            self.help.pop_group();
        }
        let group = self.stack.pop().unwrap();
        if group == VisitGroup::Global {
            self.global -= 1;
        }
    }
}

impl crate::RawCtx<'_> {
    pub(crate) fn render_help(&self, detailed: crate::Help) -> crate::error::ParseFailure {
        let h = Help::new(&self.path, detailed);
        let mut g = GlobalOnly::new(h);
        for p in self.shared.parsers.borrow().iter() {
            p.vi(&mut g);
        }
        self.visited.vi(&mut g.help);
        self.shared.help.vi(&mut g.help);
        crate::error::ParseFailure::Stdout(g.help.render())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Lit<'a>(pub(crate) ShortLong<'a>);

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "    ")?;
        }
        match self.0 {
            ShortLong::Short(s) => write!(f, "{L}{s}{T}"),
            ShortLong::Long(l) => write!(f, "{L}{l}{T}"),
            ShortLong::Both(s, l) => write!(f, "{L}{l}{T}, {L}{s}{T}"),
        }
    }
}

impl Lit<'_> {
    fn col_width(&self) -> usize {
        4 + match self.0 {
            ShortLong::Short(_) => 1,
            ShortLong::Long(l) => char_width(l),
            ShortLong::Both(_, l) => char_width(l) + 3,
        }
    }
}

impl std::fmt::Display for ShortLong<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const L: &str = Style::Literal.ansi();
        const T: &str = Style::Text.ansi();
        if f.alternate() {
            match self {
                ShortLong::Short(s) => write!(f, "    {L}-{s}{T}"),
                ShortLong::Long(l) => write!(f, "        {L}--{l}{T}"),
                ShortLong::Both(s, l) => write!(f, "    {L}-{s}{T}, {L}--{l}{T}"),
            }
        } else {
            match self {
                ShortLong::Short(s) => write!(f, "{L}-{s}{T}"),
                ShortLong::Long(l) => write!(f, "{L}--{l}{T}"),
                ShortLong::Both(s, l) => write!(f, "{L}-{s}{T}, {L}--{l}{T}"),
            }
        }
    }
}

// TODO - dedup
fn lit_name<'a>(names: &'a [crate::Lit<'a>]) -> Lit<'a> {
    let mut short = None;
    let mut long = None;
    for n in names {
        match &n.0 {
            crate::Name::Short(s) => short = short.or(Some(s)),
            crate::Name::Long(l) => long = long.or(Some(l)),
        }
    }
    Lit(match (short, long) {
        (None, None) => panic!("must have a single name"),
        (None, Some(l)) => ShortLong::Long(l),
        (Some(s), None) => ShortLong::Short(*s),
        (Some(s), Some(l)) => ShortLong::Both(*s, l),
    })
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Default, Debug, Clone, Copy)]
pub enum Place {
    Named,
    Pos,
    Command,
    Section,
    Global,
    Descr,
    Usage,
    Header,
    #[default]
    Footer,
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default)]
struct Help<'a> {
    path: &'a str,
    place: Place,
    /// Nesting depth for Section/Nested contexts; place is only updated when depth == 0
    depth: usize,
    in_global: usize,

    /// All the nonstandard sections, can be empty
    sections: String,

    /// Default sections
    named: String,
    pos: String,
    commands: String,
    global: String,
    /// Maximum seen tab slice (under the limit)
    max_tab: usize,

    detailed: crate::Help,

    output: String,
    usage: String,
    descr: String,
    header: String,
    footer_buf: String,
}

impl Help<'_> {
    fn buf(&self) -> &str {
        match self.place {
            Place::Named => &self.named,
            Place::Pos => &self.pos,
            Place::Command => &self.commands,
            Place::Section => &self.sections,
            Place::Global => &self.global,
            Place::Descr => &self.descr,
            Place::Usage => &self.usage,
            Place::Header => &self.header,
            Place::Footer => &self.footer_buf,
        }
    }

    fn mut_buf(&mut self) -> &mut String {
        match self.place {
            Place::Named => &mut self.named,
            Place::Pos => &mut self.pos,
            Place::Command => &mut self.commands,
            Place::Section => &mut self.sections,
            Place::Global => &mut self.global,
            Place::Descr => &mut self.descr,
            Place::Usage => &mut self.usage,
            Place::Header => &mut self.header,
            Place::Footer => &mut self.footer_buf,
        }
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item<'t>(&mut self, item: Item<'a, 't>) {
        use std::fmt::Write as _;

        self.update_place(&item);
        match item {
            Item::OptionParser { info, inner } => {
                if let Some(descr) = info.descr {
                    self.place = Place::Descr;
                    self.copy_text(false, descr);
                }
                self.place = Place::Usage;
                if let Some(usage) = info.usage {
                    self.usage.push_str(usage);
                } else {
                    if !self.path.is_empty() {
                        for seg in self.path.split(' ') {
                            _ = write!(self.usage, "{L}{seg}{T} ");
                        }
                    }
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.vi(&mut usage);
                    usage.render_to(&mut self.usage);
                    self.usage.truncate(self.usage.trim_end().len());
                }
                if let Some(header) = info.header {
                    self.place = Place::Header;
                    self.copy_text(false, header);
                }
                if let Some(footer) = info.footer {
                    self.place = Place::Footer;
                    self.copy_text(false, footer);
                }
                self.place = Place::Footer;
                inner.vi(self);
            }
            Item::Flag { named } => self.write_named_item(named, None),
            Item::Arg { named, meta } => self.write_named_item(named, Some(meta)),
            Item::Positional {
                meta,
                help,
                strict: _,
            } => {
                self.write_buf(format_args!("    {M}{meta}{T}"));
                self.track_tab(meta.width());
                self.help(help);
            }
            Item::Command {
                names,
                help,
                inner: _,
            } => {
                let name = lit_name(names);
                self.write_buf(format_args!("{name:#}"));
                self.track_tab(name.col_width());
                self.help(help);
            }
            Item::Nested { outer, inner } => {
                let before = self.mut_buf().len();

                let help = match outer {
                    Nest::Named(Flag { named, .. }) => {
                        let Some(name) = named.get_shortlong() else {
                            // pure env nested parser, makes little sense.
                            return;
                        };
                        self.write_buf(format_args!("{:#} ", name));
                        named.help
                    }
                    Nest::Keyword(keyword) => {
                        let name = lit_name(&keyword.named.names);
                        self.write_buf(format_args!("{:#} ", name));
                        keyword.named.help
                    }
                };

                let mut u = Usage::default();
                inner.vi(&mut u);
                u.render_to(self.mut_buf());

                if help.is_some() {
                    self.track_tab(self.written_chars_since(before));
                    self.help(help);
                } else {
                    self.mut_buf().push('\n');
                }

                self.depth += 1;
                inner.vi(self);
                self.depth -= 1;
            }
            Item::Section {
                title,
                descr,
                inner,
            } => {
                if self.depth == 0 {
                    _ = writeln!(&mut self.sections, "{H}{title}{T}");
                    self.place = Place::Section;
                }
                self.depth += 1;
                inner.vi(self);
                self.depth -= 1;
                assert_eq!(descr, None);
            }
            Item::Rendered { text } => {
                let depth = self.depth;
                for frag in crate::miniansi::split::<Block>(text) {
                    match frag {
                        Frag::Code(Block::Start(Place::Section)) => {
                            if self.depth == 0 {
                                self.place = Place::Section;
                            }
                            self.depth += 1;
                        }
                        Frag::Code(Block::EndSection) => {
                            self.depth -= 1;
                        }
                        Frag::Code(Block::Start(p)) => {
                            if self.depth == 0 {
                                self.place = p;
                            }
                        }
                        Frag::Str(s) => {
                            let buf = self.mut_buf();
                            if !(buf.ends_with('\n') || buf.is_empty()) {
                                buf.push('\n');
                            }
                            for line in s.lines() {
                                if let Some((key, _)) = line.split_once('\t') {
                                    self.track_tab(crate::miniansi::text_len(key));
                                }
                            }
                            self.mut_buf().push_str(s);
                            if !s.ends_with('\n') {
                                self.mut_buf().push('\n');
                            }
                        }
                    }
                }
                self.depth = depth;
            }
        }
    }

    fn push_group(&mut self, group: VisitGroup) {
        if matches!(group, VisitGroup::Global) {
            self.in_global += 1;
        }
    }

    fn pop_group(&mut self) {
        if self.in_global > 0 {
            self.in_global -= 1;
        }
    }

    fn identify(&self) -> VKind {
        VKind::Help
    }
}

impl Help<'_> {
    /// Check if tab width needs to be updated to account for `width`
    ///
    /// Width must be of the name/meta itself with no outer padding added.
    /// It can include inner spaces for nested usage, etc.
    fn track_tab(&mut self, width: usize) {
        if width <= MAX_TAB {
            self.max_tab = self.max_tab.max(width);
        }
    }

    fn write_env(&mut self, env: &str) {
        use std::fmt::Write as _;
        _ = writeln!(self.mut_buf(), "\tUses environment variable {L}{env}{T}");
    }

    fn write_buf(&mut self, args: std::fmt::Arguments<'_>) {
        use std::fmt::Write as _;
        _ = self.mut_buf().write_fmt(args);
    }

    fn write_named_item(&mut self, named: &crate::Named, meta: Option<crate::Metavar>) {
        let Some(sl) = named.get_shortlong() else {
            return;
        };
        match meta {
            None => self.write_buf(format_args!("{sl:#}")),
            Some(meta) => self.write_buf(format_args!("{sl:#}={meta}")),
        }
        self.track_tab(sl.col_width() + meta.map_or(0, |m| 1 + m.width()));
        self.help(named.help);
        if let Some(env) = named.env.first() {
            self.write_env(env);
        }
    }

    fn update_place(&mut self, item: &Item) {
        if self.depth == 0 {
            self.place = match item {
                _ if self.in_global > 0 => Place::Global,
                Item::Flag { .. } | Item::Arg { .. } => Place::Named,
                Item::Positional { .. } => Place::Pos,
                Item::Command { .. } => Place::Command,
                Item::Nested {
                    outer: Nest::Named(_),
                    ..
                } => Place::Named,
                Item::Nested {
                    outer: Nest::Keyword(_),
                    ..
                } => Place::Command,
                _ => self.place,
            };
        }
    }

    fn copy_text(&mut self, tab: bool, text: &str) {
        // Preserve linebreaks followed by a line that starts with a space.
        // Preserve empty lines.
        // Linebreaks are removed otherwise.
        let mut first = true;
        let mut prev_empty = false;
        for line in text.lines() {
            if !first {
                let join = if prev_empty || line.starts_with(' ') || line.is_empty() {
                    '\n'
                } else {
                    ' '
                };
                self.mut_buf().push(join);

                if tab && join == '\n' {
                    self.mut_buf().push('\t');
                }
            }
            self.mut_buf().push_str(line);

            prev_empty = line.is_empty();
            first = false;
        }
        self.mut_buf().push('\n');
    }

    fn written_chars_since(&self, before: usize) -> usize {
        let written = &self.buf()[before..];
        crate::miniansi::text_len(written)
    }

    fn help(&mut self, help: Option<&str>) {
        self.mut_buf().push('\t');
        if let Some(mut help) = help {
            if self.detailed == crate::Help::Brief {
                help = help.split_once("\n\n").map_or(help, |h| h.0);
            }
            self.copy_text(true, help);
        } else {
            self.mut_buf().push('\n');
        }
    }

    fn prepare_output(&mut self) {
        use std::fmt::Write as _;
        if !self.descr.is_empty() {
            self.output.push_str(&self.descr);
            self.output.push('\n');
        }

        if !self.usage.is_empty() {
            // with generated usage there won't be a '\n' at the end, but
            // writing directly to section will add it.
            let usage = self.usage.trim_end();
            _ = writeln!(self.output, "Usage: {usage}");
        }

        if !self.header.is_empty() {
            self.output.push('\n');
            self.output.push_str(&self.header);
        }

        if !self.pos.is_empty() {
            self.output.push('\n');
            _ = writeln!(&mut self.output, "{H}Available positional items:{T}");
            self.output.push_str(&self.pos);
        }

        if !self.sections.is_empty() {
            self.output.push('\n');
            self.output.push_str(&self.sections);
        }

        if !self.named.is_empty() {
            self.output.push('\n');
            _ = writeln!(&mut self.output, "{H}Available options:{T}");
            self.output.push_str(&self.named);
        }

        if !self.commands.is_empty() {
            self.output.push('\n');
            _ = writeln!(&mut self.output, "{H}Available commands:{T}");
            self.output.push_str(&self.commands);
        }

        if !self.global.is_empty() {
            self.output.push('\n');
            _ = writeln!(&mut self.output, "{H}Global options:{T}");
            self.output.push_str(&self.global);
        }

        if !self.footer_buf.is_empty() {
            self.output.push('\n');
            self.output.push_str(&self.footer_buf);
        }
    }
}
