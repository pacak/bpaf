use bpaf_core::*;

fn parser() -> impl Parser<usize> {
    short('a').long("alice").argument("A")
}

#[test]
fn short_separate() {
    let r = run_parser(&parser(), &["-a", "10"]);
    assert_eq!(r, Ok(10));
}

#[test]
fn short_join_eq() {
    let r = run_parser(&parser(), &["-a=10"]);
    assert_eq!(r, Ok(10));
}

#[test]
fn short_ajoint() {
    let r = run_parser(&parser(), &["-a10"]);
    assert_eq!(r, Ok(10));
}

#[test]
fn merged_shorts_simple() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c);
    let r = run_parser(&parser, ["-abc"]).unwrap();
    assert_eq!(r, (true, true, true));
}

#[test]
fn many_switch() {
    let a = short('a').switch();
    let parser = a.many::<Vec<_>>().to_options();

    let r = parser.run_inner(["-aa"]).unwrap();
    assert_eq!(r, [true, true]);

    let r = parser.run_inner(["-a", "-a"]).unwrap();
    assert_eq!(r, [true, true]);
}
