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
    assert_eq!(
        res,
        "--beta is not expected in this context: --beta cannot be used at the same time as --alpha"
    );

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "--gamma is not expected in this context: --gamma cannot be used at the same time as --alpha");

    let res = foo()
        .run_inner(Args::from(&["--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "--gamma is not expected in this context: --gamma cannot be used at the same time as --beta");

    let res = foo()
        .run_inner(Args::from(&["--alpha", "--beta", "--gamma"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "--beta is not expected in this context: --beta cannot be used at the same time as --alpha"
    );
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
