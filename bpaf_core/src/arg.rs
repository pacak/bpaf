use std::ffi::OsString;

#[derive(Debug, Copy, Clone)]
pub enum Adjacency {
    // for short arguments `-foutput`, for long - not possible
    Immediate,
    // `-f=output`, `--foo=output`
    WithEq,
}

#[derive(Debug, Clone)]
pub enum Arg {
    /// Named item with short (`-f`) or long (`-foo`) name and, optionally, attached value
    ///
    Named {
        name: OwnedName,
        value: Option<(Adjacency, OsString)>,
    },
    /// Positional item
    Pos { value: OsString },
}

#[derive(Debug, Clone)]
pub enum OwnedName {
    Long(String),
    Short(char),
}

impl Arg {
    fn encode(&self) -> OsString {
        match self {
            Arg::Named { name, value } => {
                let mut res = OsString::new();
                match name {
                    OwnedName::Long(name) => {
                        res.push("--");
                        res.push(&name);
                    }
                    OwnedName::Short(name) => {
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
                        res.push(value);
                        res
                    }
                    None => res,
                }
            }
            Arg::Pos { value } => value.clone(),
        }
    }
}

pub(crate) fn lex_os_arg(value: OsString) -> Arg {
    use crate::os_str::OsStrExt as _;
    if let Some(long) = value.strip_prefix("--") {
        match long.split_by_ascii(b'=') {
            // it's just `--` - a positional item
            _ if long.is_empty() => Arg::Pos { value },
            // `--foo=bar`?
            Some((osname, rest)) => match osname.to_str() {
                // yes, foo is a valid name - this is a long argument with a value
                Some(name) => Arg::Named {
                    name: OwnedName::Long(name.to_owned()),
                    value: Some((Adjacency::WithEq, rest.to_owned())),
                },
                // no, `foo` is not a valid name, treat the whole thing as positional
                None => Arg::Pos { value },
            },
            // `--foo` ?
            None => match long.to_str() {
                // yes, `foo` is a valid name, this is a long name with no value
                Some(name) => Arg::Named {
                    name: OwnedName::Long(name.into()),
                    value: None,
                },
                // no, "foo" is not a valid name, treat the whole thing as positional
                _ => Arg::Pos { value },
            },
        }
    } else if let Some(short) = value.strip_prefix("-") {
        let Some((name, suffix)) = short.next_char() else {
            // It's just `-` - a positional item;
            return Arg::Pos { value };
        };
        let name = OwnedName::Short(name);
        let value = match suffix.next_char() {
            // `-f=bar` - a short argument with a value
            Some(('=', rest)) => Some((Adjacency::WithEq, rest.to_owned())),

            // `-fbar`, a short argument with immediately adjacent value
            Some(_) => Some((Adjacency::Immediate, value.to_owned())),

            // it's just `-f`
            None => None,
        };
        Arg::Named { name, value }
    } else {
        Arg::Pos { value }
    }
}
