use crate::*;
use std::str::FromStr;

#[test]
fn simple_two_optional_flags() {
    let a = short('a').long("AAAAA").switch().build();
    let b = short('b').switch().build();
    let x = tuple!(a, b);
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
Usage: [-a] [-b]
this is a test

Available options:
    -a, --AAAAA
    -b
    -h, --help    Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn either_of_three_required_flags() {
    let a = short('a').req_switch().build();
    let b = short('b').req_switch().build();
    let c = short('c').req_switch().build();
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
Usage: (-a | -b | -c)
Available options:
    -a
    -b
    -c
    -h, --help      Prints help information
    -v, --version   Prints version information
";
    assert_eq!(expected_help, help);

    // must specify one of the required flags
    let err = run_inner(Args::from(&[]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("Expected one of -a, -b, -c", err);
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
Usage: [-a | -b | [-c]]
Available options:
    -a
    -b
    -c
    -h, --help      Prints help information
    -v, --version   Prints version information
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

    let help = run_inner(Args::from(&["-h"]), decorated.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a ARG]
Available options:
    -a  <ARG>
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);

    let err = run_inner(Args::from(&["-a", "x12"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Couldn't parse \"x12\": invalid digit found in string";
    assert_eq!(expected_err, err);

    let err = run_inner(Args::from(&["-a"]), decorated)
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "-a requires an argument";
    assert_eq!(expected_err, err);
}

#[test]
fn parse_errors() {
    let a = short('a').argument().build().parse(|s| i32::from_str(&s));
    let decorated = Info::default().for_parser(a);

    let err = run_inner(Args::from(&["-a", "123x"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Couldn't parse \"123x\": invalid digit found in string";
    assert_eq!(expected_err, err);

    let err = run_inner(Args::from(&["-b", "123x"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Expected -a ARG";
    assert_eq!(expected_err, err);

    let err = run_inner(Args::from(&["-a", "123", "-b"]), decorated.clone())
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "-b is not expected in this context";
    assert_eq!(expected_err, err);
}

#[test]
fn long_usage_string() {
    let a = short('a').long("a-very-long-flag-with").argument().build();
    let b = short('b').long("b-very-long-flag-with").argument().build();
    let c = short('c').long("c-very-long-flag-with").argument().build();
    let d = short('d').long("d-very-long-flag-with").argument().build();
    let e = short('e').long("e-very-long-flag-with").argument().build();
    let f = short('f').long("f-very-long-flag-with").argument().build();

    let p = tuple!(a, b, c, d, e, f);
    let decorated = Info::default().for_parser(p);

    let help = run_inner(Args::from(&["--help"]), decorated)
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: -a ARG -b ARG -c ARG -d ARG -e ARG -f ARG
Available options:
    -a, --a-very-long-flag-with <ARG>
    -b, --b-very-long-flag-with <ARG>
    -c, --c-very-long-flag-with <ARG>
    -d, --d-very-long-flag-with <ARG>
    -e, --e-very-long-flag-with <ARG>
    -f, --f-very-long-flag-with <ARG>
    -h, --help                         Prints help information
";

    assert_eq!(expected_help, help);
}

#[test]
fn group_help() {
    let a = short('a').help("flag A, related to B").switch().build();
    let b = short('b').help("flag B, related to A").switch().build();
    let c = short('c').help("flag C, unrelated").switch().build();
    let ab = tuple!(a, b).help("Explanation applicable for both A and B");
    let parser = Info::default().for_parser(tuple!(ab, c));

    let help = run_inner(Args::from(&["--help"]), parser)
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a] [-b] [-c]
Available options:
                 Explanation applicable for both A and B
    -a           flag A, related to B
    -b           flag B, related to A

    -c           flag C, unrelated
    -h, --help   Prints help information
";

    assert_eq!(expected_help, help);
}

#[test]
fn from_several_alternatives_pick_more_meaningful() {
    let a = short('a').req_switch().build();
    let b = short('b').req_switch().build();
    let c = short('c').req_switch().build();
    let p = a.or_else(b).or_else(c);
    let parser = Info::default().for_parser(p);

    let err1 = run_inner(Args::from(&["-a", "-b"]), parser.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err1, "-b is not expected in this context");

    let err2 = run_inner(Args::from(&["-b", "-a"]), parser.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err2, "-a is not expected in this context");

    let err3 = run_inner(Args::from(&["-c", "-a"]), parser.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err3, "-a is not expected in this context");

    let err4 = run_inner(Args::from(&["-a", "-c"]), parser.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err4, "-c is not expected in this context");

    let err5 = run_inner(Args::from(&["-c", "-b", "-a"]), parser.clone())
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err5, "-b is not expected in this context");
}

#[test]
fn subcommands() {
    let global_info = Info::default().descr("This is global info");
    let local_info = Info::default().descr("This is local info");

    let bar = short('b').switch().build();

    let bar_cmd = command("bar", "do bar", local_info.for_parser(bar));

    let parser = global_info.for_parser(bar_cmd);

    let help = run_inner(Args::from(&["--help"]), parser.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: COMMAND
This is global info

Available options:
    -h, --help   Prints help information

Available commands:
    bar  do bar
";
    assert_eq!(expected_help, help);

    let help = run_inner(Args::from(&["bar", "--help"]), parser.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-b]
This is local info

Available options:
    -b
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn multiple_aliases() {
    let a = short('a').short('b').short('c').req_switch().build();
    let parser = Info::default().for_parser(a);

    let help = run_inner(Args::from(&["--help"]), parser.clone())
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: -a
Available options:
    -a
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
    assert_eq!(
        run_inner(Args::from(&["-a"]), parser.clone()).unwrap(),
        true
    );
    assert_eq!(
        run_inner(Args::from(&["-b"]), parser.clone()).unwrap(),
        true
    );
    assert_eq!(
        run_inner(Args::from(&["-c"]), parser.clone()).unwrap(),
        true
    );
}
