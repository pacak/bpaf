use bpaf_core::*;

#[test]
fn pure_exits_immediately() {
    let parser = pure(42).to_options();

    let r = parser.run_inner([]).unwrap();
    assert_eq!(r, 42);
}

#[test]
fn pure_in_construct_sum_1() {
    let a = pure(42);
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner([]).unwrap();
    assert_eq!(r, (42, false));

    let r = parser.run_inner(["-b"]).unwrap();
    assert_eq!(r, (42, true));
}

#[test]
fn pure_in_construct_sum_2() {
    let a = pure(42);
    let b = short('b').switch();
    let parser = construct!(b, a).to_options();

    let r = parser.run_inner([]).unwrap();
    assert_eq!(r, (false, 42));

    let r = parser.run_inner(["-b"]).unwrap();
    assert_eq!(r, (true, 42));
}
