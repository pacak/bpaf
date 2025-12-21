use std::borrow::Cow;

use super::ShortLong;
use crate::{
    Con, Item, Metavar, Named, VKind,
    adapters::Info,
    visitors::{VisitGroup, Visitor},
};

const MAX_WIDTH: usize = 100;

#[derive(Debug)]
struct Lit<'a>(ShortLong<'a>);

#[derive(Debug)]
/// No text should include a closing newline, each item gets placed on a separate line
enum HelpItem<'a> {
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
    },
    /// Section header, a specialized text
    /// - start a new paragraph
    /// - wrap it into [`Style::Header`] / [`Style::Text`]
    Header { text: &'a str },
}

#[derive(Debug)]
struct Section<'a> {
    header: &'a str,
    descr: Option<&'a str>,
    items: Vec<HelpItem<'a>>,
}

#[derive(Debug, Default)]
pub(crate) struct Help<'a> {
    usage: String,
    info: Option<&'a Info>,
    sections: Vec<Section<'a>>,
    in_section: bool,
    max_word: usize,

    current: Vec<HelpItem<'a>>,
    named: Vec<HelpItem<'a>>,
    pos: Vec<HelpItem<'a>>,
    command: Vec<HelpItem<'a>>,
}

impl Named {
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
        let meta = meta.map_or(0, |m| m.width() + 1);
        match name {
            ShortLong::Short(_) => {
                self.max_word = self.max_word.max(2 + meta); // `-a`
            }
            ShortLong::Long(l) | ShortLong::Both(_, l) => {
                self.max_word = self.max_word.max(width(l) + 6 + meta);
            }
        }
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item(&mut self, item: Item<'a>) {
        match item {
            Item::Flag { named } => {
                let Some((name, item)) = named.help_item(None) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, None);
                let place = if self.in_section {
                    &mut self.current
                } else {
                    &mut self.named
                };
                place.push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(_) => format!("\t[{env} is set]"),
                    None => format!("\t[{env} is not set]"),
                });
                place.push(HelpItem::Text {
                    text,
                    lpad: 0,    // TODO
                    tabstop: 0, // TODO
                });
            }
            Item::Arg { named, meta } => {
                let Some((name, item)) = named.help_item(Some(meta)) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, Some(meta));
                let place = if self.in_section {
                    &mut self.current
                } else {
                    &mut self.named
                };
                place.push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(v) => format!("\t[{env} = {}]", v.to_string_lossy()),
                    None => format!("\t[{env} N/A]"),
                });
                place.push(HelpItem::Text {
                    text,
                    lpad: 0,    // TODO
                    tabstop: 0, // TODO
                });
            }
            Item::Positional { meta, help } => {
                let place = if self.in_section {
                    &mut self.current
                } else {
                    &mut self.pos
                };
                place.push(HelpItem::Pos { meta, help });
            }
            Item::Command { names, info, inner } => {
                let place = if self.in_section {
                    &mut self.current
                } else {
                    &mut self.command
                };
                todo!()
            }
            Item::Nested { named, inner } => {
                let place = if self.in_section {
                    &mut self.current
                } else {
                    &mut self.named
                };
                todo!()
            }
            Item::OptionParser { info, inner } => {
                if let Some(usage) = info.usage {
                    self.usage = usage.to_owned();
                } else {
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.visit(&mut usage);
                    self.usage = usage.render();
                }
                self.info = Some(info);
            }
            Item::Section {
                title,
                descr,
                inner,
            } => {
                self.in_section = true;
                inner.visit(self);
                self.sections.push(Section {
                    header: title,
                    descr,
                    items: std::mem::take(&mut self.current),
                });
            }
            Item::Rendered { text } => todo!(),
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

    pub(crate) fn render(mut self) -> String {
        let mut w = ConsoleWriter::new(None, self.max_word + 6);

        if let Some(text) = self.info.and_then(|i| i.descr) {
            w.write_text(text);
            w.newline();
        }

        // TODO
        w.write_text("Usage: ");
        w.write_text(&self.usage);
        w.newline();
        w.newline();
        if let Some(text) = self.info.and_then(|i| i.header) {
            w.write_text(text);
            w.newline();
        }

        let positional = Section {
            header: "Positional items:",
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
        w.write_section(cmds);
        w.write_section(named);

        if let Some(text) = self.info.and_then(|i| i.footer) {
            w.write_text(text);
            w.newline();
        }

        w.done()
    }
}

