//! Overall rendering takes 2 stages:
//! 1. collect info from the visitor. At this point we don't know what the tabstop is
//!    so can't really render it into the final version. Info is collected into several
//!    strings: one per predefined section, one for current section, one for accumulated sections.
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

use super::ShortLong;
use crate::{
    Flag, Item, Nest, VKind,
    console_writer::{MAX_TAB, Style, char_width},
    traits::Gr,
    visitors::{VisitGroup, Visitor, usage::Usage},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Lit<'a>(pub(crate) ShortLong<'a>);

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "    ")?;
        }
        match self.0 {
            ShortLong::Short(s) => write!(f, "{L}{s}{T}"),
            ShortLong::Long(l) => write!(f, "{L}{l}{T}"),
            ShortLong::Both(s, l) => write!(f, "{L}{s}{T}, {L}{l}{T}"),
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
enum Place {
    #[default]
    Named,
    Pos,
    Command,
    Section,
    Body,
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default)]
pub struct Help<'a> {
    pub(crate) path: &'a str,
    place: Place,
    footer: Option<&'a str>,
    in_section: usize,

    /// current section, if in one, empty otherwise
    current: String,
    /// All the nonstandard sections, can be empty
    sections: String,

    /// Default sections
    named: String,
    pos: String,
    commands: String,
    /// Maximum seen tab slice (under the limit)
    max_tab: usize,

    pub(crate) detailed: bool,

    output: String,
}

impl std::ops::Index<Place> for Help<'_> {
    type Output = String;
    fn index(&self, index: Place) -> &Self::Output {
        match index {
            Place::Named => &self.named,
            Place::Pos => &self.pos,
            Place::Command => &self.commands,
            Place::Section => &self.current,
            Place::Body => &self.output,
        }
    }
}

