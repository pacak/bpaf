use crate::{named::Name, pecking::Pecking, Error};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::PathBuf,
    str::FromStr,
};

struct ArgItem {
    force_pos: bool,
    value: OsOrStr<'static>,
}

#[derive(Debug, Clone)]
pub(crate) enum OsOrStr<'a> {
    Str(Cow<'a, str>),
    Os(Cow<'a, OsStr>),
}

impl OsOrStr<'_> {
    fn os(&self) -> OsString {
        match self {
            OsOrStr::Str(cow) => String::from(cow.as_ref()).into(),
            OsOrStr::Os(cow) => cow.into(),
        }
    }
    pub(crate) fn is_named(&self) -> bool {
        match self {
            OsOrStr::Str(cow) => cow.starts_with('-'),
            OsOrStr::Os(cow) => false,
        }
    }

    fn str(&self) -> Option<&str> {
        match self {
            OsOrStr::Str(cow) => Some(cow.as_ref()),
            OsOrStr::Os(_) => None,
        }
    }

    pub(crate) fn parse<T>(&self) -> Result<T, String>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: std::fmt::Display,
    {
        use std::any::{Any, TypeId};
        if TypeId::of::<T>() == TypeId::of::<OsString>() {
            let anybox: Box<dyn Any> = Box::new(self.os());
            Ok(*(anybox.downcast::<T>().unwrap()))
        } else if TypeId::of::<T>() == TypeId::of::<PathBuf>() {
            let anybox: Box<dyn Any> = Box::new(PathBuf::from(self.os()));
            Ok(*(anybox.downcast::<T>().unwrap()))
        } else {
            match self.str() {
                Some(s) => T::from_str(s).map_err(|e| e.to_string()),
                None => Err(format!(
                    "{} is not a valid utf8",
                    self.os().to_string_lossy()
                )),
            }
        }
    }
}

impl<'a> OsOrStr<'a> {
    fn as_ref(&'a self) -> OsOrStr<'a> {
        match self {
            OsOrStr::Str(cow) => Self::Str(Cow::Borrowed(cow.as_ref())),
            OsOrStr::Os(cow) => Self::Os(Cow::Borrowed(cow.as_ref())),
        }
    }
}

impl<'a> From<&'a str> for OsOrStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

impl<'a> From<&&'a str> for OsOrStr<'a> {
    fn from(value: &&'a str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a String> for OsOrStr<'a> {
    fn from(value: &'a String) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Arg<'a> {
    Named {
        name: Name<'a>,
        value: Option<OsOrStr<'a>>,
    },
    ShortSet {
        current: usize,
        names: Vec<char>,
    },
    Positional {
        value: OsOrStr<'a>,
    },
}

fn ascii_prefix(input: &OsStr) -> Option<(Cow<str>, Cow<OsStr>)> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let input = input.as_bytes();
        let pos = input.iter().position(|c| !c.is_ascii())?;
        let prefix = std::str::from_utf8(&input[..pos]).ok()?;
        let suffix = OsStr::from_bytes(&input[pos..]);
        Some((Cow::Borrowed(prefix), Cow::Borrowed(suffix)))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let wide = input.encode_wide().collect::<Vec<_>>();
        let pos = wide.iter().position(|c| *c > 128)?;
        let prefix = wide[..pos].iter().map(|c| *c as u8).collect::<Vec<_>>();
        let prefix = std::string::String::from_utf8(prefix).ok()?;
        let suffix = OsString::from_wide(&wide[pos..]);
        Some((Cow::Owned(prefix), Cow::Owned(suffix)))
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

// Try to split OsString into utf8 prefix and non-utf8 body
// split points (_) located as such:
// --foo=_REST
// -f=_REST
// -f_REST
// -foo
//
// It should be able to handle all the same scenarios as `split_param` that include
// a non-utf argument. For fully utf8 items we'll use `split_param` directly
//
//
fn mixed_arg(input: &OsStr) -> Option<Arg> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        let input = input.as_bytes();
        // shortest valid mixed non-utf argument is 3 bytes long: -cX and it must starts with a dash
        if input.len() < 3 || input[0] != b'-' {
            return None;
        }

        if input[1] == b'-' {
            // long name, must be in form of --NAME=REST
            let eq = input.iter().position(|c| *c == b'=')?;
            let name = &input[2..eq];
            if !name.is_ascii() || name.is_empty() {
                return None;
            }
            let name = std::str::from_utf8(&name).ok()?;
            let rest = OsString::from_vec(input.get(eq + 1..)?.to_vec());
            return Some(Arg::Named {
                name: Name::Long(Cow::Borrowed(name)),
                value: todo!("{:?}", Some(rest)),
            });
        } else if input[1].is_ascii_alphanumeric() {
            // short name in form of -N=REST or -NREST
            let rest = &input[if input[2] == b'=' { 3 } else { 2 }..];
            let rest = OsStr::from_bytes(rest);
            return Some(Arg::Named {
                name: Name::Short(input[1] as char),
                value: todo!("{:?}", Some(rest)),
            });
        }
    }
    #[cfg(windows)]
    {}
    return None;

    None
}

enum ShortOrSet<'a> {
    Short(Name<'a>),
    Set(Vec<char>, &'a str),
}

impl<'a> TryFrom<&'a str> for ShortOrSet<'a> {
    type Error = Error;

    #[inline(never)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut s = value.chars();
        let Some(front) = s.next() else {
            return Err(Error::fail("empty short name?"));
        };
        Ok(if s.as_str().is_empty() {
            ShortOrSet::Short(Name::Short(front))
        } else {
            let mut r = vec![front];
            let arg = s.as_str();
            r.extend(s);
            ShortOrSet::Set(r, arg)
        })
    }
}

pub(crate) fn split_param<'a>(
    value: &'a OsOrStr,

