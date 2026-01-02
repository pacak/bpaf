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
const M: &str = Style::Metavar.ansi();
const L: &str = Style::Literal.ansi();
const H: &str = Style::Header.ansi();

use std::borrow::Cow;

use super::ShortLong;
use crate::{
    Item, Metavar, Named, VKind,
    adapters::Info,
    console_writer::{
        Atom, Colorscheme, ConsoleWriter, MAX_TAB, MAX_WIDTH, Pending, Style, char_width,
        linesplit, word_width,
    },
    visitors::{VisitGroup, Visitor, usage::Usage},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Lit<'a>(pub(crate) ShortLong<'a>);

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
            crate::Name::Long(l) => long = long.or(Some(&*l)),
        }
    }
    Lit(match (short, long) {
        (None, None) => panic!("must have a single name"),
        (None, Some(l)) => ShortLong::Long(l),
        (Some(s), None) => ShortLong::Short(*s),
        (Some(s), Some(l)) => ShortLong::Both(*s, l),
    })
}

#[derive(Debug, PartialEq, Eq, Hash)]
/// No text should include a closing newline, each item gets placed on a separate line
pub(crate) enum HelpItem<'a> {
    /// An argument or a flag
    Named {
        name: ShortLong<'a>,
        meta: Option<Metavar>,
        /// If present - render it after a tabstop position,
        help: Option<&'a str>,
    },
    /// A positional item - Metavar + help
    Pos {
        meta: Metavar,
        /// If present - render it after a tabstop position
        help: Option<&'a str>,
    },
    /// A command: name/short name + help
    Cmd {
        name: Lit<'a>,
        /// If present - render it after a tabstop position
        help: Option<&'a str>,
    },
    /// An arbitrary piece of text that gets wrapped to the MAX_WIDTH.
    Text {
        /// If text contains a TAB:
        /// - text before tab is left wrapped to lpad
        /// - text after tab is left wrapped to tabstop
        text: Cow<'a, str>,
        lpad: usize,
        tabstop: usize,
        /// Should the text be to the left or to the right of the tabstop
        after_tab: bool,
    },
    /// A single blank line
    Blank,
    /// Section header, a specialized text
    /// - start a new paragraph
    /// - wrap it into [`Style::Header`] / [`Style::Text`]
    Header {
        text: &'a str,
    },
    Atom(Vec<Atom<'a>>),
}

#[derive(Debug)]
pub(crate) struct Section<'a> {
    pub(crate) header: &'a str,
    pub(crate) descr: Option<&'a str>,
    pub(crate) items: Vec<HelpItem<'a>>,
}

#[derive(Debug, Default)]
pub(crate) struct Help<'a> {
    usage: String,
    info: Option<&'a Info>,
    sections: Vec<Section<'a>>,
    in_section: u32,
    max_word: usize,

    current: Vec<HelpItem<'a>>,
    named: Vec<HelpItem<'a>>,
    pos: Vec<HelpItem<'a>>,
    command: Vec<HelpItem<'a>>,
    pub(crate) app_name: Option<&'a str>,
    place: Place,
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

impl<'a> std::ops::Index<Place> for Help<'a> {
    type Output = Vec<HelpItem<'a>>;

    fn index(&self, index: Place) -> &Self::Output {
        match index {
            Place::Named => &self.named,
            Place::Pos => &self.pos,
            Place::Command => &self.command,
            Place::Section => &self.current,
            Place::Body => todo!(),
        }
    }
}
impl<'a> std::ops::IndexMut<Place> for Help<'a> {
    fn index_mut(&mut self, index: Place) -> &mut Self::Output {
        match index {
            Place::Named => &mut self.named,
            Place::Pos => &mut self.pos,
            Place::Command => &mut self.command,
            Place::Section => &mut self.current,
            Place::Body => todo!(),
        }
    }
}

