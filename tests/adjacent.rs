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

    let r = parser.run_inner(Args::from(&[])).unwrap();
    assert_eq!(r, &[]);
}

#[test]
fn test_adjacent_prefix() {
    use bpaf::*;
    let a = short('a').req_flag(());
    let b = positional::<usize>("X");
    let ab = construct!(a, b).adjacent().optional();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(Args::from(&["-c"])).unwrap();
    assert_eq!(r, (None, true));

    let r = parser.run_inner(Args::from(&["-a", "1"])).unwrap();
    assert_eq!(r, (Some(((), 1)), false));

    let r = parser.run_inner(Args::from(&["-c", "-a", "1"])).unwrap();
    assert_eq!(r, (Some(((), 1)), true));

    let r = parser.run_inner(Args::from(&["-a", "1", "-c"])).unwrap();
    assert_eq!(r, (Some(((), 1)), true));
}
