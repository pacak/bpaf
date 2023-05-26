//! String builder, renders a string assembled from styled blocks

// help needs to support following features:
// - linebreak - insert linebreak whenever
// - newline - start text on a new line, don't start a new line if not already at one
// - margin - start text at some offset at a new line
// - tabstop - all tabstops are aligned within a section
// - section title - "usage", "available options", "available positionals", etc. starts a new
//         section - resets tabstops
// - literal text user needs to type - flags, command names, etc.
// - metavar - meta placehoder user needs to write something
// - subsection title - two spaces + text, used for adjacent groups

// help might want to render it:
// - monochrome - default mode
// - bright/dull/custom colors
// - export to markdown and groff
//
// monochrome and colors are rendered with different widths so tabstops are out of buffer rendering

// text formatting rules:
//
// want to be able to produce both brief and full versions of the documentation, it only makes
// sense to look for that in the plain text...
// - "\n " => hard line break, inserted always
// - "\n\n" => paragraphs are separated by this, only the first one in inserted unless in "full" mode
// // - "\n" => converted to spaces, text flows to the current margin value
//
// tabstops are aligned the same position within a section, tabstop sets a temporary margin for all
// the soft linebreaks, tabstop
//
// margin sets the minimal offset for any new text and retained until new margin is set:
// "hello" [margin 8] "world" is rendered as "hello   world"

struct Splitter<'a> {
    input: &'a str,
}

/// Split payload into chunks annotated with character width and containing no newlines according
/// to text formatting rules
fn split(input: &str) -> Splitter {
    Splitter { input }
}

