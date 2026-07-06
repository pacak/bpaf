use crate::*;

#[test]
fn parse_anywhere_positional() {
    let a = any::<_, String>("X", |h: &str| {
        if h != "--help" {
            Some(h.to_owned())
        } else {
            None
        }
    })
    .help("all the things");

    let b = short('b').help("batch mode").switch();
    let parser: OptionParser<(String, bool)> = construct!(a, b).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app X [-b]

Available positional items:
    X           all the things

Available options:
    -b          batch mode
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn parse_anywhere_no_catch() {
    let b = positional::<usize>("X");
    let ab = short('a').nest(b);
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    // Usage: -a <x> [-c],

    let r = parser.run_inner("3 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a', got '3'\n");

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'X'\n");

    let r = parser.run_inner("-a 221b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");

    let r = parser.run_inner("-c -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'X'\n");

    let r = parser.run_inner("-c -a 221b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");

    let r = parser.run_inner("-a -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'X'\n");

    let r = parser.run_inner("-a 221b -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");
}

#[test]
fn anywhere_catch_optional() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).optional();
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner("-a 10").unwrap();
    assert_eq!(r, (Some(10), false));

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, (None, false));
}

#[test]
fn anywhere_catch_many() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).many();
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-a 10").unwrap();
    assert_eq!(r, (vec![10], false));

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, (Vec::new(), false));
}

#[test]
fn anywhere_catch_fallback_single() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).fallback(10);

    let parser = ab.to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-a 12").unwrap();
    assert_eq!(r, 12);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 10);
}

#[test]
fn anywhere_catch_fallback_sum() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).fallback(10);
    let bc = short('a').flag(1, 0);
    let parser = construct!([ab, bc]).to_options();

    let r = parser.run_inner("-a 12").unwrap();
    assert_eq!(r, 12);

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 10);
}

#[test]
fn anywhere_catch_fallback_prod() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).fallback(10);
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner("-a 12").unwrap();
    assert_eq!(r, (12, false));

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, (10, false));
}

#[test]
fn parse_anywhere_catch_optional() {
    let b = positional::<usize>("x");

    let ab = short('a').nest(b).optional();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner("-a 221b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");

    let r = parser.run_inner("3 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'3' is not expected in this context\n");

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-c -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-c -a 221b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");

    let r = parser.run_inner("-a -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-a 221b -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '221b': invalid digit found in string\n");
}

// #[test]
// fn anywhere_literal() {
//     let tag = any(
//         "-mode",
//         |x: String| if x == "-mode" { Some(()) } else { None },
//     );
//     let mode = positional::<usize>("value");
//     let a = construct!(tag, mode).adjacent().many().catch();
//     let b = short('b').switch();
//     let parser: OptionParser<(Vec<((), usize)>, bool)> = construct!(a, b).to_options();
//
//     let r = parser.run_inner(&["-b", "-mode", "12"]).unwrap();
//     assert_eq!(r, (vec![((), 12)], true));
//
//     let r = parser.run_inner(&["-mode", "12", "-b"]).unwrap();
//     assert_eq!(r, (vec![((), 12)], true));
//
//     let r = parser.run_inner(&["-mode", "12"]).unwrap();
//     assert_eq!(r, (vec![((), 12)], false));
// }
