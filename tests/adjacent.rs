use bpaf::*;

#[test]
fn test_adjacent() {
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

#[test]
fn adjacent_error_message_pos_single() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent();
    let parser = construct!(adj, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "10"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Expected <C>, pass --help for usage information");
}

#[test]
fn adjacent_error_message_arg_single() {
    let a = short('a').req_flag(());
    let b = short('b').argument::<usize>("B");
    let c = short('c').argument::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent();
    let parser = construct!(adj, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "10"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Expected -b <B>, got \"10\". Pass --help for usage information"
    );
}

#[test]
fn adjacent_error_message_pos_many() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent().many();
    let parser = construct!(adj, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "10"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Expected <C>, got \"-a\". Pass --help for usage information"
    );
}

#[test]
fn adjacent_error_message_arg_many() {
    let a = short('a').req_flag(());
    let b = short('b').argument::<usize>("B");
    let c = short('c').argument::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent().many();
    let parser = construct!(adj, d).to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "10"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "Expected -b <B>, got \"-a\". Pass --help for usage information"
    );
}
