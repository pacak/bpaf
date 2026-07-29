use bpaf::*;

// // no Doc
// #[test]
// fn custom_usage_override_with_fn() {
//     let parser = short('p').switch().to_options().with_usage(|b| {
//         let mut buf = Doc::default();
//         buf.text("Usage: hey ");
//         buf.doc(&b);
//         buf
//     });
//     let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
//     assert_eq!(
//         r,
//         "Usage: hey [-p]\n\nAvailable options:\n    -p\n    -h, --help  Prints help information\n"
//     );
// }

// I don't think that's possible - .adjacent by itself is replaced with .nest()
// but it can't possibly handle `-a=-20` since that's two items
#[test]
fn fancy_negative() {
    let a = short('a').argument::<i32>("N").negative_lit();

    let c = short('c').argument::<usize>("C").fallback(42);

    let parser = construct!(a, c).to_options();

    let r = parser.run_inner("-a -10").unwrap();
    assert_eq!(r, (-10, 42));

    let r = parser.run_inner("-a -c").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '-c', try '-a=-c' to use it as an argument\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=-20 -c 110").unwrap();
    assert_eq!(r, (-20, 110));

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app -a=N [-c=C]

Available options:
    -a=N
    -c=C
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

// // .adjacent() is gone
// #[test]
// fn adjacent_anywhere_needs_to_consume_something() {
//     let a = short('a').switch();
//     let b = short('b').switch();
//     let parser = construct!(a, b).adjacent().to_options();
//
//     let r = parser.run_inner(&["-a"]).unwrap();
//     assert_eq!(r, (true, false));
//
//     let r = parser.run_inner(&["-b"]).unwrap();
//     assert_eq!(r, (false, true));
// }

// flag like commands are not really a thing.
// But you can get a top level variant :
#[test]
fn flag_like_commands() {
    let a = long("add").nest(short('a').req_flag(1));
    let b = short('b').req_flag(2).to_options().command("remove");
    let parser = a.or_else(b).to_options();

    let r = parser.run_inner("--add -a").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("remove -b").unwrap();
    assert_eq!(r, 2);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "Usage: app (--add {-a} | COMMAND ...)

Available options:
        --add -a
    -a
    -h, --help  Prints help information

Available commands:
    remove
";

    assert_eq!(r, expected);
}
