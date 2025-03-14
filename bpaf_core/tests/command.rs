#![allow(clippy::bool_assert_comparison)] // we are parsing values
use bpaf_core::*;

#[test]
fn simple_command() {
    let a = short('a').switch().to_options();

    let parser = a.command("alice").long("bob").to_options();

    let r = parser.run_inner(["alice"]).unwrap();
    assert_eq!(r, false);

    let r = parser.run_inner(["bob", "-a"]).unwrap();
    assert_eq!(r, true);

    // let r = parser.run_inner(["-a"]).unwrap_err();
    // assert_eq!(r, "parser doesn't support -a, but subcommand alice does");
}
