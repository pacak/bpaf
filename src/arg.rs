use std::ffi::{OsStr, OsString};

/// Preprocessed command line argument
///
/// [`OsString`] in Short/Long correspond to orignal command line item used for errors
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Arg {
    /// short flag: `-f`
    ///
    /// bool indicates if following item is also part of this Short (created
    Short(char, bool, OsString),

    /// long flag: `--flag`
    /// bool tells if it looks like --key=val or not
    Long(String, bool, OsString),

    /// "val" part of --key=val -k=val -kval
    ArgWord(OsString),

    /// separate word that can be command, positional or a separate argument to a flag
    ///
    /// Can start with `-` or `--`, doesn't have to be valid utf8
    ///
    /// `hello`
    Word(OsString),

    /// separate word that goes after `--`, strictly positional
    ///
    /// Can start with `-` or `--`, doesn't have to be valid utf8
    PosWord(OsString),
}

impl Arg {
    pub(crate) fn os_str(&self) -> &OsStr {
        match self {
            Arg::Short(_, _, s)
            | Arg::Long(_, _, s)
            | Arg::ArgWord(s)
            | Arg::Word(s)
            | Arg::PosWord(s) => s.as_ref(),
        }
    }

    pub(crate) fn match_short(&self, val: char) -> bool {
        match self {
            Arg::Short(s, _, _) => *s == val,
            Arg::ArgWord(_) | Arg::Long(_, _, _) | Arg::Word(_) | Arg::PosWord(_) => false,
        }
    }

    pub(crate) fn match_long(&self, val: &str) -> bool {
        match self {
            Arg::Long(s, _, _) => *s == val,
            Arg::Short(_, _, _) | Arg::ArgWord(_) | Arg::Word(_) | Arg::PosWord(_) => false,
        }
    }
}

