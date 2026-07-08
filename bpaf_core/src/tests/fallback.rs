use crate::*;

#[test]
fn fallback_str_ok() {
    let parser = short('a')
        .argument::<u32>("A")
        .fallback_str("42")
        .to_options();

    let r = parser.run_inner("-a 1").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn fallback_str_invalid_fallback() {
    let parser = short('a')
        .argument::<u32>("A")
        .fallback_str("not a number")
        .to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "invalid digit found in string\n");
}

#[test]
fn fallback_str_parse_error_passthrough() {
    let parser = short('a')
        .argument::<u32>("A")
        .fallback_str("42")
        .to_options();

    let r = parser.run_inner("-a x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'x': invalid digit found in string\n");
}

#[test]
fn fallback_str_display() {
    let parser = long("port")
        .help("listening port")
        .argument::<u16>("PORT")
        .fallback_str("8080")
        .display_fallback()
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [--port=PORT]

Available options:
        --port=PORT  listening port
                     [default: 8080]
    -h, --help       Prints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner("--port 3000").unwrap();
    assert_eq!(r, 3000);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 8080);
}

#[test]
fn fallback_str_on_positional() {
    let parser = positional::<String>("NAME")
        .fallback_str("world")
        .to_options();

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, "hello");

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, "world");
}

#[test]
fn fallback_str_with_guard() {
    let parser = short('n')
        .argument::<u32>("N")
        .fallback_str("5")
        .guard(|&n| n > 0, "must be positive")
        .to_options();

    let r = parser.run_inner("-n 10").unwrap();
    assert_eq!(r, 10);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 5);

    let r = parser.run_inner("-n 0").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-n 0': must be positive\n");
}

#[test]
fn parse_with_string_lit() {
    let parser = short('a')
        .argument::<u32>("N")
        .fallback_with(|| <_ as std::str::FromStr>::from_str("42"))
        .to_options();

    let r = parser.run_inner("-a 13").unwrap();
    assert_eq!(r, 13);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 42);
}
