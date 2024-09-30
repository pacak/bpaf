#![allow(clippy::bool_assert_comparison)] // we are parsing values
use bpaf_core::*;

#[test]
fn simple_missing_command() {
    let parser = pure(42).to_options().command("alice").to_options();

    let r = parser.run_inner([]).unwrap_err().unwrap_stderr();
    let expected = "expected `COMMAND`, pass `--help` for usage information";
    assert_eq!(r, expected);

    let r = parser.run_inner(["alice"]).unwrap();
    assert_eq!(r, 42);
}

#[test]
fn simple_command() {
    let a = short('a').switch().to_options();

    let parser = a.command("alice").long("bob").to_options();

    let r = parser.run_inner([]).unwrap_err().unwrap_stderr();
    let expected = "expected `COMMAND`, pass `--help` for usage information";
    assert_eq!(r, expected);

    let r = parser.run_inner(["alice"]).unwrap();
    assert_eq!(r, false);

    let r = parser.run_inner(["bob", "-a"]).unwrap();
    assert_eq!(r, true);

    let r = parser.run_inner(["-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "`-a` is not valid in this context, did you mean to pass it to command `alice`?"
    );
}