// short flag disambiguations:
//
// Short flags | short arg
// No          | No        | no problem
// Yes         | No        | use flag
// No          | Yes       | use arg
// Yes         | Yes       | ask user?
//
// -a  - just a regular short flag: "-a"
// -abc - assuming there are short flags a, b and c: "-a -b -c", assuming utf8 values AND there's no argument -a
// -abc - assuming there's no -a -b -c: "-a bc"
// -abc - assuming both short a b c AND there's argument -a - need to disambiguate  on a context level
//
// 1. parse argument into ambigous representation that can store both short flags and argument
// 2. collect short flag/arg when entering the subparsre
// 3. when reaching ambi
//

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Short(s, _, _) => write!(f, "-{}", s),
            Arg::Long(l, _, _) => write!(f, "--{}", l),
            Arg::ArgWord(w) | Arg::Word(w) | Arg::PosWord(w) => {
                write!(f, "{}", w.to_string_lossy())
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum ArgType {
    Short,
    Long,
}

/// split [`OsString`] into argument specific bits
///
/// takes a possibly non-utf8 string looking like "--name=value" and splits it into bits:
/// "--" - type, "name" - name, must be representable as utf8, "=" - optional, "value" - flag
///
/// dashes and equals sign are low codepoint values and - can look for them literally in a string.
/// This probably means not supporting dashes with diacritics, but that's okay
///
/// name must be valid utf8 after conversion and must not include `=`
///
/// argument is optional and can be non valid utf8.
///
/// The idea is to split the [`OsString`] into opaque parts by looking only at the parts simple parts
/// and let stdlib to handle the decoding of those parts.
///
/// performance wise this (at least on unix) works some small number percentage slower than the
/// previous version
///
///
/// Notation -fbar is ambigous and could mean either `-f -b -a -r` or `-f=bar`, resolve it into
/// [`Arg::Ambiguity`] and let subparser disambiguate it later depending on available short flag and
/// arguments
pub(crate) fn split_os_argument(input: &std::ffi::OsStr) -> Option<(ArgType, String, Option<Arg>)> {
    #[cfg(any(unix, windows))]
    {
        // OsString are sequences of smaller smaller elements - bytes in unix and
        // possibly invalid utf16 items on windows
        #[cfg(unix)]
        type Elt = u8;
        #[cfg(windows)]
        type Elt = u16;

        // reuse allocation on unix, don't reuse allocations on windows
        // either case - pack a vector of elements back into OsString
        fn os_from_vec(vec: Vec<Elt>) -> OsString {
            #[cfg(unix)]
            {
                <OsString as std::os::unix::ffi::OsStringExt>::from_vec(vec)
            }
            #[cfg(windows)]
            {
                <OsString as std::os::windows::ffi::OsStringExt>::from_wide(&vec)
            }
        }

        // try to decode elements into a String
        fn str_from_vec(vec: Vec<Elt>) -> Option<String> {
            Some(os_from_vec(vec).to_str()?.to_owned())
        }

        // but in either case dashes and equals are just literal values just with different width
        const DASH: Elt = b'-' as Elt;
        const EQUALS: Elt = b'=' as Elt;

        // preallocate something to store the name. oversized but avoids extra allocations/copying
        let mut name = Vec::with_capacity(input.len());

        let mut items;
        #[cfg(unix)]
        {
            items = std::os::unix::ffi::OsStrExt::as_bytes(input)
                .iter()
                .copied();
        }
        #[cfg(windows)]
        {
            items = std::os::windows::ffi::OsStrExt::encode_wide(input);
        }

        // first item must be dash, otherwise it's positional or a flag value
        if items.next()? != DASH {
            return None;
        }

        // second item may or may not be, but should be present
        let ty;
        match items.next()? {
            DASH => ty = ArgType::Long,
            val => {
                ty = ArgType::Short;
                name.push(val);
            }
        }

        // keep collecting until = or the end of the input
        loop {
            match items.next() {
                Some(EQUALS) => {
                    if ty == ArgType::Short && name.len() > 1 {
                        let mut body = name.drain(1..).collect::<Vec<_>>();
                        body.push(EQUALS);
                        body.extend(items);
                        name.truncate(1);
                        let os = Arg::ArgWord(os_from_vec(body));
                        return Some((ty, str_from_vec(name)?, Some(os)));
                    }
                    break;
                }
                Some(val) => name.push(val),
                None => {
                    if name.is_empty() {
                        return None;
                    }
                    return Some((ty, str_from_vec(name)?, None));
                }
            }
        }

        let name = str_from_vec(name)?;
        let word = {
            let os = os_from_vec(items.collect());
            Arg::ArgWord(os)
        };
        Some((ty, name, Some(word)))
    }
    #[cfg(not(any(unix, windows)))]
    {
        split_os_argument_fallback(input)
    }
}

/// similar to [`split_os_argument`] but only works for utf8 values, used as a fallback function
/// on non windows/unix OSes
#[cfg(any(all(not(windows), not(unix)), test))]
pub(crate) fn split_os_argument_fallback(
    input: &std::ffi::OsStr,
) -> Option<(ArgType, String, Option<Arg>)> {
    // fallback supports only valid utf8 os strings, matches old behavior
    let string = input.to_str()?;

    let mut chars = string.chars();
    let mut name = String::with_capacity(string.len());

    // first character must be dash, otherwise it's positional or a flag value
    if chars.next()? != '-' {
        return None;
    }

    // second character may or may not be
    let ty;
    match chars.next()? {
        '-' => ty = ArgType::Long,
        val => {
            ty = ArgType::Short;
            name.push(val);
        }
    }

    // collect the argument's name up to '=' or until the end
    // if it's a flag
    loop {
        match chars.next() {
            Some('=') => {
                if ty == ArgType::Short && name.len() > 1 {
                    let mut body = name.drain(1..).collect::<String>();
                    body.push('=');
                    body.extend(chars);
                    name.truncate(1);
                    let os = Arg::ArgWord(OsString::from(body));
                    return Some((ty, name, Some(os)));
                }
                break;
            }

            Some(val) => name.push(val),
            None => {
                if name.is_empty() {
                    return None;
                }
                return Some((ty, name, None));
            }
        }
    }

    Some((
        ty,
        name,
        Some(Arg::ArgWord(OsString::from(chars.collect::<String>()))),
    ))
}
