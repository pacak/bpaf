use std::ffi::OsStr;

use crate::arg::lex_os_arg;

#[cfg(any(windows, unix))]
#[test]
// There's no tests for "other" platforms specifically, but I think it should be covered
fn lexing_works_with_valid_and_invalid_utf8() {
    #[derive(Debug, Eq, PartialEq)]
    enum ArgType {
        Short,
        Long,
    }

    for (i_c, prefix) in [
        (ArgType::Short, "f"),
        (ArgType::Long, "foo"),
        (ArgType::Long, "口水鸡"),
    ] {
        for valid in [true, false] {
            use std::ffi::OsString;

            use crate::arg::Arg;

            let i_prefix = OsString::from(prefix);
            let mut i_suffix;
            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStringExt;
                i_suffix = OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f]);
            }
            #[cfg(not(windows))]
            {
                use std::os::unix::ffi::OsStringExt;
                i_suffix = OsString::from_vec(vec![0x66, 0x6f, 0xD8, 0x6f]);
            }

            if valid {
                i_suffix.clear();
                i_suffix.push("helloworld");
            }

            let mut os_string = OsString::new();
            match i_c {
                ArgType::Short => os_string.push("-"),
                ArgType::Long => os_string.push("--"),
            }
            os_string.push(&i_prefix);
            os_string.push("=");
            os_string.push(&i_suffix);

            let ref n @ Arg::Named {
                ref name,
                value: Some((_adj, value)),
            } = lex_os_arg(&os_string)
            else {
                unreachable!()
            };
            match name {
                crate::Name::Short(s) => {
                    assert_eq!(i_c, ArgType::Short);
                    assert_eq!(format!("{s}"), prefix);
                }
                crate::Name::Long(cow) => {
                    assert_eq!(i_c, ArgType::Long);
                    assert_eq!(cow, prefix);
                }
            }

            assert_eq!(*value, i_suffix);
            assert_eq!(os_string, n.encode());
        }
    }
}

#[test]
fn lexer_doesnt_throw_away_data() {
    for val in [
        "--",
        "-",
        "--=",
        "--foo",
        "--foo=bar",
        "-ffoo",
        "-f=foo",
        "-f=",
        "-f",
        "foo",
        "-Obits=2048",
    ] {
        let os_value: &OsStr = val.as_ref();
        let lexed = lex_os_arg(os_value);
        assert_eq!(val, lexed.encode());
    }
}

// #[test]
// fn fallback_with_strange_args_produces_same_results() {
//     use crate::args::{split_os_argument, split_os_argument_fallback};
//     let s = std::ffi::OsString::from("-Obits=2048");
//     let r1 = split_os_argument(&s);
//     let r2 = split_os_argument_fallback(&s);
//     assert_eq!(r1, r2);
// }
