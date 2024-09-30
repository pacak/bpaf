use bpaf_core::*;

#[test]
fn simple_help() {
    let a = short('a').long("alice").switch();
    let b = short('b').long("bob").argument::<usize>("BOB");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(["--help"]).unwrap_err().unwrap_stdout();

    let expected = "
Available options:
    -a, --alice
    -b, --bob=BOB
    -h, --help     Prints help information
    -V, --version  Prints version information
";
    assert_eq!(r, expected);
}
