impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Text => write!(f, "\u{1B}[0m"),
            Style::Emphasis => write!(f, "\u{1B}[1m"),
            Style::Literal => write!(f, "\u{1B}[2m"),
            Style::Metavar => write!(f, "\u{1B}[3m"),
            Style::Header => write!(f, "\u{1B}[4m"),
            Style::Invalid => write!(f, "\u{1B}[9m"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Emphasis<T>(pub T);

impl<T: std::fmt::Display> std::fmt::Display for Emphasis<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}{:#}{}", Style::Emphasis, &self.0, Style::Text)
        } else {
            write!(f, "{}{}{}", Style::Emphasis, &self.0, Style::Text)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Invalid<T>(pub T);

impl<T: std::fmt::Display> std::fmt::Display for Invalid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}{:#}{}", Style::Invalid, &self.0, Style::Text)
        } else {
            write!(f, "{}{}{}", Style::Invalid, &self.0, Style::Text)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Style {
    /// Plain text, no decorations
    Text,

    /// Word with emphasis - things like “Usage”, “Available options”, etc
    Emphasis,

    /// Something user needs to type literally - command names, etc
    Literal,

    /// Metavavar placeholder - something user needs to replace with own input
    Metavar,

    /// Section header
    Header,

    /// Invalid input given by user - used to display invalid parts of the input
    Invalid,
}

#[inline(never)]
pub(crate) fn mono(input: &str) -> String {
    // TODO - this can be done inplace if input is taken by value
    let mut out = Vec::<u8>::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut style = Style::Text;
    let mut state = State::Anywhere;
    let mut next_style = Style::Text;
    let mut state_start = 0;
    #[derive(Copy, Clone)]
    enum State {
        Anywhere,
        Csi,
        Style,
        Fin,
    }
    for (ix, c) in bytes.iter().copied().enumerate() {
        match state {
            State::Anywhere => {
                if c == 0x1B {
                    state_start = ix;
                    state = State::Csi;
                } else {
                    out.push(c)
                }
            }
            State::Csi => {
                if c == b'[' {
                    state = State::Style;
                } else {
                    out.extend_from_slice(&bytes[state_start..ix + 1]);
                    state = State::Anywhere;
                }
            }
            State::Style => {
                next_style = match c {
                    b'0' => Style::Text,
                    b'1' => Style::Emphasis,
                    b'2' => Style::Literal,
                    b'3' => Style::Metavar,
                    b'4' => Style::Header,
                    b'9' => Style::Invalid,
                    _ => {
                        out.extend_from_slice(&bytes[state_start..ix + 1]);
                        state = State::Anywhere;
                        continue;
                    }
                };
                state = State::Fin;
            }
            State::Fin => {
                if c == b'm' {
                    match (style, next_style) {
                        (Style::Text, Style::Emphasis) => out.push(b'`'),
                        (Style::Text, Style::Invalid) => out.push(b'`'),
                        (Style::Emphasis, Style::Text) => out.push(b'`'),
                        (Style::Invalid, Style::Text) => out.push(b'`'),
                        _ => {}
                    }
                    style = next_style;
                } else {
                    out.extend_from_slice(&bytes[state_start..ix + 1]);
                }
                state = State::Anywhere;
            }
        }
    }
    String::from_utf8(out).expect("Should be valid by construction")
}
