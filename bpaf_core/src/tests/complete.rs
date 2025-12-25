use crate::{Parser, construct, long, positional, pure, short};

#[test]
fn simple_complete_command() {
    let a = short('a').req_flag('a').to_options().command("alpha");
    let b = short('b').req_flag('b');
    let c = short('c').switch();
    let ab = construct!([a, b]);
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    let expected = "-b (None)\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b -c", "")).unwrap_err().unwrap_stdout();
    let expected = "";
    assert_eq!(r, expected);

    let r = parser.run_inner(("alpha", "")).unwrap_err().unwrap_stdout();
    let expected = "-a (None)\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "alpha (None)\n\
                    -b (None)\n\
                    -c (None)\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    let expected = "-c (None)\n";
    assert_eq!(r, expected);
}

#[test]
fn simple_complete_named() {
    let a = long("missy").req_flag('a');
    let b = long("missle-launcher").req_flag('b');
    let c = short('m').req_flag('c');
    let abc = construct!([a, b, c]);
    let name = long("name").argument::<String>("NAME");
    let parser = construct!(abc, name).to_options();

    let r = parser
        .run_inner(("--name=bob", "--missy"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "--missy (None)\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--name=bob", "--miss"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "--missle-launcher (None)\n--missy (None)\n";
    assert_eq!(r, expected);
}

#[test]
fn simple_complete_for_value() {
    let a = short('a').req_flag(());
    let b = short('b')
        .argument::<u32>("B")
        .complete(|_s| vec![("42".into(), None)]);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "42 (None)\n");

    let r = parser.run_inner(("-b=", "")).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse ``: cannot parse integer from empty string"
    );

    let r = parser.run_inner(("", "-b=")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "42 (None)\n");
}

#[test]
fn strict_pos_works() {
    let a = short('a').switch().help("short help");
    let b = positional::<u32>("X").help("pos help");
    let c = pure(()).to_options().descr("ket").command("ket");
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "\
ket (None)
-a (Some(\"short help\"))
Metavar(\"X\")";
    assert_eq!(r, expected);

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    let expected = "Metavar(\"X\")";
    assert_eq!(r, expected);
}
