use crate::*;

#[test]
fn offset_on_flag() {
    let a = short('a').switch().offset();
    let parser = a.many().to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, &[(Some(0), true), (Some(1), true)]);

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, &[(Some(0), true), (Some(0), true)]);
}

#[test]
fn offset_on_arg() {
    let a = short('a').argument::<usize>("A").offset();
    let parser = a.many().to_options();

    let r = parser.run_inner("-a10 -a 20 -a=30").unwrap();
    assert_eq!(r, &[(Some(0), 10), (Some(1), 20), (Some(3), 30)]);
}

#[test]
fn ofset_on_pos() {
    let a = positional::<bool>("A").offset();
    let parser = a.to_options();

    let r = parser.run_inner("false").unwrap();
    assert_eq!(r, (Some(0), false));
}

#[test]
fn mix() {
    let a = long("alpha").switch().offset();
    let b = long("beta").argument::<usize>("B").offset();
    let c = positional::<bool>("B").offset();
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner("false --beta 12").unwrap();
    assert_eq!(r, ((None, false), (Some(1), 12), (Some(0), false)));
}
