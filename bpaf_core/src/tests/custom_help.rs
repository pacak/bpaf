use crate::*;

#[test]
fn custom_help_section() {
    let a = short('a').help("A flag").req_flag(()).help_literal(
        "\u{1B}[15m\u{1b}[4mExamples\u{1b}[0m\n  -a\tA flag\n  --flag\tDoes something\u{1B}[16m",
    );
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Examples
  -a            A flag
  --flag        Does something

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn transform_line() {
    let a = short('a').switch().global();

    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -h, --help  Prints help information

Global options:
    -a
";
    assert_eq!(r, expected);
}
