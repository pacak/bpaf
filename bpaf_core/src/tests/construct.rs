use crate::*;

#[test]
fn make_tuple_var() {
    let a = short('a').switch();
    let parser = construct!(a).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn make_tuple_func_no_arg() {
    fn a() -> impl Parser<bool> {
        short('a').switch()
    }
    fn b() -> impl Parser<bool> {
        short('b').switch()
    }
    let parser = construct!(a(), b()).to_options();
    let r = parser.run_inner("-a -b").unwrap();
    assert_eq!(r, (true, true));
}

#[test]
fn make_struct_var() {
    struct A {
        a: bool,
    }
    let a = short('a').switch();
    let parser = construct!(A { a }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A { a: true }));
}

#[test]
fn make_struct_func_no_arg() {
    struct A {
        a: bool,
    }
    fn a() -> impl Parser<bool> {
        short('a').switch()
    }
    let parser = construct!(A { a() }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A { a: true }));
}

#[test]
fn make_enum_var() {
    enum A {
        A { a: bool },
    }
    let a = short('a').switch();
    let parser = construct!(A::A { a }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A::A { a: true }));
}

#[test]
fn make_enum_func_no_arg() {
    enum A {
        A { a: bool },
    }
    fn a() -> impl Parser<bool> {
        short('a').switch()
    }
    let parser = construct!(A::A { a() }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A::A { a: true }));
}

#[test]
fn make_alt_var() {
    let a = short('a').switch();
    let parser = construct!([a]).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn make_alt_func_no_arg() {
    fn a() -> impl Parser<bool> {
        short('a').switch()
    }
    let parser = construct!([a()]).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}