impl Named {
    /// Try to represent [`Named`] as a [`HelpItem`]
    ///
    /// Pure env items are not shown. Also returns a name so we can track the tabstop position
    fn help_item(&self, meta: Option<Metavar>) -> Option<(ShortLong<'_>, HelpItem<'_>)> {
        let name = self.get_shortlong()?;
        let item = HelpItem::Named {
            name,
            meta,
            help: self.help,
        };
        Some((name, item))
    }
}

impl Help<'_> {
    fn track_length(&mut self, name: ShortLong<'_>, meta: Option<Metavar>) {
        let this = name.width() + meta.map_or(0, |m| m.width() + 1);
        if this <= MAX_TAB {
            self.max_word = self.max_word.max(this);
        }
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item(&mut self, item: Item<'a>) {
        self.place = match &item {
            _ if self.in_section > 0 => Place::Section,
            Item::Flag { .. } | Item::Arg { .. } => Place::Named,
            Item::Positional { .. } => Place::Pos,
            Item::Command { .. } => Place::Command,
            _ => self.place,
        };
        let place = self.place;
        match item {
            Item::Flag { named } => {
                let Some((name, item)) = named.help_item(None) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, None);
                self[place].push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(_) => format!("\t[env:{env} is set]"),
                    None => format!("\t[env:{env} is not set]"),
                });
                self[place].push(HelpItem::Text {
                    text,
                    lpad: 0, // TODO
                    tabstop: 0,
                    after_tab: true,
                });
            }
            Item::Arg { named, meta } => {
                let Some((name, item)) = named.help_item(Some(meta)) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, Some(meta));
                self[place].push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(v) => format!("\t[env:{env}: {}]", v.to_string_lossy()),
                    None => format!("\t[env:{env}: N/A]"),
                });
                self[place].push(HelpItem::Text {
                    text,
                    lpad: 0, // TODO
                    tabstop: 0,
                    after_tab: true,
                });
            }
            Item::Positional { meta, help } => {
                self[place].push(HelpItem::Pos { meta, help });
            }
            Item::Command {
                names,
                info,
                inner: _,
            } => {
                let name = lit_name(names);
                let help = info.descr;
                self[place].push(HelpItem::Cmd { name, help });
            }
            Item::Nested { named, inner } => {
                let Some((name, item)) = named.help_item(None) else {
                    // pure env nested parser, makes little sense.
                    return;
                };

                let mut u = Usage::default();
                inner.visit(&mut u);
                let mut usage = String::new();
                u.render_to(&mut usage);

                let mut a = Vec::with_capacity(8);
                a.push(Atom::NextHelpItem);

                let this = name.width() + 1 + word_width(&usage, false);
                if this <= MAX_TAB {
                    self.max_word = self.max_word.max(this);
                }
                assert_ne!(&usage, "");

                a.push(Atom::Name(name));
                a.push(Atom::Space);
                a.push(Atom::Text {
                    text: Cow::Owned(usage),
                    split: false,
                });

                if let Some(h) = named.help {
                    a.push(Atom::TabState(true));
                    a.push(Atom::Text {
                        text: Cow::Borrowed(h),
                        split: true,
                    });
                }
                self[place].push(HelpItem::Atom(a));

                self.in_section += 1;
                inner.visit(self);
                self.in_section -= 1;
                if self.in_section == 0 {
                    let mut tmp = std::mem::take(&mut self.current);
                    self[place].append(&mut tmp);
                    std::mem::swap(&mut tmp, &mut self.current);
                }
                self[place].push(HelpItem::Blank);
            }
            Item::OptionParser { info, inner } => {
                if let Some(usage) = info.usage {
                    self.usage = usage.to_owned();
                } else {
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.visit(&mut usage);
                    self.usage = match self.app_name {
                        Some(name) => format!("Usage: {name} "),
                        None => "Usage: ".to_owned(),
                    };
                    usage.render_to(&mut self.usage);
                }
                self.info = Some(info);
            }
            Item::Section {
                title,
                descr,
                inner,
            } => {
                self.in_section += 1;
                inner.visit(self);
                self.in_section -= 1;
                // throw away inner nested sections
                if self.in_section == 0 {
                    self.sections.push(Section {
                        header: title,
                        descr,
                        items: std::mem::take(&mut self.current),
                    });
                }
            }
            Item::Rendered { text, gr } => self[place].push(HelpItem::Text {
                text: text.into(),
                lpad: 0,
                tabstop: 0,
                after_tab: true,
            }),
        }
    }

    fn push_group(&mut self, _group: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> VKind {
        VKind::Help
    }
}

