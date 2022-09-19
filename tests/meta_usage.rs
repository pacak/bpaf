use bpaf::*;

#[track_caller]
fn assert_usage<T: std::fmt::Debug>(parser: OptionParser<T>, expected: &str) {
    let output = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let usage = output
        .lines()
        .next()
        .unwrap()
        .strip_prefix("Usage: ")
        .unwrap();
    assert_eq!(usage, expected);
}

#[test]
fn optional_group_meta() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).optional().to_options();

    assert_usage(parser, "[-a A -b B]");
}

#[test]
fn sensors_meta() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).many().to_options();

    assert_usage(parser, "(-a A -b B)...");
}

#[test]
fn optional_req_select() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).optional().to_options();

    assert_usage(parser, "[-a | -b]");
}

#[test]
fn single_optional_req_select() {
    let a = short('a').req_flag(());
    let parser = construct!([a]).optional().to_options();

    assert_usage(parser, "[-a]");
}

#[test]
fn fallback_req_select() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).fallback(()).to_options();

    assert_usage(parser, "[-a | -b]");
}

#[test]
fn single_fallback_req_select() {
    let a = short('a').req_flag(());
    let parser = construct!([a]).fallback(()).to_options();

    assert_usage(parser, "[-a]");
}

#[test]
fn optional_argument_select() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!([a, b]).optional().to_options();

    assert_usage(parser, "[-a A | -b B]");
}

#[test]
fn commands_no_fallback_meta() {
    let a = pure(()).to_options().command("a");
    let b = pure(()).to_options().command("b");
    let parser = construct!([a, b]).to_options();

    assert_usage(parser, "COMMAND ...");
}

#[test]
fn commands_and_fallback_meta() {
    let a = pure(()).to_options().command("a");
    let b = pure(()).to_options().command("b");
    let parser = construct!([a, b]).fallback(()).to_options();

    assert_usage(parser, "[COMMAND ...]");
}

#[test]
fn command_no_fallback() {
    let a = pure(()).to_options().command("a");
    let parser = a.to_options();

    assert_usage(parser, "COMMAND ...");
}

#[test]
fn command_and_fallback_meta() {
    let a = pure(()).to_options().command("a");
    let parser = a.fallback(()).to_options();

    assert_usage(parser, "[COMMAND ...]");
}

#[test]
fn requierd_select() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!([a, b]).to_options();

    assert_usage(parser, "(-a | -b)");
}

#[test]
fn requierd_and() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!(a, b).to_options();

    assert_usage(parser, "-a -b");
}

#[test]
fn required_or_and() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let d = short('d').req_flag(());
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!([ab, cd]).to_options();
    assert_usage(parser, "(-a -b | -c -d)");
}

#[test]
fn required_one_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!(a, b).many().to_options();
    assert_usage(parser, "(-a -b)...");
}

#[test]
fn optional_one_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let parser = construct!(a, b).optional().many().to_options();
    assert_usage(parser, "[-a -b]...");
}

#[test]
fn required_or_many() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let d = short('d').req_flag(());
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let e = pure(((), ()));
    let f = pure(((), ()));
    let ef = construct!([e, f]);
    let parser = construct!([ab, cd, ef]).many().to_options();
    assert_usage(parser, "(-a -b | -c -d)...");
}

#[test]
fn no_actual_arguments_also_works() {
    let parser = pure(true).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "\nAvailable options:\n    -h, --help  Prints help information\n"
    );

    let x = pure(true).meta();
    assert_eq!("no parameters expected", x.to_string());
}