const BLANK: &'static str = "                                                    ";

#[derive(Debug, Copy, Clone)]
enum Chunk<'a> {
    /// A single word along with ANSI decorations
    Word {
        /// Word width in characters (ignores ANSI decoration)
        width: usize,
        text: &'a str,
    },
    Tab,
    /// Insert a line break but retain the left margin
    LineBreak,
    /// A new paragraph - resets the left margin
    Paragraph,
}

/// Split the string into components that can be filled back in into a fixed width
///
/// Output doesn't contain spaces between words
/// 1. `"\n\n"` - Paragraph
/// 2. `"\n "` - LineBreak
/// 3. `"\n"` - separates words
/// 4. `"\n    "` - code block - must start with 4 spaces, must not contain empty lines
/// 5. take next word
fn split(input: &str, mono: bool) -> Splitter<'_> {
    Splitter { input, mono }
}

#[derive(Debug)]
struct Splitter<'a> {
    input: &'a str,
    mono: bool,
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Chunk<'a>;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tail) = self.input.strip_prefix('\n') {
            if let Some(code) = tail.strip_prefix("    ") {
                // `_rest` doesn't contain leading '\n' - not useful for iteration purposes
                if let Some((code, _rest)) = code.split_once('\n') {
                    self.input = &tail[code.len()..];
                    Some(word(code, self.mono))
                } else {
                    self.input = "";
                    Some(word(tail, self.mono))
                }
            } else if let Some(tail) = tail.strip_prefix(' ') {
                self.input = tail;
                Some(Chunk::LineBreak)
            } else {
                match tail.split_once(' ') {
                    Some((this, rest)) => {
                        self.input = rest;
                        Some(word(this, self.mono))
                    }
                    None => {
                        self.input = "";
                        Some(word(tail, self.mono))
                    }
                }
            }
        } else if self.input.is_empty() {
            None
        } else if let Some(tail) = self.input.strip_prefix('\t') {
            self.input = tail;
            Some(Chunk::Tab)
        } else {
            match self.input.split_once(' ') {
                Some((this, rest)) => {
                    self.input = rest;
                    Some(word(this, self.mono))
                }
                None => {
                    let input = self.input;
                    self.input = "";
                    Some(word(input, self.mono))
                }
            }
        }
    }
}

/// Calculate width in visible characters, ignores `"\{1b}[Dm"` where D is a single digit
#[inline(never)]
fn word(text: &str, mono: bool) -> Chunk<'_> {
    let fixup = if mono { 2 } else { 0 };
    #[derive(Copy, Clone)]
    enum Goal {
        Esc,
        Bracket,
        Digit,
        M,
    }
    let mut looking_for = Goal::Esc;
    let mut width = 0;

    for c in text.chars() {
        width += 1;
        looking_for = match (c, looking_for) {
            ('\u{1B}', Goal::Esc) => Goal::Bracket,
            ('[', Goal::Bracket) => Goal::Digit,
            (d, Goal::Digit) if d.is_ascii_digit() => {
                // account for wrapping valid and invalid inputs in `` in monochrome mode
                // Assumption: fragment contains both - opening and closing valid/invalid tags
                if d == '6' || d == '9' {
                    width += fixup;
                }
                Goal::M
            }
            ('m', Goal::M) => {
                width -= 4;
                Goal::Esc
            }
            _ => Goal::Esc,
        }
    }
    Chunk::Word { width, text }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ansi())
    }
}

