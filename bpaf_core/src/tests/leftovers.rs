use crate::*;

#[test]
fn all_args() {
    let parser = leftovers::<String>().to_options();
    let r: Vec<String> = parser.run_inner("a b c").unwrap();
    assert_eq!(r, &["a", "b", "c"]);
}

#[test]
fn empty_input() {
    let parser = leftovers::<String>().to_options();
    let r: Vec<String> = parser.run_inner("").unwrap();
    let expected: Vec<String> = vec![];
    assert_eq!(r, expected);
}

#[test]
fn with_flag_ahead() {
    let a = short('a').switch();
    let parser = (a, leftovers::<String>()).to_options();
    let (r1, r2) = parser.run_inner("-a x y").unwrap();
    assert!(r1);
    assert_eq!(r2, &["x", "y"]);
}

#[test]
fn no_leftovers() {
    let a = short('a').switch();
    let parser = (a, leftovers::<String>()).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, (true, vec![]));
}

#[test]
fn after_dashdash() {
    let a = short('a').switch();
    let parser = (a, leftovers::<String>()).to_options();
    let (r1, r2) = parser.run_inner("-- -a").unwrap();
    assert!(!r1);
    assert_eq!(r2, &["-a"]);
}

#[test]
fn with_positional() {
    let p = positional::<String>("P");
    let parser = (p, leftovers::<String>()).to_options();
    let (r1, r2) = parser.run_inner("foo bar baz").unwrap();
    assert_eq!(r1, "foo");
    assert_eq!(r2, &["bar", "baz"]);
}

#[test]
fn in_optional() {
    let parser = leftovers().optional().to_options();
    let r: Option<Vec<String>> = parser.run_inner("").unwrap();
    assert_eq!(r, Some(vec![]));
}

#[test]
fn in_optional_with_args() {
    let parser = leftovers().optional().to_options();
    let r: Option<Vec<String>> = parser.run_inner("a b").unwrap();
    let r = r.unwrap();
    assert_eq!(r, &["a", "b"]);
}

#[test]
fn in_optional_after_flag() {
    let a = short('a').switch();
    let parser = (a, leftovers::<String>().optional()).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, (true, Some(vec![])));
}

#[test]
fn in_sum() {
    let empty = pure(Vec::<String>::new());
    let parser = leftovers::<String>().or_else(empty).to_options();
    let r = parser.run_inner("x").unwrap();
    assert_eq!(r, &["x"]);
}

#[test]
fn usage_output() {
    let parser = leftovers::<String>().to_options();
    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected =
        "Usage: app ...\n\nAvailable options:\n    -h, --help  Prints help information\n";
    assert_eq!(help, expected);
}

#[test]
fn usage_with_flags() {
    let a = short('a').switch();
    let parser = (a, leftovers::<String>()).to_options();
    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app [-a] ...\n\nAvailable options:\n    -a\n    -h, --help  Prints help information\n";
    assert_eq!(help, expected);
}
