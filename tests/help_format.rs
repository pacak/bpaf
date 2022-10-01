use bpaf::*;

#[test]
fn decorations() {
    let p = short('p')
        .long("parser")
        .env("BPAF_VARIABLE")
        .argument::<String>("ARG")
        .to_options()
        .descr("descr\ndescr")
        .header("header\nheader")
        .footer("footer\nfooter")
        .version("version")
        .usage("custom {usage}");

    let r = p
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
descr
descr

custom -p ARG

header
header

Available options:
    -p, --parser <ARG>  [env:BPAF_VARIABLE: N/A]
    -h, --help          Prints help information
    -V, --version       Prints version information

footer
footer
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c").switch();
    let c2 = short('c').help("c").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_dif_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c1").switch();
    let c2 = short('c').help("c2").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c1
    -b
    -c          c2
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_pos_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C");
    let c2 = positional::<String>("C").help("C");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a <C> | -b <C>)

Available positional items:
    <C>  C

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_pos_items_diff_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C1");
    let c2 = positional::<String>("C").help("C2");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (-a <C> | -b <C>)

Available positional items:
    <C>  C1
    <C>  C2

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}