#[cfg_attr(test, derive(Debug, Clone, Copy, Eq, PartialEq))]
enum Chunk<'a> {
    Raw(&'a str, usize),
    Paragraph,
    LineBreak,
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            None
        } else if let Some(tail) = self.input.strip_prefix("\n\n") {
            self.input = tail;
            Some(Chunk::Paragraph)
        } else if let Some(tail) = self.input.strip_prefix("\n ") {
            self.input = tail;
            Some(Chunk::LineBreak)
        } else if let Some(tail) = self.input.strip_prefix('\n') {
            self.input = tail;
            Some(Chunk::Raw(" ", 1))
        } else if let Some(tail) = self.input.strip_prefix(' ') {
            self.input = tail;
            Some(Chunk::Raw(" ", 1))
        } else {
            let mut char_ix = 0;

            // there's iterator position but it won't give me character length of the rest of the input
            for (byte_ix, chr) in self.input.char_indices() {
                if chr == '\n' || chr == ' ' {
                    let head = &self.input[..byte_ix];
                    let tail = &self.input[byte_ix..];
                    self.input = tail;
                    return Some(Chunk::Raw(head, char_ix));
                }
                char_ix += 1;
            }
            let head = self.input;
            self.input = "";
            Some(Chunk::Raw(head, char_ix))
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Style {
    /// Plain text, no decorations
    Text,

    /// Section title
    Title,

    /// Something user needs to type literally - command names, etc
    Literal,

    /// Metavavar placeholder - something user needs to replace with own input
    Metavar,

    /// Invalid input given by user - used to display errors
    Invalid,

    /// Something less important, usually rendered in a more dull color
    Muted,
}

// for help structure is
//
// <block>header</block>
// <block>Usage: basic --help</block>
// <block>
//   <section2>Available options</section2>
//   <dl>
//     <block>
//       <section3>pick one of those</section3>
//       <td>--intel</td>
//       <dd>dump in intel format</dd>
//     </block>
//     <block>
//       <td>--release</td>
//       <dd>install in release mode</dd>
//     </block>
//     <block>
//       <section3>built in</section3>
//       <dt>-h, --help</td>
//       <dd>prints help</dd>
//     </block>
//   </dl>
// </block>
// <block>footer</block>

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Block {
    // 0 margin
    /// level 1 section header, block for separate command inside manpage, not used in --help
    Section1,

    // 0 margin
    /// level 2 section header, "Available options" in --help, etc
    /// in plain text styled with
    Section2,

    // 2 margin
    /// level 3 section header, "group_help" header, etc.
    Section3,

    // inline, 4 margin, no nesting
    /// -h, --help
    ItemTerm,

    // widest term up below 20 margin margin plus two, but at least 4.
    /// print usage information, but also items inside Numbered/Unnumbered lists
    ItemBody,

    // 0 margin
    /// Definition list,
    DefList,

    /// block of text, blocks are separated by a blank line in man or help
    /// can contain headers or other items inside
    Block,

    // 2 margin
    /// Preformatted text
    Pre,

    // inline
    /// displayed with `` in monochrome or not when rendered with colors.
    /// In markdown this becomes a link to a term if one is defined
    TermRef,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Token {
    Text { bytes: usize, style: Style },
    BlockStart(Block),
    BlockEnd(Block),
    /*
    /// Term is a command name, positional name or flag with metavar in option lists
    ///
    /// Empty term is used to add padding for things like default value
    /// [TermStart]--count=ITEMS[TermEnd][Text "Number items to process"]
    /// [TermStart][TermEnd][Text "default value is 10"]
    ///
    /// buffer rendering assumes that there's one term in a line and no characters before it starts
    /// terms are indented by 4
    TermStart,
    TermStop,
    /// Section means indented to 0 chars, usually "available options", "available positionals", etc
    /// but also usage or header/footer. Sections are separated from each other by an empty line
    SectionStart,
    SectionStop,
    /// Subsection means some lines indented by 2 - group header or expanded anywhere
    ///
    SubsectionStart,
    SubsectionStop,
    /// Linebreak also ends current term in a list
    LineBreak,*/
}

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    /// string info saved here
    payload: String,

    /// string meta info tokens
    tokens: Vec<Token>,
}

impl Buffer {
    pub(crate) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn buffer(&mut self, buf: &Buffer) {
        self.tokens.extend(&buf.tokens);
        self.payload.push_str(&buf.payload);
    }

    pub(crate) fn first_line(&self) -> Option<Buffer> {
        if self.tokens.is_empty() {
            return None;
        }

        let mut res = Buffer::default();
        let mut cur = 0;

        for &t in &self.tokens {
            match t {
                Token::Text { bytes, style: _ } => {
                    // TODO -
                    res.tokens.push(t);
                    res.payload.push_str(&self.payload[cur..cur + bytes]);
                    cur += bytes;
                }
                _ => break,
            }
        }
        Some(res)
    }

    pub(crate) fn to_completion(&self) -> Option<String> {
        Some(self.first_line()?.payload)
    }
}

impl From<&str> for Buffer {
    fn from(value: &str) -> Self {
        let mut buf = Buffer::default();
        buf.write_str(value, Style::Text);
        buf
    }
}

impl Buffer {
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.payload.clear();
        self.tokens.clear();
    }

    #[inline(never)]
    pub(crate) fn token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub(crate) fn write<T>(&mut self, input: T, style: Style)
    where
        T: std::fmt::Display,
    {
        use std::fmt::Write;
        let old_len = self.payload.len();
        let _ = write!(self.payload, "{}", input);
        self.set_style(self.payload.len() - old_len, style);
    }

    #[inline(never)]
    fn set_style(&mut self, len: usize, style: Style) {
        // buffer chunks are unified with previous chunks of the same type
        // [metavar]"foo" + [metavar]"bar" => [metavar]"foobar"
        match self.tokens.last_mut() {
            Some(Token::Text {
                bytes: prev_bytes,
                style: prev_style,
            }) if *prev_style == style => {
                *prev_bytes += len;
            }
            _ => {
                self.tokens.push(Token::Text { bytes: len, style });
            }
        }
    }

    #[inline(never)]
    pub(crate) fn write_str(&mut self, input: &str, style: Style) {
        self.payload.push_str(input);
        self.set_style(input.len(), style);
    }

    #[inline(never)]
    pub(crate) fn write_char(&mut self, c: char, style: Style) {
        self.write_str(c.encode_utf8(&mut [0; 4]), style);
    }
}