impl std::ops::IndexMut<Place> for Help<'_> {
    fn index_mut(&mut self, index: Place) -> &mut Self::Output {
        match index {
            Place::Named => &mut self.named,
            Place::Pos => &mut self.pos,
            Place::Command => &mut self.commands,
            Place::Section => &mut self.current,
            Place::Body => &mut self.output,
        }
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item(&mut self, item: Item<'a>) {
        use std::fmt::Write as _;

        let place = self.place_for(&item);
        match item {
            Item::OptionParser { info, inner } => {
                if let Some(descr) = info.descr {
                    self.copy_text(Place::Body, descr);
                    self.output.push('\n');
                }
                _ = write!(&mut self.output, "Usage: {} ", self.path);
                if let Some(usage) = info.usage {
                    self.output.push_str(usage);
                } else {
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.vi(&mut usage);
                    usage.render_to(&mut self.output);
                }
                self.output.push('\n');

                if let Some(header) = info.header {
                    self.output.push('\n');
                    self.copy_text(Place::Body, header);
                }

                self.footer = info.footer;
            }
            Item::Flag { named } => {
                let Some(sl) = named.get_shortlong() else {
                    return;
                };

                _ = write!(&mut self[place], "{sl:#}");
                self.track_tab(sl.col_width());
                self.help(place, named.help);

                if let Some(env) = named.env.first() {
                    _ = if std::env::var_os(env).is_some() {
                        writeln!(&mut self[place], "\t[env:{env} is set]")
                    } else {
                        writeln!(&mut self[place], "\t[env:{env} is not set]")
                    }
                }
            }
            Item::Arg { named, meta } => {
                let Some(sl) = named.get_shortlong() else {
                    return;
                };

                _ = write!(&mut self[place], "{sl:#}={meta}");
                self.track_tab(sl.col_width() + 1 + meta.width());
                self.help(place, named.help);
                if let Some(env) = named.env.first() {
                    _ = match std::env::var_os(env) {
                        Some(v) => {
                            writeln!(&mut self[place], "\t[env:{env}: {}]", v.to_string_lossy())
                        }
                        None => writeln!(&mut self[place], "\t[env:{env}: N/A]"),
                    }
                }
            }
            Item::Positional { meta, help } => {
                _ = write!(&mut self[place], "    {meta}");
                self.track_tab(meta.width());
                self.help(place, help);
            }
            Item::Command {
                names,
                info,
                inner: _,
            } => {
                let name = lit_name(names);
                let help = info.descr;
                _ = write!(&mut self[place], "{name:#}");
                self.track_tab(name.col_width());
                self.help(place, help);
            }
            Item::Nested { outer, inner } => {
                let before = self[place].len();

                let help = match outer {
                    Nest::Named(Flag { named, .. }) => {
                        let Some(name) = named.get_shortlong() else {
                            // pure env nested parser, makes little sense.
                            return;
                        };
                        _ = write!(&mut self[place], "{:#} ", name);
                        named.help
                    }
                    Nest::Keyword(keyword) => {
                        let name = lit_name(&keyword.named.names);
                        _ = write!(&mut self[place], "{:#} ", name);
                        keyword.named.info.descr
                    }
                };

                let mut u = Usage::default();
                inner.vi(&mut u);
                u.render_to(&mut self[place]);

                self.track_tab(self.written_chars_since(place, before));
                self.help(place, help);

                self.in_section += 1;
                inner.vi(self);
                self.in_section -= 1;
                if self.in_section == 0 {
                    let mut tmp = std::mem::take(&mut self.current);
                    self[place].push_str(&tmp);
                    std::mem::swap(&mut tmp, &mut self.current);
                    self.current.clear();
                }
            }
            Item::Section {
                title,
                descr,
                inner,
            } => {
                self.in_section += 1;
                inner.vi(self);
                self.in_section -= 1;
                // throw away inner nested sections
                if self.in_section == 0 {
                    _ = writeln!(&mut self.sections, "{H}{title}{T}");
                    self.sections.push_str(&self.current);
                    assert_eq!(descr, None);
                    self.current.clear();
                }
            }
            Item::Rendered { text, gr } => {
                for line in text.lines() {
                    if let Some((key, _)) = line.split_once('\t') {
                        self.track_tab(
                            crate::miniansi::split(key)
                                .map(|c| match c {
                                    crate::miniansi::Frag::Str(s) => char_width(s),
                                    crate::miniansi::Frag::Code(_) => 0,
                                })
                                .sum(),
                        );
                    }
                }

                let place = gr.map_or(place, |gr| gr.into());
                self[place].push_str(text);
                self[place].push('\n');
            }
        }
    }

    fn push_group(&mut self, _: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> VKind {
        VKind::Help
    }
}

impl From<Gr> for Place {
    fn from(value: Gr) -> Self {
        match value {
            Gr::Named => Self::Named,
            Gr::Pos => Self::Pos,
            Gr::Cmd => Self::Command,
        }
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

    fn place_for(&mut self, item: &Item) -> Place {
        self.place = match &item {
            _ if self.in_section > 0 => Place::Section,
            Item::Flag { .. } | Item::Arg { .. } => Place::Named,
            Item::Positional { .. } => Place::Pos,
            Item::Command { .. } => Place::Command,
            Item::Rendered {
                gr: Some(place), ..
            } => Place::from(*place),
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
        self.place
    }

    fn copy_text(&mut self, place: Place, text: &str) {
        self[place].push_str(text);
        self[place].push('\n');
    }

    fn written_chars_since(&self, place: Place, before: usize) -> usize {
        let written = &self[place][before..];
        crate::miniansi::split(written)
            .map(|c| match c {
                crate::miniansi::Frag::Str(s) => char_width(s),
                crate::miniansi::Frag::Code(_) => 0,
            })
            .sum()
    }

    fn help(&mut self, place: Place, help: Option<&str>) {
        self[place].push('\t');
        if let Some(mut help) = help {
            if !self.detailed {
                help = help.split_once("\n\n").map_or(help, |h| h.0);
            }
            self.copy_text(place, help);
        } else {
            self[place].push('\n');
        }
    }

    pub(crate) fn render(mut self) -> String {
        use std::fmt::Write as _;
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

        if let Some(footer) = self.footer {
            self.output.push('\n');
            self.output.push_str(footer);
        }

        crate::console_writer::apply_style(&self.output, self.max_tab + 2, None)
    }
}

#[test]
fn flag_equivalence() {
    use crate::*;
    let parser = short('a').switch().help("help");
    let mut h1 = Help::default();

    parser.visit(&mut h1);

    let mut h2 = Help::default();
    let t = format!("    {L}-a{T}\thelp");
    h2.item(Item::Rendered {
        text: &t,
        gr: Some(Gr::Named),
    });
    assert_eq!(h1, h2);
}
