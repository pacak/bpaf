use bpaf::*;

#[test]
fn this_or_that_odd() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let ab = construct!(a, b);
    let a = short('a').req_flag(());
    let c = short('c').req_flag(());
    let cd = construct!(a, c);
    let parser = construct!([ab, cd]).to_options();

    let res = parser
        .run_inner(Args::from(&["-a", "-b", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`-c` cannot be used at the same time as `-b`");
}

#[test]
fn no_argument() {
    let a = short('a').argument::<i32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-42"])).unwrap();
    assert_eq!(r, (-42, false));

    //    let r = parser.run_inner(Args::from(&["-a", "-4"])).unwrap();
    //    assert_eq!(r, (-4, flse));
    let r = parser
        .run_inner(Args::from(&["-a", "-2"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`-a` requires an argument `N`, got a flag `-2`, try `-a=-2` to use it as an argument"
    );
}

#[test]
fn cannot_be_used_partial_arg() {
    let a = short('a').req_flag(10);
    let b = short('b').argument::<usize>("ARG");
    let parser = construct!([a, b]).to_options();

    // TODO - error message can be improved...
    let res = parser
        .run_inner(Args::from(&["-b", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`-b` is not expected in this context");

    let res = parser
        .run_inner(Args::from(&["-a", "-b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`-b` is not expected in this context");
}

#[test]
fn better_error_with_enum() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options)]
    enum Foo {
        Alpha,
        Beta,
        Gamma,
    }

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--beta"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "`--gamma` cannot be used at the same time as `--alpha`"
    );

    let res = foo()
        .run_inner(Args::from(&["--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`--gamma` cannot be used at the same time as `--beta`");

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");
}

#[test]
fn guard_message() {
    let parser = short('a')
        .argument::<u32>("N")
        .guard(|n| *n <= 10u32, "too high")
        .to_options();

    let res = parser
        .run_inner(Args::from(&["-a", "30"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(res, "`30`: too high");
}

#[test]
fn cannot_be_used_multiple_times() {
    let parser = short('a').switch().to_options();

    let r = parser
        .run_inner(Args::from(&["-aaa"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected = "`-a` is not expected in this context";
    assert_eq!(r, expected);

    // TODO - improve error message
    let r = parser
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected = "`-a` is not expected in this context";
    assert_eq!(r, expected);
}

#[test]
fn strict_positional_argument() {
    let a = short('a').argument::<usize>("N");
    let parser = a.to_options();

    let r = parser
        .run_inner(Args::from(&["-a", "--", "10"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`-a` requires an argument `N`");
}
