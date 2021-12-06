use crate::*;
use std::str::FromStr;

#[test]
fn simple_two_optional_flags() {
    let a = short('a').long("AAAAA").switch();
    let b = short('b').switch();
    let x = tuple!(a, b);
    let info = Info::default().descr("this is a test");
    let decorated = info.for_parser(x);

    // no version information given - no version field generated
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-v"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-v is not expected in this context", err);

    // flag can be given only once
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-a is not expected in this context", err);

    let help = decorated
        .clone()
        .run_inner(Args::from(&["-h"]))
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
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let p = a.or_else(b).or_else(c);
    let info = Info::default().version("1.0");
    let decorated = info.for_parser(p);

    // version is specified - version help is present
    let ver = decorated
        .clone()
        .run_inner(Args::from(&["-v"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = decorated
        .clone()
        .run_inner(Args::from(&["-h"]))
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
    let err = decorated
        .clone()
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("Expected one of -a, -b, -c", err);
}

#[test]
fn either_of_two_required_flags_and_one_optional() {
    let a = short('a').req_flag(true);
    let b = short('b').req_flag(false);
    let c = short('c').switch();
    let p = a.or_else(b).or_else(c);
    let info = Info::default().version("1.0");
    let decorated = info.for_parser(p);

    // version is specified - version help is present
    let ver = decorated
        .clone()
        .run_inner(Args::from(&["-v"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = decorated
        .clone()
        .run_inner(Args::from(&["-h"]))
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
    let res = decorated.clone().run_inner(Args::from(&[])).unwrap();
    assert_eq!(res, false);
}

#[test]
fn default_arguments() {
    let a = short('a')
        .argument("ARG")
        .parse(|s| i32::from_str(&s))
        .fallback(42);
    let info = Info::default();
    let decorated = info.for_parser(a);

    let help = decorated
        .clone()
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a ARG]
Available options:
    -a  <ARG>
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);

    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "x12"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Couldn't parse \"x12\": invalid digit found in string";
    assert_eq!(expected_err, err);

    let err = decorated
        .run_inner(Args::from(&["-a"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "-a requires an argument";
    assert_eq!(expected_err, err);
}

#[test]
fn parse_errors() {
    let a = short('a').argument("ARG").parse(|s| i32::from_str(&s));
    let decorated = Info::default().for_parser(a);

    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "123x"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Couldn't parse \"123x\": invalid digit found in string";
    assert_eq!(expected_err, err);

    let err = decorated
        .clone()
        .run_inner(Args::from(&["-b", "123x"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Expected -a ARG";
    assert_eq!(expected_err, err);

    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "123", "-b"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "-b is not expected in this context";
    assert_eq!(expected_err, err);
}

#[test]
fn long_usage_string() {
    let a = short('a').long("a-very-long-flag-with").argument("ARG");
    let b = short('b').long("b-very-long-flag-with").argument("ARG");
    let c = short('c').long("c-very-long-flag-with").argument("ARG");
    let d = short('d').long("d-very-long-flag-with").argument("ARG");
    let e = short('e').long("e-very-long-flag-with").argument("ARG");
    let f = short('f').long("f-very-long-flag-with").argument("ARG");

    let p = tuple!(a, b, c, d, e, f);
    let decorated = Info::default().for_parser(p);

    let help = decorated
        .run_inner(Args::from(&["--help"]))
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
    let a = short('a').help("flag A, related to B").switch();
    let b = short('b').help("flag B, related to A").switch();
    let c = short('c').help("flag C, unrelated").switch();
    let ab = tuple!(a, b).help("Explanation applicable for both A and B");
    let parser = Info::default().for_parser(tuple!(ab, c));

    let help = parser
        .run_inner(Args::from(&["--help"]))
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
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let p = a.or_else(b).or_else(c);
    let parser = Info::default().for_parser(p);

    let err1 = parser
        .clone()
        .run_inner(Args::from(&["-a", "-b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err1, "-b is not expected in this context");

    let err2 = parser
        .clone()
        .run_inner(Args::from(&["-b", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err2, "-a is not expected in this context");

    let err3 = parser
        .clone()
        .run_inner(Args::from(&["-c", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err3, "-a is not expected in this context");

    let err4 = parser
        .clone()
        .run_inner(Args::from(&["-a", "-c"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err4, "-c is not expected in this context");

    let err5 = parser
        .clone()
        .run_inner(Args::from(&["-c", "-b", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err5, "-b is not expected in this context");
}

#[test]
fn subcommands() {
    let global_info = Info::default().descr("This is global info");
    let local_info = Info::default().descr("This is local info");

    let bar = short('b').switch();

    let bar_cmd = command("bar", "do bar", local_info.for_parser(bar));

    let parser = global_info.for_parser(bar_cmd);

    let help = parser
        .clone()
        .run_inner(Args::from(&["--help"]))
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

    let help = parser
        .clone()
        .run_inner(Args::from(&["bar", "--help"]))
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
    let a = short('a').short('b').short('c').req_flag(());
    let parser = Info::default().for_parser(a);

    let help = parser
        .clone()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: -a
Available options:
    -a
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
    assert_eq!(parser.clone().run_inner(Args::from(&["-a"])).unwrap(), ());
    assert_eq!(parser.clone().run_inner(Args::from(&["-b"])).unwrap(), ());
    assert_eq!(parser.clone().run_inner(Args::from(&["-c"])).unwrap(), ());
}

#[test]
fn positional_argument() {
    let p = positional("FILE").help("File to process");
    let parser = Info::default().for_parser(p);

    let help = parser
        .clone()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: <FILE>
Available options:
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
}
