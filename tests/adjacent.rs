#[test]
fn test_adjacent() {
    use bpaf::*;

    let a = short('a').req_flag(());
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c).adjacent().many().to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "-c", "-a", "-b", "-a", "-b", "-c"]))
        .unwrap();
    assert_eq!(r, &[((), false, true), ((), true, false), ((), true, true)]);
}
