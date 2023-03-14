//! String builder, renders a string assembled from styled blocks
//!
//!
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Style {
    /// Plain text, no extra decorations
    Text,
    Section,
    Label,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Token {
    Text { bytes: usize, style: Style },
    LineBreak,
    TabStop,
    Margin(usize),
}
#[derive(Debug, Clone, Default)]
pub(crate) struct Buffer {
    /// string info saved here
    payload: String,
    /// string meta info tokens
    tokens: Vec<Token>,
    /// help listing separates keys from help info
    /// by a single two space wide gap arranging them two columns
    /// tab stop shows right most position of the second column start seen so far
    tabstop: usize,

    /// current char and margin are used to calculate tabstop
    current_margin: usize,
    current_char: usize,
}

impl Buffer {
    pub(crate) fn tabstop(&mut self) {
        self.tabstop = MAX_TAB.min(self.tabstop.max(self.current_char));
        self.tokens.push(Token::TabStop);
    }

    pub(crate) fn margin(&mut self, margin: usize) {
        self.current_margin = margin;
        self.tokens.push(Token::Margin(margin));
    }

    pub(crate) fn newline(&mut self) {
        self.current_char = 0;
        self.tokens.push(Token::LineBreak);
    }

    #[inline(never)]
    pub(crate) fn write_str(&mut self, input: &str, style: Style) {
        if self.current_char == 0 {
            self.current_char = self.current_margin;
        }

        let bytes = input.len();
        let chars = input.chars().count();

        self.current_char += chars;
        self.payload.push_str(input);
        match self.tokens.last_mut() {
            Some(Token::Text {
                bytes: pb,
                style: ps,
            }) if *ps == style => {
                *pb += bytes;
            }
            _ => {
                self.tokens.push(Token::Text { bytes, style });
            }
        }
    }

    #[inline(never)]
    pub(crate) fn write_char(&mut self, c: char, style: Style) {
        self.write_str(c.encode_utf8(&mut [0; 4]), style);
    }

    pub(crate) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            tokens: self.tokens.len(),
            payload: self.payload.len(),
        }
    }

    pub(crate) fn rollback(&mut self, checkpoint: Checkpoint) {
        self.tokens.truncate(checkpoint.tokens);
        self.payload.truncate(checkpoint.payload);
    }
    pub(crate) fn content_since(&self, checkpoint: Checkpoint) -> &str {
        &self.payload[checkpoint.payload..]
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Checkpoint {
    tokens: usize,
    payload: usize,
}

const MAX_TAB: usize = 24;
const MAX_WIDTH: usize = 100;

fn padding(f: &mut std::fmt::Formatter<'_>, width: usize) {
    write!(f, "{:width$}", "", width = width).unwrap();
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // byte offset to a start of not consumed portion of a string
        let mut byte_offset = 0;
        // character offset frmo the beginning of the line
        let mut line_offset = 0;
        // current margin value
        let mut margin = 0;
        // are we to the right of the tabstop?
        let mut after_tabstop = false;
        let mut immediate_tabstop = false;

        for token in &self.tokens {
            match *token {
                Token::Text { bytes, style } => {
                    // no matter what text should stay to the right of this position
                    let min_offset = if after_tabstop {
                        std::cmp::max(margin, self.tabstop + 2)
                    } else {
                        margin
                    };

                    if immediate_tabstop {
                        immediate_tabstop = false;
                        line_offset += 2;
                        padding(f, 2);
                    }

                    // split a string by words, lay them out between min_offset and MAX_WIDTH
                    for (ix, word) in (&self.payload[byte_offset..byte_offset + bytes])
                        .split(char::is_whitespace)
                        .enumerate()
                    {
                        let chars = word.chars().count();

                        // overflow?
                        if line_offset + chars > MAX_WIDTH {
                            writeln!(f)?;
                            line_offset = 0;
                        } else if ix != 0 {
                            padding(f, 1);
                            line_offset += 1;
                        }

                        if min_offset > line_offset {
                            padding(f, min_offset - line_offset);
                            line_offset = min_offset;
                        }

                        match style {
                            Style::Text => write!(f, "{}", word),
                            Style::Section => w_section!(f, word),
                            Style::Label => write!(f, "{}", w_flag!(word)),
                        }?;
                        line_offset += chars;
                    }
                    byte_offset += bytes;
                }
                Token::LineBreak => {
                    line_offset = 0;
                    writeln!(f)?;
                    after_tabstop = false;
                    immediate_tabstop = false;
                }
                Token::TabStop => {
                    after_tabstop = true;
                    immediate_tabstop = true;
                }
                Token::Margin(new_margin) => {
                    margin = new_margin;
                }
            }
        }
        Ok(())
    }
}

