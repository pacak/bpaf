use std::ffi::OsString;

use bpaf::*;

#[test]
fn get_any_simple() {
    let a = short('a').switch();
    let b = any::<OsString, _, _>("REST", Some).help("any help");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(Args::from(&["-b=foo", "-a"])).unwrap().1;
    assert_eq!(r, "-b=foo");
}

#[test]
fn get_any_many() {
    let a = short('a').switch();
    let b = any::<OsString, _, _>("REST", Some).help("any help").many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a", "-b"])).unwrap();
    assert_eq!(r.1, &["-b", "-b"]);
}

#[test]
fn get_any_many2() {
    let parser = any::<OsString, _, _>("REST", Some).many().to_options();

    let r = parser.run_inner(Args::from(&["-vvv"])).unwrap();
    assert_eq!(r[0], "-vvv");
}

#[test]
fn get_any_magic() {
    let parser = short('b')
        .argument::<String>("anana")
        .adjacent()
        .guard(|b| b == "anana", "not anana")
        .optional()
        .catch()
        .map(|b| b.is_some())
        .to_options();

    // -b anana - isn't a -banana
    let r = parser
        .run_inner(Args::from(&["-b", "anana"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`-b` is not expected in this context");

    // close enough :)
    assert!(parser.run_inner(Args::from(&["-b=anana"])).unwrap());

    assert!(parser.run_inner(Args::from(&["-banana"])).unwrap());
    assert!(!parser.run_inner(Args::from(&[])).unwrap());
}

#[test]
fn from_str_works_with_parse() {
    use std::str::FromStr;
    let parser = positional::<String>("A")
        .parse(|s| usize::from_str(&s))
        .to_options();

    let r = parser.run_inner(Args::from(&["42"])).unwrap();
    assert_eq!(r, 42);
}
