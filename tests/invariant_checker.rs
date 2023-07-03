use bpaf::*;

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn positional_before_argument() {
    let a = positional::<String>("a");
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
    let a = positional::<String>("a");
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

#[test]
fn floating_adjacent_is_ok() {
    let a = short('a').req_flag(());
    let b = positional::<String>("B");
    let ab = construct!(a, b).adjacent();
    let c = short('c').switch();
    construct!(ab, c).to_options().check_invariants(false);
}

#[should_panic(expected = "bpaf usage BUG: all positional and command items")]
#[test]
fn fixed_adjacent_is_not_ok() {
    let a = positional::<String>("A");
    let b = positional::<String>("B");
    let ab = construct!(a, b).adjacent();
    let c = short('c').switch();
    construct!(ab, c).to_options().check_invariants(false);
}
