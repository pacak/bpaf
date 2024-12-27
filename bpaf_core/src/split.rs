use crate::{named::Name, pecking::Pecking, Error};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::PathBuf,
    str::FromStr,
};

pub struct Args<'a> {
    args: Vec<OsOrStr<'a>>,
}
impl<'a> AsRef<[OsOrStr<'a>]> for Args<'a> {
    fn as_ref(&self) -> &[OsOrStr<'a>] {
        self.args.as_slice()
    }
}

impl<'a, const N: usize> From<&'a [&'a str; N]> for Args<'a> {
    fn from(value: &'a [&'a str; N]) -> Self {
        Self {
            args: value.iter().copied().map(OsOrStr::from).collect(),
        }
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for Args<'a> {
    fn from(value: [&'a str; N]) -> Self {
        Self {
            args: value.iter().copied().map(OsOrStr::from).collect(),
        }
    }
}

impl<'a> From<&'a [&'a OsStr]> for Args<'a> {
    fn from(value: &'a [&'a OsStr]) -> Self {
        Self {
            args: value.iter().copied().map(OsOrStr::from).collect(),
        }
    }
}

impl<'a> From<&'a [&'a str]> for Args<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        Self {
            args: value.iter().copied().map(OsOrStr::from).collect(),
        }
    }
}

impl<'a> From<&'a [String]> for Args<'a> {
    fn from(value: &'a [String]) -> Self {
        Self {
            args: value.iter().map(|s| OsOrStr::from(s.as_str())).collect(),
        }
    }
}

impl<'a> From<&'a [OsString]> for Args<'a> {
    fn from(value: &'a [OsString]) -> Self {
        Self {
            args: value.iter().map(|s| OsOrStr::from(s.as_os_str())).collect(),
        }
    }
}

impl From<std::env::Args> for Args<'_> {
    fn from(value: std::env::Args) -> Self {
        Self {
            args: value.into_iter().map(OsOrStr::from).collect(),
        }
    }
}

impl From<std::env::ArgsOs> for Args<'_> {
    fn from(value: std::env::ArgsOs) -> Self {
        Self {
            args: value.into_iter().map(OsOrStr::from).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum OsOrStr<'a> {
    Str(Cow<'a, str>),
    Os(Cow<'a, OsStr>),
}

impl std::fmt::Display for OsOrStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsOrStr::Str(cow) => write!(f, "{cow}"),
            OsOrStr::Os(cow) => {
                let os: &OsStr = cow.as_ref();
                write!(f, "{}", os.to_string_lossy())
            }
        }
    }
}

impl OsOrStr<'_> {
    fn os(&self) -> OsString {
        match self {
            OsOrStr::Str(cow) => String::from(cow.as_ref()).into(),
            OsOrStr::Os(cow) => cow.into(),
        }
    }
    pub(crate) fn to_owned(&self) -> OsOrStr<'static> {
        match self {
            OsOrStr::Str(cow) => OsOrStr::Str(Cow::Owned(cow.as_ref().to_owned())),
            OsOrStr::Os(cow) => {
                let os: &OsStr = cow.as_ref();
                OsOrStr::Os(Cow::Owned(os.to_owned()))
            }
        }
    }
    pub(crate) fn is_named(&self) -> bool {
        match self {
            OsOrStr::Str(cow) => cow.starts_with('-'),
            OsOrStr::Os(_) => false,
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
    pub(crate) fn as_ref(&'a self) -> OsOrStr<'a> {
        match self {
            OsOrStr::Str(cow) => Self::Str(Cow::Borrowed(cow.as_ref())),
            OsOrStr::Os(cow) => Self::Os(Cow::Borrowed(cow.as_ref())),
        }
    }
}

impl PartialEq<str> for OsOrStr<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            OsOrStr::Str(cow) => cow == other,
            OsOrStr::Os(cow) => other == AsRef::<OsStr>::as_ref(&cow),
        }
    }
}

impl<'a> From<&'a str> for OsOrStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

impl From<String> for OsOrStr<'static> {
    fn from(value: String) -> Self {
        Self::Str(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for OsOrStr<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Str(value)
    }
}

impl<'a> From<&'a OsStr> for OsOrStr<'a> {
    fn from(value: &'a OsStr) -> Self {
        Self::Os(Cow::Borrowed(value))
    }
}

impl From<OsString> for OsOrStr<'static> {
    fn from(value: OsString) -> Self {
        Self::Os(Cow::Owned(value))
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
        OsOrStr::Os(cow) => match cow.to_str() {
            Some(value) => split_str_param(value, args, flags),
            None => match ascii_prefix(cow.as_ref()) {
                Some((prefix, suffix)) => match split_str_param(prefix.as_ref(), args, flags)? {
                    Arg::Named { name, value } => {
                        let value = if let Some(value) = value {
                            let mut os = value.os();
                            os.push(&suffix);
                            os
                        } else {
                            suffix.into_owned()
                        };

                        let name = match name {
                            Name::Short(c) => Name::Short(c),
                            Name::Long(cow) => Name::Long(Cow::Owned(cow.into_owned())),
                        };

                        Ok(Arg::Named {
                            name,
                            value: Some(OsOrStr::from(value)),
                        })
                    }
                    Arg::ShortSet { names, .. } => {
                        let mut os: OsString = String::from_iter(&names[1..]).into();
                        os.push(&suffix);
                        Ok(Arg::Named {
                            name: Name::Short(names[0]),
                            value: Some(OsOrStr::from(os)),
                        })
                    }
                    Arg::Positional { .. } => Ok(Arg::Positional {
                        value: value.clone(),
                    }),
                },
                None => Ok(Arg::Positional {
                    value: value.as_ref(),
                }),
            },
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
                        (true, true) => return Err(Error::fail("ambiguity")), // TODO
                        (_, false) => Arg::Named {
                            name: Name::Short(names[0]),
                            value: Some(OsOrStr::from(arg)),
                        },
                        (false, true) => Arg::ShortSet { current: 0, names },
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

#[cfg(any(windows, unix))]
#[test]
fn non_utf8() {
    let suffix;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        suffix = OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f]);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        suffix = OsString::from_vec(vec![0x66, 0x6f, 0xD8, 0x6f]);
    }

    let mut os1 = OsString::new();
    let mut os2 = OsString::new();
    let m = BTreeMap::new();
    for n in &[Name::Short('a'), Name::Long(Cow::Borrowed("alice"))] {
        for sep in &["=", ""] {
            for mid in &["x", ""] {
                if matches!(n, Name::Long(_)) && sep.is_empty() {
                    continue;
                }

                os1.clear();
                os1.push(format!("{n}{sep}{mid}"));
                os1.push(&suffix);

                let val = OsOrStr::from(os1.as_os_str());
                let r = split_param(&val, &m, &m).unwrap();
                let Arg::Named { name, value } = r else {
                    panic!("{os1:?} should parse into a named arg");
                };

                os2.clear();
                os2.push(mid);
                os2.push(&suffix);
                assert_eq!(name, n.as_ref());
                assert_eq!(value.unwrap().os(), os2);
            }
        }
    }
}
