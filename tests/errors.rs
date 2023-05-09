use bpaf::*;

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
    assert_eq!(res, "--beta cannot be used at the same time as --alpha");

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "--gamma cannot be used at the same time as --alpha");

    let res = foo()
        .run_inner(Args::from(&["--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "--gamma cannot be used at the same time as --beta");

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "--beta cannot be used at the same time as --alpha");
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

    assert_eq!(res, "\"30\": too high");
}

#[test]
fn cannot_be_used_multiple_times() {
    let parser = short('a').switch().to_options();

    let r = parser
        .run_inner(Args::from(&["-aaa"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected = "-a is not expected in this context";
    assert_eq!(r, expected);

    // TODO - improve error message
    let r = parser
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected = "-a is not expected in this context";
    assert_eq!(r, expected);
}