impl Style {
    /// Placeholder ANSI values
    pub const fn ansi(&self) -> &'static str {
        // https://en.wikipedia.org/wiki/ANSI_escape_code?useskin=vector#SGR
        match self {
            Style::Text => "\u{1B}[0m",     // reset/normal
            Style::Emphasis => "\u{1B}[1m", // bold
            Style::Literal => "\u{1B}[2m",  // faint
            Style::Metavar => "\u{1B}[3m",  // italic
            Style::Header => "\u{1B}[4m",   // underline
            Style::Valid => "\u{1B}[5m",    // rapid blink
            Style::Invalid => "\u{1B}[6m",  // crossed-out
        }
    }
    const TEXT: u8 = b'0';
    const EMPHASIS: u8 = b'1';
    const LITERAL: u8 = b'2';
    const METAVAR: u8 = b'3';
    const HEADER: u8 = b'4';
    const VALID: u8 = b'5';
    const INVALID: u8 = b'6';
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    /// Plain text, no decorations
    Text,

    /// Word with emphasis - things like “Usage”, “Available options”, etc
    Emphasis,

    /// Something user needs to type literally - command names, etc
    Literal,

    /// Metavar placeholder - something user needs to replace with own input
    Metavar,

    /// Section header
    Header,

    /// Valid input given by user
    Valid,

    /// Invalid input given by user - used to display invalid parts of the input
    Invalid,
}

#[derive(Debug, Copy, Clone)]
pub struct Colorscheme {
    emphasis: &'static str,
    literal: &'static str,
    metavar: &'static str,
    header: &'static str,
    valid: &'static str,
    invalid: &'static str,
}

impl Colorscheme {
    pub const DULL: Self = Self {
        emphasis: "\x1b[4m\x1b1m", // underline + bold
        literal: "\x1b4m",         // bold
        metavar: "\x1b[1m",        // underline
        header: "\x1b[4m\x1b1m",   // underline + bold
        invalid: "\x1b[31m",       // red
        valid: "\x1b[32m",         // green
    };
    pub const BRIGHT: Self = Self {
        emphasis: "\x1b[1m\x1b[33m", // bold yellow
        literal: "\x1b[1m\x1b[32m",  // bold green
        metavar: "\x1b[1m\x1b[34m",  // bold blue
        header: "\x1b[1m\x1b[36m",   // bold
        invalid: "\x1b[31m",         // red
        valid: "\x1b[32m",           // green
    };
}

#[derive(Debug)]
struct ConsoleWriter {
    scheme: &'static Colorscheme,
    mono: bool,
    /// Current cursor horizontal position
    cursor: usize,
    /// Can change dynamically
    lpad: usize,
    /// Fixed for all the items
    tabstop: usize,
    output: String,
    pending_paragraph: bool,
    pending_space: bool,
    /// Are we before
    after_tab: bool,
    nobreak: bool,
}

impl ConsoleWriter {
    fn new(scheme: Option<&'static Colorscheme>, tabstop: usize) -> Self {
        Self {
            scheme: scheme.unwrap_or(&Colorscheme::DULL),
            mono: scheme.is_none(),
            cursor: 0,
            output: String::new(),
            lpad: 0,
            tabstop,
            pending_paragraph: false,
            pending_space: false,
            after_tab: false,
            nobreak: false,
        }
    }

