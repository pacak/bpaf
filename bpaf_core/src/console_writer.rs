use crate::visitors::{
    ShortLong,
    help::{HelpItem, Section},
};

const BLANK: &'static str = "                                                                     ";

pub(crate) const MAX_WIDTH: usize = 100;
pub(crate) const MAX_TAB: usize = 24;

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

/// 1. consider one line at a time
/// 2. empty line = paragraph
/// 3. one leading space LineBreak
/// 4. four leading spaces - indented block
/// 5. first tab in a line - TabStop, otherwise - it's a space
/// 6. multiple adjacent spaces are chomped
/// 7. output contains no spaces

#[derive(Debug)]
struct LineSplit<'a> {
    cur_line: &'a str,
    mode: Mode,
    rest: &'a str,
    mono: bool,
}

#[derive(Debug)]
enum Mode {
    NextLine,
    Newline,
    TakeRest,
    Parse,
}

fn linesplit<'a>(input: &'a str, mono: bool) -> LineSplit<'a> {
    LineSplit {
        cur_line: "",
        mode: Mode::NextLine,
        rest: input,
        mono,
    }
}

impl<'a> Iterator for LineSplit<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &self.mode {
                Mode::NextLine => {
                    (self.cur_line, self.rest) = match self.rest.split_once('\n') {
                        Some(split) => split,
                        None if self.rest.is_empty() => return None,
                        None => (self.rest, ""),
                    };
                    self.mode = Mode::Newline;
                }
                Mode::Newline => {
                    if self.cur_line.starts_with("    ") {
                        self.mode = Mode::TakeRest;
                        return Some(Chunk::LineBreak);
                    } else if self.cur_line.is_empty() {
                        self.mode = Mode::NextLine;
                        return Some(Chunk::Paragraph);
                    } else if let Some(tail) = self.cur_line.strip_prefix(' ') {
                        self.mode = Mode::Parse;
                        self.cur_line = tail;
                        return Some(Chunk::LineBreak);
                    } else {
                        self.mode = Mode::Parse;
                    }
                }
                Mode::TakeRest => {
                    self.mode = Mode::NextLine;
                    return Some(word(self.cur_line, self.mono));
                }
                Mode::Parse => {
                    if let Some(rest) = self.cur_line.strip_prefix('\t') {
                        self.cur_line = rest;
                        return Some(Chunk::Tab);
                    } else if let Some(rest) = self.cur_line.strip_prefix(' ') {
                        self.cur_line = rest;
                    } else {
                        match self
                            .cur_line
                            .as_bytes()
                            .iter()
                            .position(|u| u.is_ascii_whitespace())
                        {
                            Some(mid) => {
                                let (this, rest) = self.cur_line.split_at(mid);
                                self.cur_line = rest;
                                return Some(word(this, self.mono));
                            }
                            None => {
                                self.mode = Mode::NextLine;
                                return Some(word(self.cur_line, self.mono));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Calculate width in visible characters, ignores `"\{1b}[Dm"` where D is a single digit
#[inline(never)]
fn word_width(text: &str, mono: bool) -> usize {
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
    width
}

fn word(text: &str, mono: bool) -> Chunk<'_> {
    Chunk::Word {
        width: word_width(text, mono),
        text,
    }
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

#[derive(Copy, Clone)]
pub struct Colorscheme {
    emphasis: &'static str,
    literal: &'static str,
    metavar: &'static str,
    header: &'static str,
    valid: &'static str,
    invalid: &'static str,
}
impl std::fmt::Debug for Colorscheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Colorscheme {..}").finish()
    }
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Default)]
pub(crate) enum Pending {
    #[default]
    Nothing,
    Space,
    Newline,
    Paragraph,
}

#[derive(Debug)]
pub(crate) struct ConsoleWriter {
    scheme: &'static Colorscheme,
    mono: bool,
    /// Current cursor horizontal position
    cursor: usize,
    /// Can change dynamically
    lpad: usize,
    /// Fixed for all the items
    tabstop: usize,
    output: String,
    pending: Pending,

    /// Are we before
    after_tab: bool,
    nobreak: bool,

    /// If any written text should contain everything after an empty line or just the brief version
    detailed: bool,
}

impl ConsoleWriter {
    pub(crate) fn new(
        scheme: Option<&'static Colorscheme>,
        tabstop: usize,
        detailed: bool,
    ) -> Self {
        Self {
            scheme: scheme.unwrap_or(&Colorscheme::DULL),
            mono: scheme.is_none(),
            cursor: 0,
            output: String::new(),
            lpad: 0,
            tabstop,
            pending: Pending::Nothing,
            after_tab: false,
            nobreak: false,
            detailed,
        }
    }

    pub(crate) fn write_section(&mut self, section: Section) {
        if section.items.is_empty() {
            return;
        }
        use std::fmt::Write as _;
        const H: &str = Style::Header.ansi();
        const T: &str = Style::Text.ansi();
        self.pending = Pending::Paragraph;
        self.handle_pending();
        _ = write!(&mut self.output, "{H}{}{T}", section.header);
        self.cursor = char_width(&section.header);
        self.pending = Pending::Newline;
        if let Some(descr) = section.descr {
            self.write_text(descr);
            self.pending = self.pending.max(Pending::Newline);
        }
        self.pending = Pending::Newline;

        let mut set = std::collections::HashSet::new();
        for item in section.items.iter() {
            if set.insert(item) {
                self.write_item(&item);
            }
        }
    }
    pub(crate) fn write_item(&mut self, item: &HelpItem) {
        use std::fmt::Write as _;
        const L: &str = Style::Literal.ansi();
        const T: &str = Style::Text.ansi();
        const M: &str = Style::Metavar.ansi();
        match item {
            HelpItem::Named { name, meta, help } => {
                self.pending = self.pending.max(Pending::Newline);
                self.handle_pending();
                _ = write!(&mut self.output, "{name:#}");
                self.cursor = 4 + name.width();

                if let Some(meta) = meta {
                    self.cursor += 1 + meta.width();
                    _ = write!(&mut self.output, "={M}{meta}{T}");
                }
                self.after_tab = false;
                if let Some(help) = help {
                    self.nobreak = true;
                    self.pending = self.tabstop();
                    self.write_text(help);
                }
            }
            HelpItem::Pos { meta, help } => {
                self.handle_pending();

                self.cursor += 4 + meta.width();
                _ = write!(&mut self.output, "    {M}{meta}{T}");
                if let Some(help) = help {
                    self.pending = self.tabstop();
                    self.write_text(help);
                }
            }
            HelpItem::Cmd { name, help } => {
                self.handle_pending();
                _ = match name.0 {
                    ShortLong::Short(s) => {
                        self.cursor += 5;
                        write!(&mut self.output, "    {L}{s}{T}")
                    }
                    ShortLong::Long(l) => {
                        self.cursor += 4 + char_width(l);
                        write!(&mut self.output, "    {L}{l}{T}")
                    }
                    ShortLong::Both(s, l) => {
                        self.cursor += 6 + char_width(l);
                        write!(&mut self.output, "    {L}{s}{T}, {L}{l}{T}")
                    }
                };

                if let Some(help) = help {
                    self.pending = self.tabstop();
                    self.write_text(help);
                }
            }
            HelpItem::Text {
                text,
                lpad,
                tabstop,
            } => {
                self.handle_pending();
                self.write_text(&text);
                // todo!("{text:?} {lpad:?} {tabstop:?}")
            }
            HelpItem::Header { text } => {
                self.pending = Pending::Paragraph;
                self.handle_pending();
                self.pending = Pending::Nothing;

                self.output.push_str(Style::Header.ansi());

                self.write_text(text);
                self.output.push_str(Style::Text.ansi());
                self.pending = Pending::Newline;
            }
        }
        self.after_tab = false;
        self.pending = Pending::Newline;
    }

    pub(crate) fn newline(&mut self) {
        self.pending = Pending::Newline;
    }

    pub(crate) fn paragraph(&mut self) {
        self.pending = Pending::Paragraph;
    }

    pub(crate) fn tabstop(&mut self) -> Pending {
        if self.after_tab {
            return Pending::Space;
        }
        self.after_tab = true;
        self.nobreak = true;
        if let Some(diff) = self.tabstop.checked_sub(self.cursor) {
            self.output.push_str(&BLANK[..diff]);
            self.cursor = self.tabstop;
            Pending::Nothing
        } else {
            self.cursor += 1;
            self.output.push(' ');
            Pending::Space
        }
    }

    fn handle_pending(&mut self) {
        match self.pending {
            Pending::Nothing => {}
            Pending::Space => {
                self.output.push(' ');
                self.cursor += 1;
            }
            Pending::Newline => {
                if self.cursor > 0 {
                    self.output.push('\n');
                    self.cursor = 0;
                }
            }
            Pending::Paragraph => {
                if !self.output.is_empty() {
                    self.output.push_str("\n\n");
                    self.cursor = 0;
                }
            }
        }
        self.pending = Pending::Nothing;
    }

    pub(crate) fn write_text(&mut self, mut text: &str) {
        if !self.detailed
            && let Some((prefix, _)) = text.split_once("\n\n")
        {
            text = prefix;
        }
        for chunk in linesplit(text, self.mono) {
            self.pending = match chunk {
                Chunk::Word { width, text } => {
                    if width + 1 + self.cursor > MAX_WIDTH
                        && !self.nobreak
                        && self.pending == Pending::Space
                    {
                        self.pending = Pending::Newline;
                    }
                    self.handle_pending();
                    if self.cursor == 0 {
                        let pad = if self.after_tab {
                            self.tabstop
                        } else {
                            self.lpad
                        };
                        self.cursor = pad;
                        self.output.push_str(&BLANK[..pad]);
                    }
                    self.output.push_str(text);
                    self.cursor += width;
                    self.nobreak = false;
                    Pending::Space
                }
                Chunk::Tab => self.tabstop(),
                Chunk::LineBreak => self.pending.max(Pending::Newline),
                Chunk::Paragraph => self.pending.max(Pending::Paragraph),
            }
        }
    }
    pub(crate) fn done(mut self) -> String {
        if self.pending >= Pending::Newline {
            self.output.push('\n');
        }
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
pub(crate) fn char_width(s: &str) -> usize {
    s.chars().count()
}
