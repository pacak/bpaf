pub(crate) const MAX_WIDTH: usize = 100;
pub(crate) const MAX_TAB: usize = 28;

/// Apply color scheme, split long lines and layout tab column
/// - lines must have at most one tab
/// - lines that start with 2 or more spaces are not wrapped
/// - all line breaks are preserved
pub(crate) fn apply_style(input: &str, tab: usize, scheme: Option<&Colorscheme>) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let mut cursor = 0;
        match line.split_once('\t') {
            Some((key, help)) => {
                write_styled(0, usize::MAX, key, scheme, &mut cursor, &mut out, false);
                if cursor > tab && !help.is_empty() {
                    out.push_str("  ");
                }
                for (ix, word) in help.split_ascii_whitespace().enumerate() {
                    write_styled(tab, MAX_WIDTH, word, scheme, &mut cursor, &mut out, ix > 0);
                }
            }

            None => {
                if line.starts_with("  ") {
                    write_styled(0, usize::MAX, line, scheme, &mut cursor, &mut out, false);
                } else {
                    for (ix, word) in line.split_ascii_whitespace().enumerate() {
                        write_styled(0, MAX_WIDTH, word, scheme, &mut cursor, &mut out, ix > 0);
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}

/// Apply final style to the rendering, input MUST NOT contain ANSI sequences
fn write_styled(
    start: usize,
    end: usize,
    from: &str,
    scheme: Option<&Colorscheme>,
    cursor: &mut usize,
    output: &mut String,
    mut pending_sep: bool,
) {
    use crate::miniansi::Frag;

    if let Some(missing) = start.checked_sub(*cursor) {
        output.extend(std::iter::repeat_n(' ', missing));
        *cursor = start;
    }

    for item in crate::miniansi::split(from) {
        match item {
            Frag::Str(s) => {
                *cursor += char_width(s) + pending_sep as usize;
                if *cursor > end {
                    output.push('\n');
                    *cursor = start;
                    if start > 0 {
                        output.extend(std::iter::repeat_n(' ', start));
                    }
                } else if pending_sep {
                    output.push(' ');
                }
                pending_sep = false;
                output.push_str(s);
            }

            Frag::Code(code) => {
                if let Some(scheme) = scheme {
                    output.push_str(scheme[code])
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Colorscheme {
    pub text: &'static str,
    pub emphasis: &'static str,
    pub literal: &'static str,
    pub metavar: &'static str,
    pub header: &'static str,
    pub valid: &'static str,
    pub invalid: &'static str,
    // TODO - custom1, custom2?
}

impl std::ops::Index<Style> for Colorscheme {
    type Output = &'static str;

    fn index(&self, index: Style) -> &Self::Output {
        match index {
            Style::Text => &self.text,
            Style::Emphasis => &self.emphasis,
            Style::Literal => &self.literal,
            Style::Metavar => &self.metavar,
            Style::Header => &self.header,
            Style::Valid => &self.valid,
            Style::Invalid => &self.invalid,
        }
    }
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
        text: "\x1b[0m]",          // reset
    };
    pub const BRIGHT: Self = Self {
        emphasis: "\x1b[1m\x1b[33m", // bold yellow
        literal: "\x1b[1m\x1b[32m",  // bold green
        metavar: "\x1b[1m\x1b[34m",  // bold blue
        header: "\x1b[1m\x1b[36m",   // bold
        invalid: "\x1b[31m",         // red
        valid: "\x1b[32m",           // green
        text: "\x1b[0m]",            // reset
    };
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
}

impl TryFrom<u8> for Style {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'0' => Ok(Style::Text),
            b'1' => Ok(Style::Emphasis),
            b'2' => Ok(Style::Literal),
            b'3' => Ok(Style::Metavar),
            b'4' => Ok(Style::Header),
            b'5' => Ok(Style::Valid),
            b'6' => Ok(Style::Invalid),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    /// Plain text, no decorations
    Text,

    /// Word with emphasis - things like "Usage", "Available options", etc
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

#[test]
fn render_simple() {
    let r = apply_style("a\tb", 10, None);
    assert_eq!(r, "a         b\n");

    let r = apply_style("a\n\tb", 6, None);
    assert_eq!(r, "a\n      b\n");

    let r = apply_style("a b c d", 3, None);
    assert_eq!(r, "a b c d\n");
}

#[inline(never)]
pub(crate) fn char_width(s: &str) -> usize {
    s.chars().count()
}