impl<'a> Help<'a> {
    /// Render collected help into console `--help` output
    ///
    /// It should render the following items, in order
    /// - header
    /// - usage line
    /// - many of
    ///   - section title
    ///   - section description
    ///   - section items
    /// - footer
    ///
    /// Items come in 3 horizontal bits:
    /// - short flag or short flag placeholder
    /// - long flag
    /// - item description.
    /// long flag can push the description to the left but otherwise is padded
    pub(crate) fn render(mut self, detailed: bool) -> String {
        let mut w = ConsoleWriter::new(None, self.max_word + 6, detailed);

        if let Some(text) = self.info.and_then(|i| i.descr) {
            w.write_text(text);
            w.paragraph();
        }

        // TODO
        w.write_text(&self.usage);
        w.paragraph();

        if let Some(text) = self.info.and_then(|i| i.header) {
            w.write_text(text);
            w.paragraph();
        }

        let positional = Section {
            header: "Available positional items:",
            descr: None,
            items: std::mem::take(&mut self.pos),
        };

        let cmds = Section {
            header: "Available commands:",
            descr: None,
            items: std::mem::take(&mut self.command),
        };

        let named = Section {
            header: "Available options:",
            descr: None,
            items: std::mem::take(&mut self.named),
        };

        w.write_section(positional);
        for section in self.sections {
            w.write_section(section);
        }
        w.write_section(named);
        w.write_section(cmds);

        if let Some(text) = self.info.and_then(|i| i.footer) {
            w.paragraph();
            w.write_text(text);
            w.newline();
        }

        w.done()
    }
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default)]
pub struct Help2<'a> {
    pub(crate) app_name: Option<&'a str>,
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
    /// Width of the current tab slice
    width: usize,
    /// Maximum seen tab slice (under the limit)
    max_tab: usize,

    detailed: bool,

    output: String,
    pending: Pending,

    column: Column,
    column_dirty: bool,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum Column {
    #[default]
    Text,
    Item,
    Help,
}
impl Column {
    fn inc(&self) -> Column {
        match self {
            Column::Text => Column::Item,
            Column::Item => Column::Help,
            Column::Help => Column::Text,
        }
    }
}

impl std::ops::Index<Place> for Help2<'_> {
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

impl std::ops::IndexMut<Place> for Help2<'_> {
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

// Columns are separated by '\t', `\n' preserves

