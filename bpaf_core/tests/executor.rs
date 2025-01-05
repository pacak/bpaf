#![allow(clippy::bool_assert_comparison)] // we check parsed value, not a condition

use bpaf_core::*;
#[test]
fn simple_flag_parser() {
    let parser = long("alice").switch().to_options();

    let r = parser.run_inner(["--alice"]).unwrap();
    assert_eq!(r, true);

    let r = parser.run_inner([]).unwrap();
    assert_eq!(r, false);
}

#[test]
fn pair_of_flags() {
    let alice = long("alice").switch();
    let bob = long("bob").switch();

    let parser = construct!(alice, bob).to_options();

    let r = parser.run_inner(["--alice", "--bob"]).unwrap();
    assert_eq!(r, (true, true));

    let r = parser.run_inner(["--bob"]).unwrap();
    assert_eq!(r, (false, true));

    let r = parser.run_inner(["--alice"]).unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner([]).unwrap();
    assert_eq!(r, (false, false));
}

#[test]
fn req_flag_simple() {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Unit;
    let parser = long("alice").req_flag(Unit).to_options();

    let r = parser.run_inner(["--alice"]).unwrap();
    assert_eq!(r, Unit);

    let r = parser.run_inner([]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected --alice");
}

#[test]
fn alt_of_req() {
    let alice = long("alice").req_flag('a').into_box();
    let bob = long("bob").req_flag('b').into_box();

    let parser = construct!([alice, bob]).to_options();

    let r = parser.run_inner(["--bob"]).unwrap();
    assert_eq!(r, 'b');

    // let r = run_parser(&alt, &["--alice", "--bob"]);
    // assert_eq!(r, Err(Error::Invalid));
    //
    // let r = run_parser(&alt, &["--alice"]);
    // assert_eq!(r, Ok('a'));
}

#[test]
fn pair_of_duplicated_names() {
    let alice1 = long("alice").req_flag('1').into_box();
    let alice2 = long("alice").req_flag('2').into_box();
    let parser = construct!(alice1, alice2).to_options();
    let r = parser.run_inner(["--alice", "--alice"]).unwrap();
    assert_eq!(r, ('1', '2'));
}

#[test]
fn simple_positional() {
    let parser = positional::<String>("ARG").to_options();

    let r = parser.run_inner([]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected <ARG>");

    let r = parser.run_inner(["item"]).unwrap();
    assert_eq!(r, "item");
}

#[test]
fn pair_of_positionals() {
    let alice = positional::<u32>("ALICE");
    let bob = positional::<u32>("BOB");
    let parser = construct!(alice, bob).to_options();

    let r = parser.run_inner(["1", "2"]).unwrap();
    assert_eq!(r, (1, 2));

    let r = parser.run_inner(["1"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected <BOB>");

    let r = parser.run_inner([]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected <ALICE>, <BOB>");
}

#[test]
fn many_positionals_good() {
    let a = positional::<String>("A").many::<Vec<_>>().to_options();

    let r = a.run_inner(["a", "b", "c"]).unwrap();
    assert_eq!(r, &["a", "b", "c"]);
}

#[test]
fn depth_first() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let ab = construct!(a, b);
    let c = short('c').req_flag('c');
    let parser = construct!(ab, c).to_options();
    let r = parser.run_inner(["-a", "-b", "-c"]).unwrap();
    assert_eq!(r, (('a', 'b'), 'c'));
}

#[test]
/// This can never produce the result since `a` is greedy
fn many_positionals_bad() {
    let a = positional::<String>("A").many::<Vec<_>>();
    let b = positional::<String>("B");
    let p = construct!(a, b).to_options();

    let r = p.run_inner(["a", "b", "c"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected <B>");
    //    assert_eq!(r,
}

#[test]
fn badly_emulated_args() {
    let alice_f = long("alice").req_flag('a').into_box();
    let bob_f = long("bob").req_flag('b').into_box();
    let alice_p = positional::<u32>("ALICE");
    let bob_p = positional::<u32>("BOB");
    let alice = construct!(alice_f, alice_p);
    let bob = construct!(bob_f, bob_p);

    let a = construct!([alice, bob]).to_options();

    let r = a
        .run_inner(["--alice", "--bob"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <ALICE>, <BOB>");

    // let r = run_parser(
    //     &alt,
    //     &["--alice", "--bob", "10", "20"],
    // );
    // assert_eq!(r, Err(Error::Invalid));

    let r = a.run_inner(["--alice", "10"]).unwrap();
    assert_eq!(r, ('a', 10));

    let r = a.run_inner(["--bob", "20"]).unwrap();
    assert_eq!(r, ('b', 20));
}

#[test]
fn argument_flavors() {
    let a = short('a').argument::<usize>("A").to_options();

    let r = a.run_inner(["-a3"]).unwrap();
    assert_eq!(r, 3);

    let r = a.run_inner(["-a=3"]).unwrap();
    assert_eq!(r, 3);

    let r = a.run_inner(["-a", "3"]).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn simple_guard() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small")
        .to_options();

    let r = a.run_inner(["-a=3"]).unwrap();
    assert_eq!(r, 3);

    let r = a.run_inner(["-a=13"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "failed: must be small");
}

#[test]
fn req_flag_and_guard_pair() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small");
    let b = short('b').req_flag(());
    let p = construct!(a, b).to_options();

    let r = p.run_inner(["-a=13"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "failed: must be small");

    let r = p.run_inner(["-a=3"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected -b");
}

#[test]
fn guard_and_req_flag_pair() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small");
    let b = short('b').req_flag(());
    let p = construct!(b, a).to_options();

    let r = p.run_inner(["-a=13"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "failed: must be small");

    let r = p.run_inner(["-a=3"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected -b");
}
