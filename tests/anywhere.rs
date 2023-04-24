use bpaf::*;

#[test]
fn parse_anywhere_positional() {
    let a = any::<String>("x")
        .guard(|h| h != "--help", "ignore help")
        .anywhere();

    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    assert_eq!(
        r,
        "Usage: <x> [-b]\n\nAvailable options:\n    -b\n    -h, --help  Prints help information\n"
    );
    // this should be allowed because "anywhere" prevents anything inside from being positional
    parser.check_invariants(true);
}

#[test]
fn parse_anywhere_no_catch() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).anywhere();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    // Usage: -a <x> [-c],

    let r = parser
        .run_inner(Args::from(&["3", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");

    let r = parser
        .run_inner(Args::from(&["-c", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-c", "-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");

    let r = parser
        .run_inner(Args::from(&["-a", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Expected <x>, got \"-c\". Pass --help for usage information"
    );

    let r = parser
        .run_inner(Args::from(&["-a", "221b", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");
}

#[test]
fn parse_anywhere_catch_required() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).anywhere().catch();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(Args::from(&["-c", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    // this should complain about unexpected -a
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");

    let r = parser
        .run_inner(Args::from(&["-c", "-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");

    let r = parser
        .run_inner(Args::from(&["-a", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");

    let r = parser
        .run_inner(Args::from(&["-a", "221b", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Couldn't parse \"221b\": invalid digit found in string");

    let r = parser
        .run_inner(Args::from(&["3", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <x>, pass --help for usage information");
}

#[test]
fn parse_anywhere_catch_optional() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("x");
    let ab = construct!(a, b).anywhere().catch().optional();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser
        .run_inner(Args::from(&["-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["-c", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["-c", "-a", "221b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["-a", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["-a", "221b", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "-a is not expected in this context");

    let r = parser
        .run_inner(Args::from(&["3", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "3 is not expected in this context");
}