impl<'a> Visitor<'a> for Help2<'a> {
    fn item(&mut self, item: Item<'a>) {
        use std::fmt::Write as _;

        let place = self.place_for(&item);
        match item {
            Item::OptionParser { info, inner } => {
                if let Some(descr) = info.descr {
                    // shouldn't contain tabs, but you never know...
                    self.write_text(Place::Body, descr);
                    self.output.push_str("\n\n");
                    self.column = Column::Text;
                }

                _ = match self.app_name {
                    Some(name) => write!(&mut self.output, "Usage: {name} "),
                    None => write!(&mut self.output, "Usage: "),
                };
                if let Some(usage) = info.usage {
                    self.output.push_str(usage);
                } else {
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.visit(&mut usage);
                    usage.render_to(&mut self.output);
                }
                self.output.push_str("\n\n");

                if let Some(header) = info.header {
                    self.output.push('\n');
                    self.write_text(Place::Body, header);
                    self.column = Column::Text;
                    self.output.push('\n');
                    self.output.push('\n');
                }

                self.footer = info.footer;
            }
            Item::Flag { named } => {
                let Some(sl) = named.get_shortlong() else {
                    return;
                };
                self.change_column(place, Column::Item);
                _ = write!(&mut self[place], "{L}{sl}{T}");
                self.width = sl.width();
                self.help(place, named.help);
            }
            Item::Arg { named, meta } => {
                let Some(sl) = named.get_shortlong() else {
                    return;
                };
                self.change_column(place, Column::Item);
                _ = write!(&mut self[place], "{L}{sl}{T}={M}{meta}{T}");
                self.width = sl.width() + 1 + meta.width();
                self.help(place, named.help);
            }
            Item::Positional { meta, help } => {
                self.change_column(place, Column::Item);
                _ = write!(&mut self[place], "{M}{meta}{T}");
                self.help(place, help);
            }
            Item::Command { names, info, inner } => todo!(),
            Item::Nested { named, inner } => todo!(),
            Item::Section {
                title,
                descr,
                inner,
            } => todo!(),
            Item::Rendered { text, gr: place } => {
                // TODO - I need to know which group to put things into
                todo!();
            }
        }
    }

    fn push_group(&mut self, _: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> VKind {
        VKind::Help
    }
}

impl Help2<'_> {
    fn place_for(&mut self, item: &Item) -> Place {
        self.place = match &item {
            _ if self.in_section > 0 => Place::Section,
            Item::Flag { .. } | Item::Arg { .. } => Place::Named,
            Item::Positional { .. } => Place::Pos,
            Item::Command { .. } => Place::Command,
            _ => self.place,
        };
        self.place
    }

    fn change_column(&mut self, place: Place, column: Column) {
        if column == self.column {
            return;
        }
        if self.column == Column::Item && column != Column::Item {
            if self.width < MAX_TAB {
                self.max_tab = self.width;
            }
            self.width = 0;
        }
        let missing = (3 + column as usize - self.column as usize) % 3;
        self.column = column;
        self[place].extend(std::iter::repeat_n('\t', missing));
    }

    fn handle_pending(&mut self) {
        match self.pending {
            Pending::Nothing => {}
            Pending::Space => {
                self.output.push(' ');
                self.width += 1;
            }
            Pending::TabSep => {
                self.output.push_str("  ");
                self.width = 0;
            }
            Pending::Newline => {
                self.output.push('\n');
                self.width = 0;
            }
            Pending::Paragraph => {
                self.output.push_str("\n\n");
                self.width = 0;
            }
        }
    }

    /// During the first pass
    /// - don't expand tabs
    /// - don't break long lines
    /// - preserve linesplits
    fn write_text(&mut self, place: Place, mut text: &str) {
        if !self.detailed
            && let Some((prefix, _)) = text.split_once("\n\n")
        {
            text = prefix;
        }
        for chunk in minisplit(text) {
            match chunk {
                Chunk::Newline => {
                    self.width = 0;
                    self[place].push('\n');
                }
                Chunk::Tab => {
                    self.width = 0;
                    self[place].push('\t');
                    self.change_column(self.place, self.column.inc());
                }
                Chunk::Text(text) => {
                    self[place].push_str(text);
                    if self.column == Column::Item {
                        self.width += word_width(text, false);
                    }
                }
            }
        }
    }

    fn help(&mut self, place: Place, help: Option<&str>) {
        if let Some(help) = help {
            self.change_column(place, Column::Help);
            self[place].push_str(help);
        }
    }

    pub(crate) fn render(mut self) -> String {
        use std::fmt::Write as _;
        if !self.pos.is_empty() {
            _ = write!(&mut self.output, "{H}Available positional items:{T}\n ");
            self.output.push_str(&self.pos);
        }
        if !self.sections.is_empty() {
            self.output.push('\n');
            self.output.push_str(&self.sections);
            self.output.push('\n');
        }

        if !self.named.is_empty() {
            _ = write!(&mut self.output, "{H}Available options:{T}\n ");
            self.output.push_str(&self.named);
        }

        if !self.commands.is_empty() {
            _ = write!(&mut self.output, "{H}Available commands:{T}\n ");
            self.output.push_str(&self.commands);
        }

        if let Some(footer) = self.footer {
            self.output.push('\n');
            self.output.push_str(footer);
        }

        self.pending = Pending::Nothing;
        self.column = Column::Text;

        println!("{:?}", self.output);

        let mut pen = None;
        for c in linesplit(&std::mem::take(&mut self.output), false) {
            println!("col: {:?}; {c:?}", self.column);
            match c {
                crate::console_writer::Chunk::Word { width, text } => {
                    self.width += width;
                    if self.width > MAX_WIDTH {
                        self.pending = Pending::Newline;
                    }
                    if let Some(pad) = pen.take() {
                        println!("{pad:?} before {text:?}");
                        self.output.extend(std::iter::repeat_n(' ', pad));
                    } else {
                        self.handle_pending();
                    }
                    self.output.push_str(text);
                    self.pending = Pending::Space;
                }
                crate::console_writer::Chunk::Tab => {
                    self.column = self.column.inc();
                    match self.column {
                        Column::Text => self.pending = Pending::Newline,
                        Column::Item => {
                            pen = Some(4);
                            self.pending = Pending::Space;
                        }
                        Column::Help => {
                            pen = Some(self.max_tab);
                            self.pending = Pending::Space;
                        }
                    }
                }
                crate::console_writer::Chunk::LineBreak => {
                    self.pending = Pending::Newline;
                }
                crate::console_writer::Chunk::Paragraph => {
                    self.pending = Pending::Paragraph;
                }
            }
        }

        crate::console_writer2::apply_style(&self.output, self.max_tab, None)
    }
}

