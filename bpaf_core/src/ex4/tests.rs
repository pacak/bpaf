use super::{long, parse_args, positional, Alt, Error, Pair, Parser};

#[test]
fn simple_flag_parser() {
    let alice = long("alice").switch();
    let r = parse_args(&alice, &["--alice".into()]);
    assert_eq!(r, Ok(true));

    let r = parse_args(&alice, &[]);
    assert_eq!(r, Ok(false));
}

#[test]
fn pair_of_flags() {
    let alice = long("alice").switch();
    let bob = long("bob").switch();
    let both = Pair(alice, bob);

    let r = parse_args(&both, &["--alice".into(), "--bob".into()]);
    assert_eq!(r, Ok((true, true)));

    let r = parse_args(&both, &["--bob".into()]);
    assert_eq!(r, Ok((false, true)));

    let r = parse_args(&both, &["--alice".into()]);
    assert_eq!(r, Ok((true, false)));

    let r = parse_args(&both, &[]);
    assert_eq!(r, Ok((false, false)));
}

#[test]
fn req_flag() {
    let alice = long("alice").req_flag(());

    let r = parse_args(&alice, &["--alice".into()]);
    assert_eq!(r, Ok(()));

    let r = parse_args(&alice, &[]);
    assert_eq!(r, Err(Error::Missing));
}

#[test]
fn alt_of_req() {
    let alice = long("alice").req_flag('a').into_box();
    let bob = long("bob").req_flag('b').into_box();

    let alt = Alt {
        items: vec![alice, bob],
    };

    let r = parse_args(&alt, &["--alice".into(), "--bob".into()]);
    assert_eq!(r, Err(Error::Invalid));

    let r = parse_args(&alt, &["--alice".into()]);
    assert_eq!(r, Ok('a'));

    let r = parse_args(&alt, &["--bob".into()]);
    assert_eq!(r, Ok('b'));
}

#[test]
fn simple_positional() {
    let a = positional::<String>("ARG");

    let r = parse_args(&a, &[]);
    assert_eq!(r, Err(Error::Missing));

    let r = parse_args(&a, &["item".into()]);
    assert_eq!(r.as_deref(), Ok("item"));
}

#[test]
fn pair_of_positionals() {
    let alice = positional::<u32>("ALICE");
    let bob = positional::<u32>("BOB");
    let both = Pair(alice, bob);

    let r = parse_args(&both, &["1".into(), "2".into()]);
    assert_eq!(r, Ok((1, 2)));

    let r = parse_args(&both, &["1".into()]);
    assert_eq!(r, Err(Error::Missing));

    let r = parse_args(&both, &[]);
    assert_eq!(r, Err(Error::Missing));
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

    // let r = parse_args(&alt, &["--alice".into(), "--bob".into()]);
    // assert_eq!(r, Err(Error::Invalid));

    // let r = parse_args(&alt, &["--alice".into(), "10".into()]);
    // assert_eq!(r, Ok(('a', 10)));

    let r = parse_args(&alt, &["--bob".into(), "20".into()]);
    assert_eq!(r, Ok(('b', 20)));
}
