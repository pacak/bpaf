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
fn alt_req() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner(["-a", "-b"]).unwrap_err().unwrap_stderr();
    let expected = "`-b` cannot be used at the same time as `-a`";
    assert_eq!(r, expected);
}

#[test]
fn many_switch_separate() {
    let a = short('a').switch();
    let parser = a.many::<Vec<_>>().to_options();

    let r = parser.run_inner(["-a", "-a"]).unwrap();
    assert_eq!(r, [true, true]);
}

#[test]
fn many_switch_merged() {
    let a = short('a').switch();
    let parser = a.many::<Vec<_>>().to_options();

    let r = parser.run_inner(["-aa"]).unwrap();
    assert_eq!(r, [true, true]);
}

#[test]
fn simple_switch() {
    let parser = short('a').switch().to_options();
    let r = parser.run_inner(["-a"]).unwrap();
    assert!(r);
}