#[test]
fn changing_column_works() {
    let mut h = Help2::default();
    h.change_column(Place::Named, Column::Item);
    h.width = 12;
    assert_eq!(h.named, "\t");
    h.change_column(Place::Named, Column::Help);
    assert_eq!(h.width, 0);
    assert_eq!(h.max_tab, 12);
    assert_eq!(h.named, "\t\t");
    h.change_column(Place::Named, Column::Text);
    assert_eq!(h.named, "\t\t\t");
    h.change_column(Place::Named, Column::Help);
    assert_eq!(h.named, "\t\t\t\t\t");
}

#[test]
fn flag_equivalence() {
    use crate::*;
    let parser = short('a').switch().help("help");
    let mut h1 = Help2::default();

    parser.visit(&mut h1);

    let mut h2 = Help2::default();
    let t = format!("{L}-a{T}\thelp");
    h2.item(Item::Rendered {
        text: &t,
        gr: Some(Gr::Named),
    });
    assert_eq!(h1, h2);
}

#[derive(Debug, PartialEq, Eq)]
enum Chunk<'a> {
    Newline,
    Tab,
    Text(&'a str),
}

fn minisplit(input: &str) -> impl Iterator<Item = Chunk<'_>> {
    let mut input = input;

    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }

        match input.char_indices().find(|&(_, c)| c == '\n' || c == '\t') {
            Some((i, sep)) => {
                if i > 0 {
                    let text = &input[..i];
                    input = &input[i..];
                    Some(Chunk::Text(text))
                } else {
                    input = &input[1..];
                    match sep {
                        '\n' => Some(Chunk::Newline),
                        '\t' => Some(Chunk::Tab),
                        _ => unreachable!(),
                    }
                }
            }
            None => {
                if input.is_empty() {
                    None
                } else {
                    Some(Chunk::Text(std::mem::take(&mut input)))
                }
            }
        }
    })
}
