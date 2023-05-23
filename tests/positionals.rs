use bpaf::*;

#[test]
fn positional_with_help() {
    let user = positional::<String>("USER").help("github user\nin two lines");
    let api = positional::<String>("API_KEY").help("api key to use");
    let parser = construct!(user, api).to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: USER API_KEY

Available positional items:
    USER        github user in two lines
    API_KEY     api key to use

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn help_for_positional() {
    let c = positional::<String>("C").help("help for\nc");
    let d = positional::<String>("DDD").help("help for\nddd");
    let parser = construct!(c, d).to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: C DDD

Available positional items:
    C           help for c
    DDD         help for ddd

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn dash_is_positional() {
    let a = positional::<String>("FILE");
    let parser = a.to_options();
    assert_eq!("-", parser.run_inner(Args::from(&["-"])).unwrap());
}

#[test]
fn helpful_error_message() {
    let parser = positional::<String>("FOO")
        .some("You need to specify at least one FOO")
        .to_options();

    let err = parser
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("You need to specify at least one FOO", err);
}

#[test]
fn positional_argument() {
    let p = positional::<String>("FILE")
        .help("file name")
        .group_help("File to process");
    let parser = p.to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: FILE

Available positional items:
  File to process
    FILE        file name

Available options:
    -h, --help  Prints help information
";

    assert_eq!(expected, help);
}

#[test]
#[should_panic(expected = "bpaf usage BUG: all positional")]
fn positional_help_complain_1() {
    let a = positional::<String>("a");
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stderr();
}

#[test]
#[should_panic(expected = "bpaf usage BUG: all positional")]
fn positional_help_complain_2() {
    let a = positional::<String>("a");
    let b = short('b').switch();
    let ba = construct!(b, a);
    let c = short('c').switch();
    let parser = construct!(ba, c).to_options();

    parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stderr();
}

#[test]
#[should_panic(expected = "bpaf usage BUG: all positional")]
fn positional_help_complain_3() {
    let a = positional::<String>("a");
    let b = short('b').argument::<String>("B");
    let ba = construct!([b, a]);
    let c = short('c').switch();
    let parser = construct!(ba, c).to_options();

    parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stderr();
}

#[test]
fn positional_help_complain_4() {
    let a = positional::<String>("a");
    let b = short('b').argument::<String>("B");
    let parser = construct!([b, a]).to_options();

    parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
}

#[test]
fn strictly_positional() {
    let parser = positional::<String>("A").strict().to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "Usage: -- A\n\nAvailable options:\n    -h, --help  Prints help information\n"
    );

    let r = parser
        .run_inner(Args::from(&["a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected `A` to be on the right side of `--`");

    let r = parser.run_inner(Args::from(&["--", "a"])).unwrap();
    assert_eq!(r, "a");

    let r = parser
        .run_inner(Args::from(&["a", "--"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected `A` to be on the right side of `--`");

    let r = parser
        .run_inner(Args::from(&["--"]))
        .unwrap_err()
        .unwrap_stderr();
    // TODO - hmmmm.... this is a bit odd
    assert_eq!(r, "Expected `-- A`, pass `--help` for usage information");
}
