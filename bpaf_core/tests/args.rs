use bpaf_core::*;

fn parser() -> OptionParser<usize> {
    short('a').long("alice").argument("A").to_options()
}

#[test]
fn short_separate() {
    let r = parser().run_inner(["-a", "10"]).unwrap();
    assert_eq!(r, 10);
}

#[test]
fn short_join_eq() {
    let r = parser().run_inner(["-a=10"]).unwrap();
    assert_eq!(r, 10);
}

#[test]
fn short_ajoint() {
    let r = parser().run_inner(["-a10"]).unwrap();
    assert_eq!(r, 10);
}

#[test]
fn merged_shorts_simple() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c).to_options();
    let r = parser.run_inner(["-abc"]).unwrap();
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
