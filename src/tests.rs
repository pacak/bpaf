use crate::*;
use std::str::FromStr;

#[test]
fn simple_two_optional_flags() {
    let mk = Parser::pure(curry!(|a, b| (a, b)));
    let a = short('a').long("AAAAA").switch().build();
    let b = short('b').switch().build();
    let x = mk.ap(a).ap(b);
    let info = Info::default().descr("this is a test");
    let decorated = info.for_parser(x);

    // no version information given - no version field generated
    let err = run_inner(Args::from(&["-a", "-v"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-v is not expected in this context", err);

    // flag can be given only once
    let err = run_inner(Args::from(&["-a", "-a"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-a is not expected in this context", err);

    let help = run_inner(Args::from(&["-h"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [-a|--AAAAA] [-b]
this is a test

Available options:
    -a, AAAAA
    -b
    -h, help    Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn either_of_three_required_flags() {
    let mk = Parser::pure(|a| a);
    let a = short('a').req_switch().build();
    let b = short('b').req_switch().build();
    let c = short('c').req_switch().build();
    let p = mk.ap(a.or_else(b).or_else(c));
    let info = Info::default().version("1.0");
    let decorated = info.for_parser(p);

    // version is specified - version help is present
    let ver = run_inner(Args::from(&["-v"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = run_inner(Args::from(&["-h"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: ((-a) | (-b) | (-c))
Available options:
    -a
    -b
    -c
    -h, help      Prints help information
    -v, version   Prints version information
";
    assert_eq!(expected_help, help);

    // must specify one of the required flags
    let err = run_inner(Args::from(&[]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("Expected one of (-a), (-b), (-c)", err);
}

#[test]
fn either_of_two_required_flags_and_one_optional() {
    let a = short('a').req_switch().build();
    let b = short('b').req_switch().build();
    let c = short('c').switch().build();
    let p = a.or_else(b).or_else(c);
    let info = Info::default().version("1.0");
    let decorated = info.for_parser(p);

    // version is specified - version help is present
    let ver = run_inner(Args::from(&["-v"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = run_inner(Args::from(&["-h"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [(-a) | (-b) | [-c]]
Available options:
    -a
    -b
    -c
    -h, help      Prints help information
    -v, version   Prints version information
";
    assert_eq!(expected_help, help);

    // fallback to default
    let res = run_inner(Args::from(&[]), decorated.clone()).unwrap();
    assert_eq!(res, false);
}

#[test]
fn default_arguments() {
    let a = short('a')
        .argument()
        .build()
        .parse(|s| i32::from_str(&s))
        .fallback(42);
    let info = Info::default();
    let decorated = info.for_parser(a);

    let help = run_inner(Args::from(&["-h"]), decorated)
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a]
Available options:
    -a
    -h, help   Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn parse_errors() {
    let a = short('a').argument().build().parse(|s| i32::from_str(&s));
    let decorated = Info::default().for_parser(a);

    let err = run_inner(Args::from(&["-a", "123x"]), decorated)
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "invalid digit found in string";
    assert_eq!(expected_err, err);
}
