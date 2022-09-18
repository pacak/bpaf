use std::ffi::OsString;

/// Preprocessed command line argument
///
/// [`OsString`] in Short/Long correspond to orignal command line item used for errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Arg {
    /// ambiguity between group of short options and a short option with an argument
    /// `-abc` can be either equivalent of `-a -b -c` or `-a=bc`
    ///
    /// OsString is always valid utf8 here
    Ambiguity(Vec<char>, OsString),

    /// short flag: `-f`
    ///
    /// bool indicates if following item is also part of this Short (created
    Short(char, bool, OsString),

    /// long flag: `--flag`
    ///
    Long(String, bool, OsString),

    /// separate word that can be command, positional or an argument to a flag
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
            Arg::Ambiguity(_, w) | Arg::Word(w) | Arg::PosWord(w) => {
                write!(f, "{}", w.to_string_lossy())
            }
        }
    }
}

pub(crate) fn push_vec(vec: &mut Vec<Arg>, os: OsString, pos_only: &mut bool) {
    if *pos_only {
        return vec.push(Arg::PosWord(os));
    }

    match split_os_argument(&os) {
        // -f and -fbar
        Some((ArgType::Short, short, None)) => {
            let mut chars = short.chars();
            let mut prev = chars.next();
            let mut ambig = Vec::new();
            for c in chars {
                if let Some(prev) = std::mem::take(&mut prev) {
                    ambig.push(prev);
                }
                ambig.push(c);
            }
            match prev {
                Some(p) => vec.push(Arg::Short(p, false, os)),
                None => {
                    vec.push(Arg::Ambiguity(ambig, os));
                }
            }
        }
        // -f=a
        Some((ArgType::Short, short, Some(arg))) => {
            assert_eq!(
                short.len(),
                1,
                "short flag with an argument must have only one key"
            );
            let key = short.chars().next().unwrap();
            vec.push(Arg::Short(key, true, os));
            vec.push(arg);
        }
        Some((ArgType::Long, long, None)) => {
            vec.push(Arg::Long(long, false, os));
        }
        Some((ArgType::Long, long, Some(arg))) => {
            vec.push(Arg::Long(long, true, os));
            vec.push(arg);
        }
        _ => {
            *pos_only = os == "--";
            if *pos_only {
                vec.push(Arg::PosWord(os));
            } else {
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

        let name = str_from_vec(name)?;
        let word = {
            let os = os_from_vec(items.collect());
            Arg::Word(os)
        };
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

    Some((
        ty,
        name,
        Some(Arg::Word(OsString::from(chars.collect::<String>()))),
    ))
}
