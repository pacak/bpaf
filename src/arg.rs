use std::ffi::{OsStr, OsString};

/// Preprocessed command line argument
///
/// [`OsString`] in Short/Long correspond to orignal command line item used for errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Arg {
    /// short flag
    Short(char, OsString),
    /// long flag
    Long(String, OsString),
    /// separate word that can be command, positional or an argument to a flag
    Word(OsString),
    /// separate word that goes after --, strictly positional
    PosWord(OsString),
}

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Short(s, _) => write!(f, "-{}", s),
            Arg::Long(l, _) => write!(f, "--{}", l),
            Arg::Word(w) | Arg::PosWord(w) => write!(f, "{}", w.to_string_lossy()),
        }
    }
}

impl Arg {
    pub(crate) fn as_os(&self) -> &OsStr {
        match self {
            Arg::Short(_, os) | Arg::Long(_, os) | Arg::Word(os) | Arg::PosWord(os) => os.as_ref(),
        }
    }
}

pub(crate) fn word_arg(os: OsString, pos_only: bool) -> Arg {
    if pos_only {
        Arg::PosWord(os)
    } else {
        Arg::Word(os)
    }
}

pub(crate) fn push_vec(vec: &mut Vec<Arg>, mut os: OsString, pos_only: &mut bool) {
    if *pos_only {
        return vec.push(word_arg(os, true));
    }

    match split_os_argument(&os) {
        Some((ArgType::Short, short, None)) => {
            for f in short.chars() {
                vec.push(Arg::Short(f, os));
                os = OsString::new();
            }
        }
        Some((ArgType::Short, short, Some(arg))) => {
            assert_eq!(
                short.len(),
                1,
                "short flag with an argument must have only one key"
            );
            let key = short.chars().next().unwrap();
            vec.push(Arg::Short(key, os));
            vec.push(arg);
        }
        Some((ArgType::Long, long, None)) => {
            vec.push(Arg::Long(long, os));
        }
        Some((ArgType::Long, long, Some(arg))) => {
            vec.push(Arg::Long(long, os));
            vec.push(arg);
        }
        _ => {
            if *pos_only {
                vec.push(Arg::PosWord(os));
            } else {
                *pos_only = os == "--";
                vec.push(Arg::Word(os));
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
/// on supporting -fbar
/// - ideally bpaf wants to support any utf8 character (here `f`) which requires detecting one
///   out of bytes on unix and utf16 codepoints on windows
/// - bpaf needs to store ambigous combo of -f=bar and -f -b -a -r until user consumes -f either as
///   a flag or as an argument and drop -b, -a and -r if it was an argument.
/// - bpaf wants to prevent users from using parsers for -b, -a or -r before parser for -f
///
/// Conclusion: possible in theory but adds too much complexity for the value it offers.
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
                Some(EQUALS) => break,
                Some(val) => name.push(val),
                None => {
                    if name.is_empty() {
                        return None;
                    }
                    return Some((ty, str_from_vec(name)?, None));
                }
            }
        }

        // name must be present
        if name.is_empty() {
            return None;
        }
        let name = str_from_vec(name)?;
        let word = word_arg(os_from_vec(items.collect()), false);
        Some((ty, name, Some(word)))
    }
    #[cfg(not(any(unix, windows)))]
    {
        split_os_argument_fallback(input)
    }
}

/// similar to [`split_os_argument`] but only works for utf8 values, used as a fallback function
/// on non
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
            Some('=') => break,
            Some(val) => name.push(val),
            None => {
                if name.is_empty() {
                    return None;
                }
                return Some((ty, name, None));
            }
        }
    }
    if name.is_empty() {
        return None;
    }

    Some((
        ty,
        name,
        Some(Arg::Word(OsString::from(chars.collect::<String>()))),
    ))
}