    fn write_section(&mut self, section: Section) {
        if section.items.is_empty() {
            return;
        }
        use std::fmt::Write as _;
        const H: &str = Style::Header.ansi();
        const T: &str = Style::Text.ansi();
        _ = write!(&mut self.output, "{H}{}{T}", section.header);
        self.newline();
        if let Some(descr) = section.descr {
            self.write_text(descr);
            self.newline();
        }

        for item in section.items {
            self.write_item(item);
        }
    }
    fn write_item(&mut self, item: HelpItem) {
        use std::fmt::Write as _;
        if self.pending_paragraph {
            self.pending_paragraph = false;
            self.output.push('\n');
        }
        match item {
            HelpItem::Named { name, meta, help } => {
                const L: &str = Style::Literal.ansi();
                const T: &str = Style::Text.ansi();
                const M: &str = Style::Metavar.ansi();
                _ = match name {
                    ShortLong::Short(s) => {
                        self.cursor += 6;
                        write!(&mut self.output, "    {L}-{s}{T}")
                    }
                    ShortLong::Long(l) => {
                        self.cursor += 6 + 2 + 2 + width(l);
                        write!(&mut self.output, "        {L}--{l}{T}")
                    }
                    ShortLong::Both(s, l) => {
                        self.cursor += 6 + 2 + 2 + width(l);
                        write!(&mut self.output, "    {L}-{s}{T}, {L}--{l}{T}")
                    }
                };

                if let Some(meta) = meta {
                    self.cursor += 1 + meta.width();
                    _ = write!(&mut self.output, "={M}{meta}{T}");
                }
                if let Some(help) = help {
                    self.nobreak = true;
                    self.tabstop();
                    self.write_text(help);
                }
            }
            HelpItem::Pos { meta, help } => todo!(),
            HelpItem::Cmd { name, help } => todo!(),
            HelpItem::Text {
                text,
                lpad,
                tabstop,
            } => {
                self.write_text(&text);
                // todo!("{text:?} {lpad:?} {tabstop:?}")
            }
            HelpItem::Header { text } => {
                self.pending_paragraph = !self.output.is_empty();
                self.output.push_str(Style::Header.ansi());
                self.write_text(text);
                self.output.push_str(Style::Text.ansi());
            }
        }
        self.after_tab = false;
        self.cursor = 0;
        self.output.push('\n');
        self.pending_space = false;
    }

    fn newline(&mut self) {
        self.pending_space = false;
        self.cursor = 0;
        self.pending_space = false;
        self.output.push('\n');
    }

    fn tabstop(&mut self) {
        if self.after_tab {
            return;
        }
        self.after_tab = true;
        if let Some(diff) = self.tabstop.checked_sub(self.cursor) {
            self.output.push_str(&BLANK[..diff]);
            self.pending_space = false;
            self.cursor = self.tabstop;
        } else {
            self.pending_space = true;
            self.cursor += 1;
            self.output.push(' ');
        }
        self.nobreak = true;
    }

    fn write_text(&mut self, text: &str) {
        for chunk in split(text, self.mono) {
            match chunk {
                Chunk::Word { width, text } => {
                    if self.pending_paragraph {
                        self.pending_paragraph = false;
                        self.output.push('\n');
                        self.cursor = 0;
                    }
                    if width + 1 + self.cursor > MAX_WIDTH && !self.nobreak {
                        let pad = if self.after_tab {
                            self.tabstop
                        } else {
                            self.lpad
                        };
                        self.output.push('\n');
                        self.output.push_str(&BLANK[..pad]);
                        self.cursor = pad;
                        self.pending_space = false;
                    }
                    if self.pending_space {
                        self.output.push(' ');
                        self.pending_space = false;
                    }
                    self.output.push_str(text);
                    self.pending_space = true;
                    self.cursor += width;
                    self.nobreak = false;
                }
                Chunk::Tab => self.tabstop(),
                Chunk::LineBreak => {
                    self.cursor = 0;
                    self.output.push('\n');
                    self.pending_space = false;
                }
                Chunk::Paragraph => {
                    self.cursor = 0;
                    self.output.push('\n');
                    self.pending_paragraph = true;
                }
            }
        }
    }
    fn done(&self) -> String {
        apply_style(&self.output, self.scheme, self.mono)
    }
}

