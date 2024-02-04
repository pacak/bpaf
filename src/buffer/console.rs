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

use super::{
    splitter::{split, Chunk},
    Block, Doc, Skip, Token,
};

#[cfg(feature = "color")]
use super::Style;

const MAX_TAB: usize = 24;
pub(crate) const MAX_WIDTH: usize = 100;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Default to dull color if colors are enabled,
#[allow(dead_code)] // not fully used in without colors
pub(crate) enum Color {
    Monochrome,
    #[cfg(feature = "color")]
    Dull,
    #[cfg(feature = "color")]
    Bright,
}

impl Default for Color {
    fn default() -> Self {
        #![allow(clippy::let_and_return)]
        #![allow(unused_mut)]
        #![allow(unused_assignments)]
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
    pub(crate) fn push_str(self, style: Style, res: &mut String, item: &str) {
        use owo_colors::OwoColorize;
        use std::fmt::Write;
        match self {
            Color::Monochrome => {
                res.push_str(item);
                Ok(())
            }
            Color::Dull => match style {
                Style::Text => {
                    res.push_str(item);
                    Ok(())
                }
                Style::Emphasis => write!(res, "{}", item.underline().bold()),
                Style::Literal => write!(res, "{}", item.bold()),
                Style::Metavar => write!(res, "{}", item.underline()),
                Style::Invalid => write!(res, "{}", item.bold().red()),
            },
            Color::Bright => match style {
                Style::Text => {
                    res.push_str(item);
                    Ok(())
                }
                Style::Emphasis => write!(res, "{}", item.yellow().bold()),
                Style::Literal => write!(res, "{}", item.green().bold()),
                Style::Metavar => write!(res, "{}", item.blue().bold()),
                Style::Invalid => write!(res, "{}", item.red().bold()),
            },
        }
        .unwrap();
    }
}

const PADDING: &str = "                                                  ";

impl Doc {
    /// Render a monochrome version of the document
    ///
    /// `full` indicates if full message should be rendered, this makes
    /// difference for rendered help message, otherwise you can pass `true`.
    #[must_use]
    pub fn monochrome(&self, full: bool) -> String {
        self.render_console(full, Color::Monochrome, MAX_WIDTH)
    }

    #[allow(clippy::too_many_lines)] // it's a big ass match statement
    pub(crate) fn render_console(&self, full: bool, color: Color, max_width: usize) -> String {
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
        let mut skip = Skip::default();
        let mut char_pos = 0;

        let mut margins: Vec<usize> = Vec::new();

        // a single new line, unless one exists
        let mut pending_newline = false;
        // a double newline, unless one exists
        let mut pending_blank_line = false;

        let mut pending_margin = false;

        for token in self.tokens.iter().copied() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;

                    if skip.enabled() {
                        continue;
                    }

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
                                    if char_pos + s.len() > max_width {
                                        char_pos = 0;
                                        res.truncate(res.trim_end().len());
                                        res.push('\n');
                                        if s == " " {
                                            continue;
                                        }
                                    }
                                }

                                let mut pushed = 0;
                                if let Some(missing) = margin.checked_sub(char_pos) {
                                    res.push_str(&PADDING[..missing]);
                                    char_pos = margin;
                                    pushed = missing;
                                }
                                if pending_margin && char_pos >= MAX_TAB + 4 && pushed < 2 {
                                    let missing = 2 - pushed;
                                    res.push_str(&PADDING[..missing]);
                                    char_pos += missing;
                                }

                                pending_newline = false;
                                pending_blank_line = false;
                                pending_margin = false;

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
                                    skip.enable();
                                    break;
                                }
                            }
                            Chunk::LineBreak => {
                                res.push('\n');
                                char_pos = 0;
                            }
                        }
                    }
                }
                Token::BlockStart(block) => {
                    #[cfg(test)]
                    stack.push(block);
                    let margin = margins.last().copied().unwrap_or(0usize);

                    match block {
                        Block::Header | Block::Section2 => {
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
                            pending_margin = true;
                        }
                        Block::InlineBlock => {
                            skip.push();
                        }
                        Block::Block => {
                            margins.push(margin);
                        }
                        Block::DefinitionList | Block::Meta | Block::Mono => {}
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
                        Block::ItemBody => {
                            pending_margin = false;
                        }
                        Block::Header
                        | Block::Section2
                        | Block::Section3
                        | Block::ItemTerm
                        | Block::DefinitionList
                        | Block::Meta
                        | Block::Mono => {}
                        Block::InlineBlock => {
                            skip.pop();
                        }
                        Block::Block => {
                            pending_blank_line = true;
                        }
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