    args: &BTreeMap<Name, Pecking>,
    flags: &BTreeMap<Name, Pecking>,
) -> Result<Arg<'a>, Error> {
    match value {
        OsOrStr::Str(cow) => split_str_param(cow.as_ref(), args, flags),
        OsOrStr::Os(cow) => match ascii_prefix(cow.as_ref()) {
            Some(_) => todo!(),
            None => Ok(Arg::Positional {
                value: value.as_ref(),
            }),
        },
    }
}

// Try to parse a front value into a flag/argument/positional/set of bools
//
// Will reject ambiguities or combinations like `-foo=bar`
// Does not check if name is actually available unless faced with ambiguity possibility.
pub(crate) fn split_str_param<'a>(
    value: &'a str,
    args: &BTreeMap<Name, Pecking>,
    flags: &BTreeMap<Name, Pecking>,
) -> Result<Arg<'a>, Error> {
    Ok(if let Some(long) = value.strip_prefix("--") {
        if let Some((name, arg)) = long.split_once('=') {
            // not `--=bar`
            if name.is_empty() {
                return Err(Error::fail("Very unexpected short name"));
            }
            // `--foo=bar`
            Arg::Named {
                name: Name::Long(Cow::Borrowed(name)),
                value: Some(OsOrStr::from(arg)),
            }
        } else {
            // `--foo`
            Arg::Named {
                name: Name::Long(Cow::Borrowed(long)),
                value: None,
            }
        }
    } else if value == "-" {
        // single dash is a positional item

        Arg::Positional {
            value: OsOrStr::from(value),
        }
    } else if let Some(short_name) = value.strip_prefix("-") {
        if let Some((name, arg)) = short_name.split_once('=') {
            // not `-foo=bar`
            let ShortOrSet::Short(name) = ShortOrSet::try_from(name)? else {
                return Err(Error::fail("No -foo=bar plz"));
            };

            // but `-f=bar` is okay
            Arg::Named {
                name,
                value: Some(OsOrStr::from(arg)),
            }
        } else {
            match ShortOrSet::try_from(short_name)? {
                // `-f`
                ShortOrSet::Short(name) => Arg::Named { name, value: None },
                // -foo which can be either `-f=oo` or `-f -o -o`
                // Or both, so do the disambiguation
                ShortOrSet::Set(names, arg) => {
                    let is_arg = args.contains_key(&Name::Short(names[0]));
                    let is_flags = names.iter().all(|f| flags.contains_key(&Name::Short(*f)));

                    match (is_arg, is_flags) {
                        (true, true) => return Err(Error::fail("ambiguity")),
                        (true, false) => Arg::Named {
                            name: Name::Short(names[0]),
                            value: Some(OsOrStr::from(arg)),
                        },
                        (false, true) => Arg::ShortSet { current: 0, names },
                        (false, false) => return Err(Error::fail("not expected in this context")),
                    }
                }
            }
        }
    } else {
        Arg::Positional {
            value: OsOrStr::from(value),
        }
    })
}
