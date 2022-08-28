use bpaf::*;

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn positional_before_argument() {
    let a = positional("a");
    let b = short('b').switch();
    construct!(a, b).to_options().check_invariants(false)
}

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn command_before_argument() {
    let a = pure(()).to_options().command("xx");
    let b = short('b').switch();
    construct!(a, b).to_options().check_invariants(false)
}

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn positional_before_argument_nested() {
    let a = positional("a");
    let b = short('b').switch();
    construct!(a, b)
        .to_options()
        .command("cmd")
        .to_options()
        .check_invariants(false)
}

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn command_before_argument_nested() {
    let a = pure(()).to_options().command("xx");
    let b = short('b').switch();
    construct!(a, b)
        .to_options()
        .command("cmd")
        .to_options()
        .check_invariants(false)
}
