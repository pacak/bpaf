use std::thread::AccessError;

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

fn split_csi(input: &str) -> Option<(Style, &str)> {
    todo!();
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
pub(crate) fn mono(input: String) -> String {
    // TODO - convert input to Vec, do the transformation inplace then
    // convert it back from Vec to String, should be smaller.
    let mut out = String::with_capacity(input.len());
    let mut input = input.chars();
    let mut style = Style::Text;
    let mut state = State::Anywhere;
    let mut next_style = Style::Text;
    #[derive(Copy, Clone)]
    enum State {
        Anywhere,
        Csi,
        Style,
        Fin,
    }
    let mut c0 = ' ';
    loop {
        let Some(c) = input.next() else {
            return out;
        };
        match state {
            State::Anywhere if c == '\u{1B}' => state = State::Csi,
            State::Anywhere => out.push(c),
            State::Csi if c == '[' => state = State::Style,
            State::Csi => {
                out.push('\u{1B}');
                out.push(c);
                state = State::Anywhere;
            }
            State::Style => {
                next_style = match c {
                    '0' => Style::Text,
                    '1' => Style::Emphasis,
                    '2' => Style::Literal,
                    '3' => Style::Metavar,
                    '4' => Style::Header,
                    '9' => Style::Invalid,
                    _ => {
                        out.push_str("\u{1B}[");
                        out.push(c);
                        state = State::Anywhere;
                        continue;
                    }
                };
                c0 = c;
                state = State::Fin;
            }
            State::Fin if c == 'm' => {
                match (style, next_style) {
                    (Style::Text, Style::Emphasis) => out.push('`'),
                    (Style::Text, Style::Invalid) => out.push('`'),
                    (Style::Emphasis, Style::Text) => out.push('`'),
                    (Style::Invalid, Style::Text) => out.push('`'),
                    _ => {}
                }
                style = next_style;
                state = State::Anywhere;
            }
            State::Fin => {
                out.push_str("\u{1B}[");
                out.push(c0);
                out.push(c);
                state = State::Anywhere;
            }
        }
    }
}
