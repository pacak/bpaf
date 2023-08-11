use std::ffi::OsString;

use bpaf::*;

#[test]
fn get_any_simple() {
    let a = short('a').switch();
    let b = any("REST", Some).help("any help");
    let parser: OptionParser<(bool, OsString)> = construct!(a, b).to_options();

    let r = parser.run_inner(&["-a", "-b"]).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(&["-b", "-a"]).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(&["-b=foo", "-a"]).unwrap().1;
    assert_eq!(r, "-b=foo");
}

#[test]
fn get_any_many() {
    let a = short('a').switch();
    let b = any("REST", Some).help("any help").many();
    let parser: OptionParser<(bool, Vec<OsString>)> = construct!(a, b).to_options();

    let r = parser.run_inner(&["-a", "-b"]).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(&["-b", "-a"]).unwrap();
    assert_eq!(r.1, &["-b"]);

    let r = parser.run_inner(&["-b", "-a", "-b"]).unwrap();
    assert_eq!(r.1, &["-b", "-b"]);
}

#[test]
fn get_any_many2() {
    let parser: OptionParser<Vec<OsString>> = any("REST", Some).many().to_options();

    let r = parser.run_inner(&["-vvv"]).unwrap();
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
        .run_inner(&["-b", "anana"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`-b` is not expected in this context");

    // close enough :)
    assert!(parser.run_inner(&["-b=anana"]).unwrap());

    assert!(parser.run_inner(&["-banana"]).unwrap());
    assert!(!parser.run_inner(&[]).unwrap());
}

#[test]
fn from_str_works_with_parse() {
    use std::str::FromStr;
    let parser = positional::<String>("A")
        .parse(|s| usize::from_str(&s))
        .to_options();

    let r = parser.run_inner(&["42"]).unwrap();
    assert_eq!(r, 42);
}

#[test]
fn squashed_short_names() {
    let a = short('a').switch();
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).to_options();

    // For now instead of supporting I parse "-Obits=2048" instead as "-O" with "bits=2048" body
    // https://github.com/pacak/bpaf/issues/121
    //
    // let r = parser.run_inner(&["-ab=foo"]).unwrap();
    // assert_eq!(r.1, "foo");

    // an odd one, short flag a is squashed with short argument b
    let r = parser.run_inner(&["-ab", "foo"]).unwrap();
    assert_eq!(r.1, "foo");

    // same, but body for b is adjacent as well
    let r = parser.run_inner(&["-abfoo"]).unwrap();
    assert_eq!(r.1, "foo");

    // normal
    let r = parser.run_inner(&["-a", "-bfoo"]).unwrap();
    assert_eq!(r.1, "foo");
}
