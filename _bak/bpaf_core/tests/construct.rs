use bpaf_core::*;

#[test]
fn named_struct() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Options {
        a: bool,
        b: bool,
    }
    let a = short('a').switch();
    let b = || short('b').switch();
    let parser = construct!(Options { a, b() }).to_options();

    let expected = Options { a: true, b: false };
    let r = parser.run_inner(["-a"]).unwrap();
    assert_eq!(r, expected);
}

#[test]
fn tuple_struct() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Options(bool, bool);
    let a = short('a').switch();
    let b = || short('b').switch();
    let parser = construct!(Options(a, b())).to_options();

    let expected = Options(true, false);
    let r = parser.run_inner(["-a"]).unwrap();
    assert_eq!(r, expected);
}

#[test]
fn named_enum() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum Opt {
        Options { a: bool, b: bool },
    }
    let a = short('a').switch();
    let b = || short('b').switch();
    let parser = construct!(Opt::Options { a, b() }).to_options();

    let expected = Opt::Options { a: true, b: false };
    let r = parser.run_inner(["-a"]).unwrap();
    assert_eq!(r, expected);
}

#[test]
fn tuple_enum() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum Opt {
        Options(bool, bool),
    }
    let a = short('a').switch();
    let b = || short('b').switch();
    let parser = construct!(Opt::Options(a, b())).to_options();

    let expected = Opt::Options(true, false);
    let r = parser.run_inner(["-a"]).unwrap();
    assert_eq!(r, expected);
}

#[test]
fn tuple() {
    let a = short('a').switch();
    let b = || short('b').switch();
    let parser = construct!(a, b()).to_options();

    let expected = (true, false);
    let r = parser.run_inner(["-a"]).unwrap();
    assert_eq!(r, expected);
}
