use std::ffi::{OsStr, OsString};

use crate::complete::Shell;

#[derive(Debug, Clone)]
pub struct Args {
    pub(crate) app: String,
    pub(crate) items: Vec<OsString>,
    pub(crate) complete: Option<Shell>,
}

impl Args {
    pub(crate) fn get(&self, ix: u32) -> Option<&OsStr> {
        Some(self.items.get(ix as usize)?)
    }

    pub(crate) fn len(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.app = name.into();
        self
    }

    pub fn set_comp(mut self, shell: Shell) -> Self {
        self.complete = Some(shell);
        self
    }
}

impl Args {
    pub fn make(
        app: impl Into<String>,
        items: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> Self {
        let items: Vec<OsString> = items.into_iter().map(|v| v.into()).collect();
        assert!(
            items.len() < i32::MAX as usize,
            "Way too many command line arguments"
        );
        Self {
            app: app.into(),
            complete: None,
            items,
        }
    }
}

impl std::ops::Index<u32> for Args {
    type Output = OsString;

    fn index(&self, index: u32) -> &Self::Output {
        &self.items[index as usize]
    }
}

impl From<&[&str]> for Args {
    fn from(value: &[&str]) -> Self {
        Self::make("app", value.iter().map(OsString::from))
    }
}

impl From<&[OsString]> for Args {
    fn from(value: &[OsString]) -> Self {
        Self::make("app", value.iter().cloned())
    }
}

impl<const W: usize> From<&[&str; W]> for Args {
    fn from(value: &[&str; W]) -> Self {
        Self::make("app", value.iter().map(OsString::from))
    }
}

impl<const W: usize> From<[&str; W]> for Args {
    fn from(value: [&str; W]) -> Self {
        Self::make("app", value.iter().map(OsString::from))
    }
}

impl From<std::env::Args> for Args {
    fn from(mut value: std::env::Args) -> Self {
        let app = std::path::Path::new(&value.next().expect("Empty args?"))
            .file_name()
            .expect("No file?")
            .to_string_lossy()
            .into_owned();
        Self::make(app, value.map(OsString::from))
    }
}

impl From<std::env::ArgsOs> for Args {
    fn from(mut value: std::env::ArgsOs) -> Self {
        let app = std::path::Path::new(&value.next().expect("Empty args?"))
            .file_name()
            .expect("No file?")
            .to_string_lossy()
            .into_owned();
        Self::make(app, value)
    }
}

impl From<&str> for Args {
    fn from(value: &str) -> Self {
        Self::make("app", split(value).unwrap().into_iter().map(OsString::from))
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
        let mut res = Self::make("app", items);
        res.complete = Some(Shell::Test);
        res
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
