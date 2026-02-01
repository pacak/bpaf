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

#[test]
fn posix_basic() {
    let p = positional::<String>("X").posix();
    let parser = p.to_options();

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn posix_with_ddash() {
    let p = positional::<String>("X").posix();
    let parser = p.to_options();

    let r = parser.run_inner("-- hello").unwrap();
    assert_eq!(r, "hello");
}

#[test]
fn posix_prevents_named_after() {
    let a = short('a').switch();
    let p = positional::<String>("X").posix();
    let parser = construct!(a, p).to_options();

    let r = parser.run_inner("value -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");
}

#[test]
fn posix_allows_named_before() {
    let a = short('a').switch();
    let p = positional::<String>("X").posix();
    let parser = construct!(a, p).to_options();

    let r = parser.run_inner("-a value").unwrap();
    assert_eq!(r, (true, "value".to_owned()));
}

#[test]
fn posix_then_another_positional() {
    let a = positional::<String>("A").posix();
    let b = positional::<String>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("hello world").unwrap();
    assert_eq!(r, ("hello".to_owned(), "world".to_owned()));
}

#[test]
fn strict_posix_requires_ddash() {
    let p = positional::<i32>("N").strict().posix();
    let parser = p.to_options();

    let r = parser.run_inner("10").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '10' (N) to follow '--'\n");

    let r = parser.run_inner("-- 10").unwrap();
    assert_eq!(r, 10);
}

#[test]
fn strict_posix_with_flag_before() {
    let a = short('a').switch();
    let p = positional::<i32>("N").strict().posix();
    let parser = construct!(a, p).to_options();

    let r = parser.run_inner("-a -- 42").unwrap();
    assert_eq!(r, (true, 42));

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '42' (N) to follow '--'\n");
}

#[test]
fn strict_posix_in_strict_mode_after_ddash() {
    let p = positional::<i32>("N").strict().posix();
    let parser = p.to_options();

    let r = parser.run_inner("-- 42").unwrap();
    assert_eq!(r, 42);
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
