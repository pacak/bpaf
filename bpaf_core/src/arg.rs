use std::{borrow::Cow, ffi::OsStr};

use crate::{Lit, Name};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Adjacency {
    // for short arguments `-foutput`, for long - not possible
    Immediate,
    // `-f=output`, `--foo=output`
    WithEq,
}
impl std::fmt::Display for Adjacency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Adjacency::Immediate => Ok(()),
            Adjacency::WithEq => f.write_str("="),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    /// Named item with short (`-f`) or long (`-foo`) name and, optionally, attached value
    ///
    Named {
        name: Name<'a>,
        value: Option<(Adjacency, &'a OsStr)>,
    },
    /// Positional item
    Pos { value: &'a OsStr },
}

pub(crate) fn as_name<'a>(val: &'a OsStr) -> Option<Lit<'a>> {
    let s = val.to_str()?;
    let first = s.chars().next()?;
    Some(Lit(if s.len() > first.len_utf8() {
        Name::Long(Cow::Borrowed(s))
    } else {
        Name::Short(first)
    }))
}

#[cfg(test)]
impl Arg<'_> {
    pub(crate) fn encode(&self) -> std::ffi::OsString {
        match self {
            Arg::Named { name, value } => {
                let mut res = std::ffi::OsString::new();
                match name {
                    Name::Long(name) => {
                        res.push("--");
                        res.push(name.as_ref());
                    }
                    Name::Short(name) => {
                        let mut b = [0; 4];
                        res.push("-");
                        let result = name.encode_utf8(&mut b);
                        res.push(result);
                    }
                }
                match value {
                    Some((adj, value)) => {
                        match adj {
                            Adjacency::Immediate => {}
                            Adjacency::WithEq => res.push("="),
                        }
                        res.push::<&OsStr>(value.as_ref());
                        res
                    }
                    None => res,
                }
            }
            Arg::Pos { value } => value.to_os_string(),
        }
    }
}

pub(crate) fn lex_os_arg(value: &OsStr) -> Arg<'_> {
    use crate::os_str::OsStrExt as _;
    if let Some(long) = value.strip_prefix("--") {
        match long.split_by_ascii(b'=') {
            // it's just `--` - a positional item
            _ if long.is_empty() => Arg::Pos { value },
            // `--foo=bar`?
            Some((osname, rest)) => match osname.to_str() {
                // yes, foo is a valid name - this is a long argument with a value
                Some(name) => Arg::Named {
                    name: Name::Long(Cow::Borrowed(name)),
                    value: Some((Adjacency::WithEq, rest)),
                },
                // no, `foo` is not a valid name, treat the whole thing as positional
                None => Arg::Pos { value },
            },
            // `--foo` ?
            None => match long.to_str() {
                // yes, `foo` is a valid name, this is a long name with no value
                Some(name) => Arg::Named {
                    name: Name::Long(Cow::Borrowed(name)),
                    value: None,
                },
                // no, "foo" is not a valid name, treat the whole thing as positional
                None => Arg::Pos { value },
            },
        }
    } else if let Some(short) = value.strip_prefix("-") {
        let Some((name, suffix)) = short.next_char() else {
            // It's just `-` - a positional item;
            return Arg::Pos { value };
        };
        let name = Name::Short(name);
        let value = match suffix.next_char() {
            // `-f=bar` - a short argument with a value
            Some(('=', rest)) => Some((Adjacency::WithEq, rest)),

            // `-fbar`, a short argument with immediately adjacent value
            Some(_) => Some((Adjacency::Immediate, suffix)),

            // it's just `-f`, no value
            None => None,
        };
        Arg::Named { name, value }
    } else {
        Arg::Pos { value }
    }
}
