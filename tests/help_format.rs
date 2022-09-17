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
