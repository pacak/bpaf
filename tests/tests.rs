use bpaf::*;

// no Doc
#[test]
fn custom_usage_override_with_fn() {
    let parser = short('p').switch().to_options().with_usage(|b| {
        let mut buf = Doc::default();
        buf.text("Usage: hey ");
        buf.doc(&b);
        buf
    });
    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: hey [-p]\n\nAvailable options:\n    -p\n    -h, --help  Prints help information\n"
    );
}

// I don't think that's possible - .adjacent by itself is replaced with .nest()
// but it can't possibly handle `-a=-20` since that's two items
#[test]
fn fancy_negative() {
    let a = short('a').req_flag(());
    #[allow(clippy::redundant_closure)]
    let b = any("A", |i: i32| Some(i));
    let ab = construct!(a, b).adjacent().map(|x| x.1);

    let c = short('c').argument::<usize>("C").fallback(42);

    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(&["-a", "-10"]).unwrap();
    assert_eq!(r, (-10, 42));

    let r = parser.run_inner(&["-a=-20", "-c", "110"]).unwrap();
    assert_eq!(r, (-20, 110));

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    // TODO - rendering sucks once you start inventing fancy combinations and don't provide help...
    let expected = "\
Usage: -a A [-c=C]

Available options:
  -a A

    -c=C
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

// .adjacent() is gone
#[test]
fn adjacent_anywhere_needs_to_consume_something() {
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = construct!(a, b).adjacent().to_options();

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner(&["-b"]).unwrap();
    assert_eq!(r, (false, true));
}

// currently such commands are not possible - it's not a literal. Need to make a new adapter for
// that
#[test]
fn flag_like_commands() {
    let a = short('a').req_flag(1).to_options().command("--add");
    let b = short('b').req_flag(2).to_options().command("remove");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner(&["--add", "-a"]).unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner(&["remove", "-b"]).unwrap();
    assert_eq!(r, 2);

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "Usage: COMMAND ...\n\nAvailable options:\n    -h, --help  Prints help information\n\nAvailable commands:\n    --add\n    remove\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(&["--add", "--help"])
        .unwrap_err()
        .unwrap_stdout();
    let expected =
        "Usage: --add -a\n\nAvailable options:\n    -a\n    -h, --help  Prints help information\n";
    assert_eq!(r, expected);
}