const MAX_TAB: usize = 24;
const MAX_WIDTH: usize = 100;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Default to dull color if colors are enabled,
pub(crate) enum Color {
    Monochrome,
    #[cfg(feature = "color")]
    Dull,
    #[cfg(feature = "color")]
    Bright,
}

impl Default for Color {
    fn default() -> Self {
        #[allow(clippy::let_and_return)]
        #[allow(unused_mut)]
        let mut res;
        #[cfg(not(feature = "color"))]
        {
            res = Color::Monochrome;
        }

        #[cfg(feature = "color")]
        {
            res = Color::Dull;
        }

        #[cfg(feature = "bright-color")]
        {
            res = Color::Bright;
        }

        #[cfg(feature = "dull-color")]
        {
            res = Color::Dull;
        }

        #[cfg(feature = "color")]
        {
            use supports_color::{on, Stream};
            if !(on(Stream::Stdout).is_some() && on(Stream::Stderr).is_some()) {
                res = Color::Monochrome;
            }
        }
        res
    }
}

#[cfg(feature = "color")]
impl Color {
    fn push_str(&self, style: Style, res: &mut String, item: &str) {
        use owo_colors::OwoColorize;
        use std::fmt::Write;
        match self {
            Color::Monochrome => Ok(res.push_str(item)),
            Color::Dull => match style {
                Style::Text => Ok(res.push_str(item)),
                Style::Title => write!(res, "{}", item.underline().bold()),
                Style::Literal => write!(res, "{}", item.bold()),
                Style::Metavar => write!(res, "{}", item.underline()),
                Style::Invalid => write!(res, "{}", item.bold()),
                Style::Muted => write!(res, "{}", item.dimmed()),
            },
            Color::Bright => match style {
                Style::Text => Ok(res.push_str(item)),
                Style::Title => write!(res, "{}", item.yellow().bold()),
                Style::Literal => write!(res, "{}", item.green().bold()),
                Style::Metavar => write!(res, "{}", item.blue().bold()),
                Style::Invalid => write!(res, "{}", item.red().bold()),
                Style::Muted => write!(res, "{}", item.dimmed()),
            },
        }
        .unwrap();
    }
}

const PADDING: &str = "                                                  ";

impl Buffer {
    pub(crate) fn monochrome(&self, full: bool) -> String {
        self.render_console(full, Color::Monochrome)
    }

