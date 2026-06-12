use crate::*;

#[test]
fn positional_with_help() {
    let user = positional::<String>("USER").help("github user\nin two lines");
    let api = positional::<String>("API_KEY").help("api key to use");
    let parser = construct!(user, api).to_options();

    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected_help = "\
Usage: app USER API_KEY

Available positional items:
    USER        github user in two lines
    API_KEY     api key to use

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn dash_is_positional() {
    let a = positional::<String>("FILE");
    let parser = a.to_options();
    assert_eq!("-", parser.run_inner("-").unwrap());
}

#[test]
fn helpful_error_message() {
    let parser = positional::<String>("FOO")
        .some("you need to specify at least one FOO")
        .to_options();

    let err = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!("you need to specify at least one FOO\n", err);
}

#[test]
fn positional_argument() {
    let p = positional::<String>("FILE")
        .help("file name")
        .group_help("File to process");
    let parser = p.to_options();

    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app FILE

File to process
    FILE        file name

Available options:
    -h, --help  Prints help information
";

    assert_eq!(expected, help);
}

#[test]
fn positional_help_no_complain_1() {
    let a = positional::<String>("a");
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app <a> [-b]

Available positional items:
    <a>

Available options:
    -b
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn positional_help_no_complain_2() {
    let a = positional::<String>("a");
    let b = short('b').switch();
    let ba = construct!(b, a);
    let c = short('c').switch();
    let parser = construct!(ba, c).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app [-b] <a> [-c]

Available positional items:
    <a>

Available options:
    -b
    -c
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn positional_help_no_complain_3() {
    let a = positional::<String>("a");
    let b = short('b').argument::<String>("B");
    let ba = construct!([b, a]);
    let c = short('c').switch();
    let parser = construct!(ba, c).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app (-b=B | <a>) [-c]

Available positional items:
    <a>

Available options:
    -b=B
    -c
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn positional_help_complain_4() {
    let a = positional::<String>("a");
    let b = short('b').argument::<String>("B");
    let parser = construct!([b, a]).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app (-b=B | <a>)

Available positional items:
    <a>

Available options:
    -b=B
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn strictly_positional() {
    let parser = positional::<String>("A").strict().to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: app -- A

Available positional items:
    A

Available options:
    -h, --help  Prints help information
"
    );

    let r = parser.run_inner("a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'a' (A) to follow '--'\n");

    let r = parser.run_inner("-- a").unwrap();
    assert_eq!(r, "a");

    let r = parser.run_inner("a --").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'a' (A) to follow '--'\n");

    let r = parser.run_inner("--").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A'\n");
}

// #[test]
// fn non_strictly_positional() {
//     let parser = positional::<String>("A").non_strict().to_options();
//
//     let r = parser.run_inner(&["a"]).unwrap();
//     assert_eq!(r, "a");
//
//     let r = parser.run_inner(&["--", "a"]).unwrap_err().unwrap_stderr();
//     assert_eq!(r, "expected 'A' to be on the left side of '--'");
//
//     let r = parser.run_inner(&["--"]).unwrap_err().unwrap_stderr();
//     assert_eq!(r, "expected 'A', pass '--help' for usage information");
// }
