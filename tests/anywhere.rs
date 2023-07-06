use bpaf::*;

#[test]
fn parse_anywhere_positional() {
    let a = any("X", |h| if h != "--help" { Some(h) } else { None })
        .help("all the things")
        .anywhere();

    let b = short('b').help("batch mode").switch();
    let parser: OptionParser<(String, bool)> = construct!(a, b).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: X [-b]

Available options:
    X           all the things
    -b          batch mode
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
    // this should be allowed because "anywhere" prevents anything inside from being positional
    parser.check_invariants(true);
}

#[test]
fn parse_anywhere_no_catch() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("X");
    let ab = construct!(a, b).adjacent();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    // Usage: -a <x> [-c],

    let r = parser.run_inner(&["3", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected `X`, pass `--help` for usage information");

    let r = parser.run_inner(&["-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected `X`, pass `--help` for usage information");

    let r = parser
        .run_inner(&["-a", "221b"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Couldn't parse `221b`: invalid digit found in string\n\n-a THIS -> 221b"
    );

    let r = parser.run_inner(&["-c", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected `X`, pass `--help` for usage information");

    let r = parser
        .run_inner(&["-c", "-a", "221b"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Couldn't parse `221b`: invalid digit found in string\n\n-a THIS -> 221b"
    );

    let r = parser.run_inner(&["-a", "-c"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "Expected `X`, got `-c`. Pass `--help` for usage information\n\n-a STRANGE -> -c"
    );

    let r = parser
        .run_inner(&["-a", "221b", "-c"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Couldn't parse `221b`: invalid digit found in string\n\n-a THIS -> 221b -c"
    );
}

#[test]
fn anywhere_catch_optional() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).adjacent().optional().catch();
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap();
    assert_eq!(r, (Some(((), 10)), false));

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, (None, true));

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, (None, false));
}

#[test]
fn anywhere_catch_many() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).adjacent().many().catch();
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner(&["-a"]).unwrap();

    assert_eq!(r, (vec![], true));

    let r = parser.run_inner(&["-a", "10"]).unwrap();
    assert_eq!(r, (vec![((), 10)], false));

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, (Vec::new(), false));
}

#[test]
fn anywhere_catch_fallback() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).adjacent().fallback(((), 10));
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner(&["-a", "12"]).unwrap();
    assert_eq!(r, (((), 12), false));

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, (((), 10), true));

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, (((), 10), false));
}

#[test]
fn parse_anywhere_catch_optional() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");

    // optional + catch makes it so parser succeeds without consuming anything
    // usually leaving `-a` untouched to be consumed by something else
    let ab = construct!(a, b).adjacent().optional().catch();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(&["-a", "221b"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`-a` is not expected in this context\n\nTHIS -> -a 221b");

    let r = parser.run_inner(&["3", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`3` is not expected in this context\n\nTHIS -> 3 -a");

    let r = parser.run_inner(&["-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` is not expected in this context\n\nTHIS -> -a");

    let r = parser.run_inner(&["-c", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` is not expected in this context\n\n-c THIS -> -a");

    let r = parser
        .run_inner(&["-c", "-a", "221b"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`-a` is not expected in this context\n\n-c THIS -> -a 221b"
    );

    let r = parser.run_inner(&["-a", "-c"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` is not expected in this context\n\nTHIS -> -a -c");

    let r = parser
        .run_inner(&["-a", "221b", "-c"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`-a` is not expected in this context\n\nTHIS -> -a 221b -c"
    );
}

#[test]
fn anywhere_literal() {
    let tag = any(
        "-mode",
        |x: String| if x == "-mode" { Some(()) } else { None },
    );
    let mode = positional::<usize>("value");
    let a = construct!(tag, mode).adjacent().many().catch();
    let b = short('b').switch();
    let parser: OptionParser<(Vec<((), usize)>, bool)> = construct!(a, b).to_options();

    let r = parser.run_inner(&["-b", "-mode", "12"]).unwrap();
    assert_eq!(r, (vec![((), 12)], true));

    let r = parser.run_inner(&["-mode", "12", "-b"]).unwrap();
    assert_eq!(r, (vec![((), 12)], true));

    let r = parser.run_inner(&["-mode", "12"]).unwrap();
    assert_eq!(r, (vec![((), 12)], false));
}