    pub(crate) fn render_console(&self, full: bool, color: Color) -> String {
        let mut res = String::new();
        let mut tabstop = 0;
        let mut byte_pos = 0;
        {
            let mut current = 0;
            let mut in_term = false;
            // looking for widest term below MAX_TAB
            for token in self.tokens.iter().copied() {
                match token {
                    Token::Text { bytes, style: _ } => {
                        if in_term {
                            current += self.payload[byte_pos..byte_pos + bytes].chars().count();
                        }
                        byte_pos += bytes;
                    }
                    Token::BlockStart(Block::ItemTerm) => {
                        in_term = true;
                        current = 0;
                    }
                    Token::BlockEnd(Block::ItemTerm) => {
                        in_term = false;
                        if current > tabstop && current <= MAX_TAB {
                            tabstop = current;
                        }
                    }
                    _ => {}
                }
            }
            byte_pos = 0;
        }
        let tabstop = tabstop + 4;

        #[cfg(test)]
        let mut stack = Vec::new();

        let mut char_pos = 0;

        let mut margins: Vec<usize> = Vec::new();

        // a single new line, unless one exists
        let mut pending_newline = false;
        // a double newline, unless one exists
        let mut pending_blank_line = false;

        for token in self.tokens.iter().copied() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    for chunk in split(input) {
                        match chunk {
                            Chunk::Raw(s, w) => {
                                let margin = margins.last().copied().unwrap_or(0usize);
                                if !res.is_empty() {
                                    if (pending_newline || pending_blank_line)
                                        && !res.ends_with('\n')
                                    {
                                        char_pos = 0;
                                        res.push('\n');
                                    }
                                    if pending_blank_line && !res.ends_with("\n\n") {
                                        res.push('\n');
                                    }
                                    if char_pos > MAX_WIDTH {
                                        char_pos = 0;
                                        res.truncate(res.trim_end().len());
                                        res.push('\n');
                                        if s == " " {
                                            continue;
                                        }
                                    }
                                }

                                pending_newline = false;
                                pending_blank_line = false;

                                if let Some(missing) = margin.checked_sub(char_pos) {
                                    res.push_str(&PADDING[..missing]);
                                    char_pos = margin;
                                }
                                #[cfg(feature = "color")]
                                {
                                    color.push_str(style, &mut res, s);
                                }
                                #[cfg(not(feature = "color"))]
                                {
                                    let _ = style;
                                    let _ = color;
                                    res.push_str(s);
                                }
                                char_pos += w;
                            }
                            Chunk::Paragraph => {
                                res.push('\n');
                                char_pos = 0;
                                if !full {
                                    break;
                                }
                            }
                            Chunk::LineBreak => {
                                res.push('\n');
                                char_pos = 0;
                            }
                        }
                    }
                    byte_pos += bytes;
                }
                Token::BlockStart(block) => {
                    #[cfg(test)]
                    stack.push(block);
                    let margin = margins.last().copied().unwrap_or(0usize);

                    match block {
                        Block::Section1 | Block::Section2 => {
                            pending_newline = true;
                            margins.push(margin);
                        }
                        Block::Section3 => {
                            pending_newline = true;
                            margins.push(margin + 2);
                        }
                        Block::ItemTerm => {
                            pending_newline = true;
                            margins.push(margin + 4);
                        }
                        Block::ItemBody => {
                            margins.push(margin + tabstop + 2);
                        }
                        Block::DefList => todo!(),
                        Block::Block => {
                            margins.push(margin);
                        }
                        Block::Pre => todo!(),
                        Block::TermRef => {
                            if color == Color::Monochrome {
                                res.push('`');
                                char_pos += 1;
                            }
                        }
                    }
                }
                Token::BlockEnd(block) => {
                    #[cfg(test)]
                    assert_eq!(stack.pop(), Some(block));

                    margins.pop();
                    match block {
                        Block::Section1 => todo!(),
                        Block::Section2 => {}
                        Block::Section3 => {}
                        Block::ItemTerm => {}
                        Block::ItemBody => {}
                        Block::DefList => todo!(),
                        Block::Block => {
                            pending_blank_line = true;
                        }
                        Block::Pre => todo!(),
                        Block::TermRef => {
                            if color == Color::Monochrome {
                                res.push('`');
                                char_pos += 1;
                            }
                        }
                    }
                }
            }
        }
        if pending_newline || pending_blank_line {
            res.push('\n');
        }
        #[cfg(test)]
        assert_eq!(stack, &[]);
        res
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tabstop_works() {
        // tabstop followed by newline
        let mut m = Buffer::default();
        m.token(Token::TermStart);
        m.text("aa");
        m.token(Token::TermStop);
        m.token(Token::LineBreak);

        m.token(Token::TermStart);
        m.text("b");
        m.token(Token::TermStop);
        m.text("c");
        m.token(Token::LineBreak);
        assert_eq!(m.monochrome(true), "    aa\n    b   c\n");
        m.clear();

        // plain, narrow first
        m.token(Token::TermStart);
        m.text("1");
        m.token(Token::TermStop);
        m.text("22");
        m.token(Token::LineBreak);

        m.token(Token::TermStart);
        m.text("33");
        m.token(Token::TermStop);
        m.text("4");
        m.token(Token::LineBreak);
        assert_eq!(m.monochrome(true), "    1   22\n    33  4\n");
        m.clear();

        // plain, wide first
        m.token(Token::TermStart);
        m.text("aa");
        m.token(Token::TermStop);

        m.text("b");
        m.token(Token::LineBreak);

        m.token(Token::TermStart);
        m.text("c");
        m.token(Token::TermStop);

        m.text("dd");
        m.token(Token::LineBreak);
        assert_eq!(m.monochrome(true), "    aa  b\n    c   dd\n");
        m.clear();

        // two different styles first
        m.token(Token::TermStart);
        m.text("a");
        m.literal("b");
        m.token(Token::TermStop);

        m.literal("c");
        m.token(Token::LineBreak);
        m.token(Token::TermStart);
        m.text("d");
        m.token(Token::TermStop);

        m.literal("e");
        m.token(Token::LineBreak);
        assert_eq!(m.monochrome(true), "    ab  c\n    d   e\n");
    }

    #[test]
    fn linewrap_works() {
        let mut m = Buffer::default();
        m.token(Token::TermStart);
        m.write_str("--hello", Style::Literal);
        m.token(Token::TermStop);
        for i in 0..25 {
            m.write_str(&format!("and word{i} "), Style::Text)
        }
        m.write_str("and last word", Style::Text);
        m.token(Token::LineBreak);

        let expected =
"    --hello  and word0 and word1 and word2 and word3 and word4 and word5 and word6 and word7 and word8
             and word9 and word10 and word11 and word12 and word13 and word14 and word15 and word16 and
             word17 and word18 and word19 and word20 and word21 and word22 and word23 and word24 and last
             word
";

        assert_eq!(m.monochrome(true), expected);
    }

    #[test]
    fn very_long_tabstop() {
        let mut m = Buffer::default();
        m.token(Token::TermStart);
        m.write_str(
            "--this-is-a-very-long-option <DON'T DO THIS AT HOME>",
            Style::Literal,
        );
        m.token(Token::TermStop);
        for i in 0..15 {
            m.write_str(&format!("and word{i} "), Style::Text)
        }
        m.write_str("and last word", Style::Text);
        m.token(Token::LineBreak);

        let expected =
"    --this-is-a-very-long-option <DON'T DO THIS AT HOME>  and word0 and word1 and word2 and word3 and word4
      and word5 and word6 and word7 and word8 and word9 and word10 and word11 and word12 and word13 and
      word14 and last word
";

        assert_eq!(m.monochrome(true), expected);
    }

    #[test]
    fn line_breaking_rules() {
        let mut buffer = Buffer::default();
        buffer.write_str("hello ", Style::Text);
        assert_eq!(buffer.monochrome(true), "hello ");
        buffer.clear();

        buffer.write_str("hello\n world\n", Style::Text);
        assert_eq!(buffer.monochrome(true), "hello\nworld ");
        buffer.clear();

        buffer.write_str("hello\nworld", Style::Text);
        assert_eq!(buffer.monochrome(true), "hello world");
        buffer.clear();

        buffer.write_str("hello\nworld\n", Style::Text);
        assert_eq!(buffer.monochrome(true), "hello world ");
        buffer.clear();

        buffer.write_str("hello\n\nworld", Style::Text);
        assert_eq!(buffer.monochrome(false), "hello\n");
        buffer.clear();

        buffer.write_str("hello\n\nworld", Style::Text);
        assert_eq!(buffer.monochrome(true), "hello\nworld");
        buffer.clear();
    }

    #[test]
    fn splitter_works() {
        assert_eq!(
            split("hello ").collect::<Vec<_>>(),
            [Chunk::Raw("hello", 5), Chunk::Raw(" ", 1)]
        );

        assert_eq!(
            split("hello\nworld").collect::<Vec<_>>(),
            [
                Chunk::Raw("hello", 5),
                Chunk::Raw(" ", 1),
                Chunk::Raw("world", 5)
            ]
        );

        assert_eq!(
            split("hello\n world").collect::<Vec<_>>(),
            [
                Chunk::Raw("hello", 5),
                Chunk::HardLineBreak,
                Chunk::Raw("world", 5)
            ]
        );

        assert_eq!(
            split("hello\n\nworld").collect::<Vec<_>>(),
            [
                Chunk::Raw("hello", 5),
                Chunk::SoftLineBreak,
                Chunk::Raw("world", 5)
            ]
        );
    }
}*/