/// Apply final style to the rendering
pub fn apply_style(unstyled: &str, scheme: &Colorscheme, mono: bool) -> String {
    let mut output = Vec::with_capacity(unstyled.len() * 2);
    // TODO - perform postprocessing to insert ` or apply color scheme where appropriate

    #[derive(Copy, Clone)]
    enum Goal {
        Esc,
        Bracket,
        Digit,
        M,
    }
    let mut goal = Goal::Esc;
    let mut cur_style = b' ';
    let mut tick = false;
    let mut style = "";
    for c in unstyled.as_bytes().iter() {
        goal = match (*c, goal) {
            (b'\x1b', Goal::Esc) => Goal::Bracket,
            (b'[', Goal::Bracket) => Goal::Digit,
            (d, Goal::Digit) => {
                style = match d {
                    Style::TEXT => "\x1b[0m",
                    Style::EMPHASIS => scheme.emphasis,
                    Style::LITERAL => scheme.literal,
                    Style::METAVAR => scheme.metavar,
                    Style::HEADER => scheme.header,
                    Style::VALID => scheme.valid,
                    Style::INVALID => scheme.invalid,
                    _ => {
                        goal = Goal::Esc;
                        continue;
                    }
                };

                // if mono then on those transitions insert `
                // - text -> valid/invalid
                // - valid/invalid -> text
                tick = cur_style == Style::VALID
                    || d == Style::VALID
                    || cur_style == Style::INVALID
                    || d == Style::INVALID;

                cur_style = d;
                Goal::M
            }
            (b'm', Goal::M) => {
                if mono {
                    if tick {
                        output.push(b'`');
                    }
                } else {
                    output.extend(style.as_bytes());
                }
                Goal::Esc
            }
            (d, _) => {
                output.push(d);
                Goal::Esc
            }
        }
    }
    String::from_utf8(output).expect("Should be valid by construction")
}

#[inline(never)]
fn width(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_pair_of_headers() {
        let mut w = ConsoleWriter::new(None, 60);
        w.write_item(HelpItem::Header { text: "Hello" });
        w.write_item(HelpItem::Header { text: "Cat news" });
        let expected = "Hello\n\nCat news\n";
        assert_eq!(w.done(), expected);

        let mut w = ConsoleWriter::new(Some(&Colorscheme::DULL), 60);
        w.write_item(HelpItem::Header { text: "Hello" });
        w.write_item(HelpItem::Header { text: "Cat news" });
        let expected = "\u{1b}[4m\u{1b}1mHello\u{1b}[0m\n\u{1b}[4m\u{1b}1m\nCat news\u{1b}[0m\n";
        assert_eq!(w.done(), expected);
    }

    #[test]
    fn named_items() {
        let mut w = ConsoleWriter::new(None, 20);
        let help = Some(
            "Animal's name to use this time, and a long long help to use \
        long enough so it can't fit all on a single line and must be wrapped \
        into several lines. Probably even more than several lines - I want a \
        bunch of them. Will use this twice, with different argument name?",
        );

        w.write_item(HelpItem::Named {
            name: ShortLong::Both('c', "cat"),
            meta: Some(Metavar("NAME")),
            help,
        });

        w.write_item(HelpItem::Named {
            name: ShortLong::Both('k', "ket"),
            meta: Some(Metavar("Ket")),
            help,
        });

        w.write_item(HelpItem::Named {
            name: ShortLong::Long("quetzalcoatl-the-feathered-serpent"),
            meta: None,
            help: help,
        });
        let expected = r#"    -c, --cat=NAME  Animal's name to use this time, and a long long help to use long enough so it can't fit all on a
                    single line and must be wrapped into several lines. Probably even more than several lines - I
                    want a bunch of them. Will use this twice, with different argument name?
    -k, --ket=<Ket> Animal's name to use this time, and a long long help to use long enough so it can't fit all on a
                    single line and must be wrapped into several lines. Probably even more than several lines - I
                    want a bunch of them. Will use this twice, with different argument name?
        --quetzalcoatl-the-feathered-serpent  Animal's name to use this time, and a long long help to use long
                    enough so it can't fit all on a single line and must be wrapped into several lines. Probably even
                    more than several lines - I want a bunch of them. Will use this twice, with different argument
                    name?
"#;
        let r = w.done();
        println!("{r}");
        assert_eq!(w.done(), expected);
    }
}
