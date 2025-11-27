use std::{
    ffi::{OsStr, OsString},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct Args {
    pub(crate) items: Rc<[OsString]>,
    pub(crate) complete: bool,
}
impl Args {
    pub(crate) fn get(&self, ix: usize) -> Option<&OsString> {
        self.items.get(ix)
    }
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }
}

impl std::ops::Index<usize> for Args {
    type Output = OsString;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl From<&[&str]> for Args {
    fn from(value: &[&str]) -> Self {
        Self {
            items: value.iter().map(OsString::from).collect(),
            complete: false,
        }
    }
}

impl From<&[OsString]> for Args {
    fn from(value: &[OsString]) -> Self {
        Self {
            items: value.into(),
            complete: false,
        }
    }
}

impl<const W: usize> From<[&str; W]> for Args {
    fn from(value: [&str; W]) -> Self {
        Self {
            items: value.iter().map(OsString::from).collect(),
            complete: false,
        }
    }
}

impl From<std::env::ArgsOs> for Args {
    fn from(value: std::env::ArgsOs) -> Self {
        Self {
            items: value.collect(),
            complete: false, // TODO
        }
    }
}

impl From<&str> for Args {
    fn from(value: &str) -> Self {
        Self {
            items: split(value)
                .unwrap()
                .into_iter()
                .map(OsString::from)
                .collect(),
            complete: false,
        }
    }
}

impl From<(&str, &str)> for Args {
    fn from(value: (&str, &str)) -> Self {
        let mut items = split(value.0)
            .unwrap()
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>();
        items.push(OsString::from(value.1));
        Self {
            items: items.into(),
            complete: true,
        }
    }
}

/// An error returned when shell parsing fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("missing closing quote")
    }
}

impl std::error::Error for ParseError {}

enum State {
    /// Within a delimiter.
    Delimiter,
    /// After backslash, but before starting word.
    Backslash,
    /// Within an unquoted word.
    Unquoted,
    /// After backslash in an unquoted word.
    UnquotedBackslash,
    /// Within a single quoted word.
    SingleQuoted,
    /// Within a double quoted word.
    DoubleQuoted,
    /// After backslash inside a double quoted word.
    DoubleQuotedBackslash,
    /// Inside a comment.
    Comment,
}

fn split(s: &str) -> Result<Vec<String>, ParseError> {
    use State::*;

    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = s.chars();
    let mut state = Delimiter;

    loop {
        state = if let Some(c) = chars.next() {
            // Process new character
            match state {
                Delimiter => match c {
                    '\'' => SingleQuoted,
                    '\"' => DoubleQuoted,
                    '\\' => Backslash,
                    '\t' | ' ' | '\n' => Delimiter,
                    '#' => Comment,
                    c => {
                        word.push(c);
                        Unquoted
                    }
                },
                Backslash => match c {
                    '\n' => Delimiter,
                    c => {
                        word.push(c);
                        Unquoted
                    }
                },
                Unquoted => match c {
                    '\'' => SingleQuoted,
                    '\"' => DoubleQuoted,
                    '\\' => UnquotedBackslash,
                    '\t' | ' ' | '\n' => {
                        words.push(std::mem::take(&mut word));
                        Delimiter
                    }
                    c => {
                        word.push(c);
                        Unquoted
                    }
                },
                UnquotedBackslash => match c {
                    '\n' => Unquoted,
                    c => {
                        word.push(c);
                        Unquoted
                    }
                },
                SingleQuoted => match c {
                    '\'' => Unquoted,
                    c => {
                        word.push(c);
                        SingleQuoted
                    }
                },
                DoubleQuoted => match c {
                    '\"' => Unquoted,
                    '\\' => DoubleQuotedBackslash,
                    c => {
                        word.push(c);
                        DoubleQuoted
                    }
                },
                DoubleQuotedBackslash => match c {
                    '\n' => DoubleQuoted,
                    '$' | '`' | '"' | '\\' => {
                        word.push(c);
                        DoubleQuoted
                    }
                    c => {
                        word.push('\\');
                        word.push(c);
                        DoubleQuoted
                    }
                },
                Comment => match c {
                    '\n' => Delimiter,
                    _ => Comment,
                },
            }
        } else {
            // Process end of input
            match state {
                Delimiter | Comment => break,
                Backslash => {
                    word.push('\\');
                    words.push(std::mem::take(&mut word));
                    break;
                }
                Unquoted => {
                    words.push(std::mem::take(&mut word));
                    break;
                }
                UnquotedBackslash => {
                    word.push('\\');
                    words.push(std::mem::take(&mut word));
                    break;
                }
                SingleQuoted | DoubleQuoted | DoubleQuotedBackslash => return Err(ParseError),
            }
        }
    }

    Ok(words)
}
