use std::any::Any;

use crate::{
    construct,
    executor::{Ctx, Fragment},
    long, positional, short, Error,
};

use super::{run_parser, Alt, Parser};

#[test]
fn simple_flag_parser() {
    let alice = long("alice").switch();
    let r = run_parser(&alice, &["--alice"]);
    assert_eq!(r, Ok(true));

    let r = run_parser::<_, &&str>(&alice, &[]);
    assert_eq!(r, Ok(false));
}

#[test]
fn pair_of_flags() {
    let alice = long("alice").switch();
    let bob = long("bob").switch();

    let both = construct!(alice, bob);

    let r = run_parser(&both, &["--alice", "--bob"]);
    assert_eq!(r, Ok((true, true)));

    let r = run_parser(&both, &["--bob"]);
    assert_eq!(r, Ok((false, true)));

    let r = run_parser(&both, &["--alice"]);
    assert_eq!(r, Ok((true, false)));

    let r = run_parser::<_, &&str>(&both, &[]);
    assert_eq!(r, Ok((false, false)));
}

#[test]
fn req_flag_simple() {
    let alice = long("alice").req_flag(());

    let r = run_parser(&alice, ["--alice"]);
    assert_eq!(r, Ok(()));

    let r = run_parser::<_, &str>(&alice, []).unwrap_err();
    assert_eq!(r, "Expected --alice");
}

#[test]
fn alt_of_req() {
    let alice = long("alice").req_flag('a').into_box();
    let bob = long("bob").req_flag('b').into_box();

    let alt = construct!([alice, bob]);

    let r = run_parser(&alt, &["--bob"]);
    assert_eq!(r, Ok('b'));

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
    let pair = construct!(alice1, alice2);
    let r = run_parser(&pair, &["--alice", "--alice"]);
    assert_eq!(r, Ok(('1', '2')));
}

#[test]
fn simple_positional() {
    let a = positional::<String>("ARG");

    let r = run_parser::<_, &&str>(&a, &[]).unwrap_err();
    assert_eq!(r, "Expected <ARG>");

    let r = run_parser(&a, &["item"]);
    assert_eq!(r.as_deref(), Ok("item"));
}

#[test]
fn pair_of_positionals() {
    let alice = positional::<u32>("ALICE");
    let bob = positional::<u32>("BOB");
    let both = construct!(alice, bob);

    let r = run_parser(&both, &["1", "2"]);
    assert_eq!(r, Ok((1, 2)));

    let r = run_parser(&both, &["1"]).unwrap_err();
    assert_eq!(r, "Expected <BOB>");

    let r = run_parser::<_, &&str>(&both, &[]).unwrap_err();
    assert_eq!(r, "Expected <ALICE>");
}

#[test]
fn many_positionals_good() {
    let a = positional::<String>("A").many::<Vec<_>>();

    let r = run_parser(&a, &["a", "b", "c"]).unwrap();
    assert_eq!(r, &["a", "b", "c"]);
}

#[test]
fn depth_first() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let ab = construct!(a, b);
    let c = short('c').req_flag('c');
    let abc = construct!(ab, c);
    let r = run_parser(&abc, &["-a", "-b", "-c"]);
    assert_eq!(r, Ok((('a', 'b'), 'c')));
}

#[test]
/// This can never produce the result since `a` is greedy
fn many_positionals_bad() {
    let a = positional::<String>("A").many::<Vec<_>>();
    let b = positional::<String>("B");
    let p = construct!(a, b);

    let r = run_parser(&p, &["a", "b", "c"]).unwrap_err();
    assert_eq!(r, "Expected <B>");
    //    assert_eq!(r,
}

#[test]
fn badly_emulated_args() {
    let alice_f = long("alice").req_flag('a').into_box();
    let bob_f = long("bob").req_flag('b').into_box();
    let alice_p = positional::<u32>("ALICE");
    let bob_p = positional::<u32>("BOB");
    let alice = construct!(alice_f, alice_p).into_box();
    let bob = construct!(bob_f, bob_p).into_box();

    let alt = Alt {
        items: vec![alice, bob],
    };

    let r = run_parser(&alt, &["--alice", "--bob"]).unwrap_err();
    assert_eq!(r, "Expected <ALICE>, <BOB>");

    // let r = run_parser(
    //     &alt,
    //     &["--alice", "--bob", "10", "20"],
    // );
    // assert_eq!(r, Err(Error::Invalid));

    let r = run_parser(&alt, &["--alice", "10"]);
    assert_eq!(r, Ok(('a', 10)));

    let r = run_parser(&alt, &["--bob", "20"]);
    assert_eq!(r, Ok(('b', 20)));
}

#[test]
fn argument_flavors() {
    let a = short('a').argument::<usize>("A");

    let r = run_parser(&a, &["-a3"]);
    assert_eq!(r, Ok(3));

    let r = run_parser(&a, &["-a=3"]);
    assert_eq!(r, Ok(3));

    let r = run_parser(&a, &["-a", "3"]);
    assert_eq!(r, Ok(3));
}

#[test]
fn simple_guard() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small");

    let r = run_parser(&a, &["-a=3"]);
    assert_eq!(r, Ok(3));

    let r = run_parser(&a, &["-a=13"]).unwrap_err();
    assert_eq!(r, "failed: must be small");
}

#[test]
fn req_flag_and_guard_pair() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small");
    let b = short('b').req_flag(());
    let p = construct!(a, b);

    let r = run_parser(&p, &["-a=13"]).unwrap_err();
    assert_eq!(r, "failed: must be small");

    let r = run_parser(&p, &["-a=3"]).unwrap_err();
    assert_eq!(r, "Expected -b");
}

#[test]
fn guard_and_req_flag_pair() {
    let a = short('a')
        .argument::<usize>("A")
        .guard(|x: &usize| *x < 10, "must be small");
    let b = short('b').req_flag(());
    let p = construct!(b, a);

    let r = run_parser(&p, &["-a=13"]).unwrap_err();
    assert_eq!(r, "failed: must be small");

    let r = run_parser(&p, &["-a=3"]).unwrap_err();
    assert_eq!(r, "Expected -b");
}