#[test]
fn tabstop_works() {
    // tabstop followed by newline
    let mut m = Buffer::default();
    m.write_str("aa", Style::Text);
    m.tabstop();
    m.newline();
    m.write_str("b", Style::Text);
    m.tabstop();
    m.write_str("c", Style::Text);
    m.newline();
    assert_eq!(m.to_string(), "aa\nb   c\n");

    // plain, narrow first
    let mut m = Buffer::default();
    m.write_str("1", Style::Text);
    m.tabstop();
    m.write_str("22", Style::Text);
    m.newline();
    m.write_str("33", Style::Text);
    m.tabstop();
    m.write_str("4", Style::Text);
    m.newline();
    assert_eq!(m.to_string(), "1   22\n33  4\n");

    // plain, wide first
    let mut m = Buffer::default();
    m.write_str("aa", Style::Text);
    m.tabstop();
    m.write_str("b", Style::Text);
    m.newline();
    m.write_str("c", Style::Text);
    m.tabstop();
    m.write_str("dd", Style::Text);
    m.newline();
    assert_eq!(m.to_string(), "aa  b\nc   dd\n");

    // two different styles first
    let mut m = Buffer::default();
    m.write_str("a", Style::Text);
    m.write_str("b", Style::Label);
    m.tabstop();
    m.write_str("c", Style::Label);
    m.newline();
    m.write_str("d", Style::Text);
    m.tabstop();
    m.write_str("e", Style::Label);
    m.newline();
    assert_eq!(m.to_string(), "ab  c\nd   e\n");

    // two different styles with margin first
    let mut m = Buffer::default();
    m.margin(2);
    m.write_str("a", Style::Text);
    m.write_str("b", Style::Label);
    m.tabstop();
    m.write_str("c", Style::Label);
    m.margin(0);
    m.newline();
    m.write_str("d", Style::Text);
    m.tabstop();
    m.write_str("e", Style::Label);
    m.newline();
    assert_eq!(m.to_string(), "  ab  c\nd     e\n");
}

#[test]
fn margin_works() {
    let mut m = Buffer::default();
    m.margin(2);
    m.write_str("a", Style::Text);
    m.newline();
    m.write_str("b", Style::Text);
    m.newline();
    m.write_str("c", Style::Text);
    m.newline();
    assert_eq!(m.to_string(), "  a\n  b\n  c\n");
}

#[test]
fn linewrap_works() {
    let mut m = Buffer::default();
    m.write_str("--hello", Style::Label);
    m.tabstop();
    for _ in 0..15 {
        m.write_str("word and word ", Style::Text)
    }
    m.write_str("and word", Style::Text);
    m.newline();

    let expected = "\
--hello  word and word word and word word and word word and word word and word word and word word and
         word word and word word and word word and word word and word word and word word and word
         word and word word and word and word
";

    assert_eq!(m.to_string(), expected);
}

#[test]
fn very_long_tabstop() {
    let mut m = Buffer::default();
    m.write_str(
        "--this-is-a-very-long-option <DON'T DO THIS AT HOME>",
        Style::Label,
    );
    m.tabstop();
    for _ in 0..15 {
        m.write_str("word and word ", Style::Text)
    }
    m.write_str("and word", Style::Text);
    m.newline();

    let expected = "\
--this-is-a-very-long-option <DON'T DO THIS AT HOME>  word and word word and word word and word word
                          and word word and word word and word word and word word and word word and
                          word word and word word and word word and word word and word word and word
                          word and word and word
";

    assert_eq!(m.to_string(), expected);
}
