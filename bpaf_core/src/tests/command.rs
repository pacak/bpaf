use crate::*;

#[test]
fn command_with_aliases() {
    let inner = pure(()).to_options().descr("inner descr");
    let cmd = inner.command("foo").long("bar").short('f').short('b');
    let parser = cmd.to_options().descr("outer");

    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected_help = "\
outer

Usage: app COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    foo, f      inner descr
";
    assert_eq!(expected_help, help);

    let help = parser.run_inner("f --help").unwrap_err().unwrap_stdout();

    let expected_help = "inner descr

Usage: app foo

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    // hidden and visible aliases are working
    parser.run_inner("foo").unwrap();
    parser.run_inner("f").unwrap();
    parser.run_inner("bar").unwrap();
    parser.run_inner("b").unwrap();

    // and "k" isn't a thing
    parser.run_inner("k").unwrap_err();
}
