use crate::{long, positional, short};

use super::{run_parser, Alt, Pair, Parser};

#[test]
fn simple_flag_parser() {
    let alice = long("alice").switch();
    let r = run_parser(&alice, &["--alice"]);
    assert_eq!(r, Ok(true));

    let r = run_parser(&alice, &[]);
    assert_eq!(r, Ok(false));
}

#[test]
fn pair_of_flags() {
    let alice = long("alice").switch();
    let bob = long("bob").switch();
    let both = Pair(alice, bob);

    let r = run_parser(&both, &["--alice", "--bob"]);
    assert_eq!(r, Ok((true, true)));

    let r = run_parser(&both, &["--bob"]);
    assert_eq!(r, Ok((false, true)));

    let r = run_parser(&both, &["--alice"]);
    assert_eq!(r, Ok((true, false)));

    let r = run_parser(&both, &[]);
    assert_eq!(r, Ok((false, false)));
}

#[test]
fn req_flag() {
    let alice = long("alice").req_flag(());

    let r = run_parser(&alice, &["--alice"]);
    assert_eq!(r, Ok(()));

    let r = run_parser(&alice, &[]).unwrap_err();
    assert_eq!(r, "Expected --alice");
}

#[test]
fn alt_of_req() {
    let alice = long("alice").req_flag('a').into_box();
    let bob = long("bob").req_flag('b').into_box();

    let alt = Alt {
        items: vec![alice, bob],
    };

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
    let pair = Pair(alice1, alice2);
    let r = run_parser(&pair, &["--alice", "--alice"]);
    assert_eq!(r, Ok(('1', '2')));
}

#[test]
fn simple_positional() {
    let a = positional::<String>("ARG");

    let r = run_parser(&a, &[]).unwrap_err();
    assert_eq!(r, "Expected <ARG>");

    let r = run_parser(&a, &["item"]);
    assert_eq!(r.as_deref(), Ok("item"));
}

#[test]
fn pair_of_positionals() {
    let alice = positional::<u32>("ALICE");
    let bob = positional::<u32>("BOB");
    let both = Pair(alice, bob);

    let r = run_parser(&both, &["1", "2"]);
    assert_eq!(r, Ok((1, 2)));

    let r = run_parser(&both, &["1"]).unwrap_err();
    assert_eq!(r, "Expected <BOB>");

    let r = run_parser(&both, &[]).unwrap_err();
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
    let ab = Pair(a, b);
    let c = short('c').req_flag('c');
    let abc = Pair(ab, c);
    let r = run_parser(&abc, &["-a", "-b", "-c"]);
    assert_eq!(r, Ok((('a', 'b'), 'c')));
}

#[test]
/// This can never produce the result since `a` is greedy
fn many_positionals_bad() {
    let a = positional::<String>("A").many::<Vec<_>>();
    let b = positional::<String>("B");
    let p = Pair(a, b);

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
    let alice = Pair(alice_f, alice_p).into_box();
    let bob = Pair(bob_f, bob_p).into_box();

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
