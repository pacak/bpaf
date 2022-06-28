use crate::*;
use std::str::FromStr;

#[test]
fn construct_with_fn() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Opts {
        a: bool,
        b: bool,
        c: bool,
    }

    fn a() -> Parser<bool> {
        short('a').switch()
    }

    let b = short('b').switch();

    fn c() -> Parser<bool> {
        short('c').switch()
    }

    let parser = Info::default().for_parser(construct!(Opts { a(), b, c() }));
    let help = parser
        .clone()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [-a] [-b] [-c]

Available options:
    -a
    -b
    -c
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);

    assert_eq!(
        Opts {
            a: false,
            b: true,
            c: true
        },
        parser.run_inner(Args::from(&["-b", "-c"])).unwrap()
    );
}

#[test]
fn simple_two_optional_flags() {
    let a = short('a').long("AAAAA").switch();
    let b = short('b').switch();
    let x = construct!(a, b);
    let info = Info::default().descr("this is a test");
    let decorated = info.for_parser(x);

    // no version information given - no version field generated
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-V"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-V is not expected in this context", err);

    // flag can be given only once
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-a is not expected in this context", err);

    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
this is a test

Usage: [-a] [-b]

Available options:
    -a, --AAAAA
    -b
    -h, --help    Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn simple_two_optional_flags_with_one_hidden() {
    let a = short('a').long("AAAAA").switch();
    let b = short('b').switch().hide();
    let x = construct!(a, b);
    let info = Info::default().descr("this is a test");
    let decorated = info.for_parser(x);

    // no version information given - no version field generated
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-V"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-V is not expected in this context", err);

    // flag can be given only once
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-a is not expected in this context", err);

    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
this is a test

Usage: [-a]

Available options:
    -a, --AAAAA
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
        .run_inner(Args::from(&["-V"]))
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
    -V, --version   Prints version information
";
    assert_eq!(expected_help, help);

    // must specify one of the required flags
    let err = decorated
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        "Expected (-a | -b | -c), pass --help for usage information",
        err
    );
}

#[test]
fn either_of_three_required_flags2() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let p = construct!([a, b, c]);
    let info = Info::default().version("1.0");
    let decorated = info.for_parser(p);

    // version is specified - version help is present
    let ver = decorated
        .clone()
        .run_inner(Args::from(&["-V"]))
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
    -V, --version   Prints version information
";
    assert_eq!(expected_help, help);

    // must specify one of the required flags
    let err = decorated
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        "Expected (-a | -b | -c), pass --help for usage information",
        err
    );
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
        .run_inner(Args::from(&["-V"]))
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
    -V, --version   Prints version information
";
    assert_eq!(expected_help, help);

    // fallback to default
    let res = decorated.run_inner(Args::from(&[])).unwrap();
    assert!(!res);
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
    let expected_err = "Expected -a ARG, pass --help for usage information";
    assert_eq!(expected_err, err);

    let err = decorated
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

    let p = construct!(a, b, c, d, e, f);
    let parser = Info::default().for_parser(p);

    let help = parser
        .clone()
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
    assert_eq!(
        "-a requires an argument, got flag -b",
        parser
            .clone()
            .run_inner(Args::from(&["-a", "-b"]))
            .unwrap_err()
            .unwrap_stderr()
    );

    drop(parser);
}

#[test]
fn group_help() {
    let a = short('a').help("flag A, related to B").switch();
    let b = short('b').help("flag B, related to A").switch();
    let c = short('c').help("flag C, unrelated").switch();
    let ab = construct!(a, b).group_help("Explanation applicable for both A and B");
    let parser = Info::default().for_parser(construct!(ab, c));

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

    let bar_cmd = command("bar", Some("do bar"), local_info.for_parser(bar));

    let parser = global_info.for_parser(bar_cmd);

    let help = parser
        .clone()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
This is global info

Usage: COMMAND ...

Available options:
    -h, --help   Prints help information

Available commands:
    bar  do bar
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(Args::from(&["bar", "--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
This is local info

Usage: [-b]

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
    parser.clone().run_inner(Args::from(&["-a"])).unwrap();
    parser.clone().run_inner(Args::from(&["-b"])).unwrap();
    parser.run_inner(Args::from(&["-c"])).unwrap();
}

#[test]
fn positional_argument() {
    let p = positional("FILE").group_help("File to process");
    let parser = Info::default().for_parser(p);

    let help = parser
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

mod git {
    use super::*;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Opt {
        Fetch {
            dry_run: bool,
            all: bool,
            repository: String,
        },
        Add {
            interactive: bool,
            all: bool,
            files: Vec<String>,
        },
    }

    fn setup() -> info::OptionParser<Opt> {
        let dry_run = long("dry_run").switch();
        let all = long("all").switch();
        let repository = positional("SRC").fallback("origin".to_string());
        let fetch = construct!(Opt::Fetch {
            dry_run,
            all,
            repository
        });
        let fetch_info = Info::default().descr("fetches branches from remote repository");
        let fetch_cmd = command(
            "fetch",
            Some("fetch branches from remote repository"),
            fetch_info.for_parser(fetch),
        );

        let interactive = short('i').switch();
        let all = long("all").switch();
        let files = positional("FILE").many();
        let add = construct!(Opt::Add {
            interactive,
            all,
            files
        });
        let add_info = Info::default().descr("add files to the staging area");
        let add_cmd = command(
            "add",
            Some("add files to the staging area"),
            add_info.for_parser(add),
        );

        Info::default()
            .descr("The stupid content tracker")
            .for_parser(fetch_cmd.or_else(add_cmd))
    }

    #[test]
    fn no_command() {
        let parser = setup();

        let expected_err = "Expected COMMAND ..., pass --help for usage information";
        assert_eq!(
            expected_err,
            parser
                .run_inner(Args::from(&[]))
                .unwrap_err()
                .unwrap_stderr()
        );
    }

    #[test]
    fn root_help() {
        let parser = setup();
        let expected_help = "\
The stupid content tracker

Usage: COMMAND ...

Available options:
    -h, --help   Prints help information

Available commands:
    fetch  fetch branches from remote repository
    add    add files to the staging area
";

        assert_eq!(
            expected_help,
            parser
                .run_inner(Args::from(&["--help"]))
                .unwrap_err()
                .unwrap_stdout()
        );
    }

    #[test]
    fn fetch_help() {
        let parser = setup();
        let expected_help = "\
fetches branches from remote repository

Usage: [--dry_run] [--all] [<SRC>]

Available options:
        --dry_run
        --all
    -h, --help      Prints help information
";
        assert_eq!(
            expected_help,
            parser
                .run_inner(Args::from(&["fetch", "--help"]))
                .unwrap_err()
                .unwrap_stdout()
        );
    }

    #[test]
    fn add_help() {
        let parser = setup();
        let expected_help = "\
add files to the staging area

Usage: [-i] [--all] <FILE>...

Available options:
    -i
        --all
    -h, --help   Prints help information
";
        assert_eq!(
            expected_help,
            parser
                .run_inner(Args::from(&["add", "--help"]))
                .unwrap_err()
                .unwrap_stdout()
        );
    }
}

#[test]
fn arg_bench() {
    use std::path::PathBuf;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct AppArgs {
        number: u32,
        opt_number: Option<u32>,
        width: u32,
        input: Vec<PathBuf>,
    }

    let number = long("number")
        .help("Sets a number")
        .argument("number")
        .from_str();

    let opt_number = long("opt-number")
        .help("Sets an optional number")
        .argument("opt-number")
        .from_str()
        .optional();

    let width = long("width")
        .help("Sets width")
        .argument("width")
        .from_str()
        .guard(|n: &u32| *n > 0, "Width must be positive")
        .fallback(10);

    let input = positional_os("INPUT").map(PathBuf::from).many();

    let parser = construct!(AppArgs {
        number,
        opt_number,
        width,
        input
    });

    let parser = Info::default().for_parser(parser);

    assert_eq!(
        AppArgs {
            number: 42,
            opt_number: None,
            width: 10,
            input: vec![PathBuf::from("foo"), PathBuf::from("foo2")],
        },
        parser
            .clone()
            .run_inner(Args::from(&["--number", "42", "foo", "foo2"]))
            .unwrap()
    );

    assert_eq!(
        AppArgs {
            number: 42,
            opt_number: None,
            width: 10,
            input: Vec::new()
        },
        parser
            .clone()
            .run_inner(Args::from(&["--number", "42"]))
            .unwrap()
    );

    drop(parser);
}

#[test]
fn simple_cargo_helper() {
    let a = short('a').long("AAAAA").switch();
    let b = short('b').switch();
    let parser = construct!(a, b);
    let info = Info::default().descr("this is a test");
    let decorated = info.for_parser(cargo_helper("simple", parser));

    // cargo run variant
    let ok = decorated.clone().run_inner(Args::from(&["-a"])).unwrap();
    assert_eq!((true, false), ok);

    // cargo simple variant
    let ok = decorated
        .clone()
        .run_inner(Args::from(&["simple", "-b"]))
        .unwrap();
    assert_eq!((false, true), ok);

    // flag can be given only once
    let err = decorated
        .clone()
        .run_inner(Args::from(&["-a", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-a is not expected in this context", err);

    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
this is a test

Usage: [-a] [-b]

Available options:
    -a, --AAAAA
    -b
    -h, --help    Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn long_path_in_construct() {
    let a = short('a').switch();
    let _ = construct!(std::option::Option::Some(a));

    let b = short('b').switch();
    let _ = construct!(::std::option::Option::Some(b));
}

#[test]
fn helpful_error_message() {
    let a = positional("FOO").some("You need to specify at least one FOO");
    let parser = Info::default().for_parser(a);

    let err = parser
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("You need to specify at least one FOO", err);
}

/*
#[test]
fn help_with_default_parse() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options)]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(String),

        #[bpaf(default)]
        NoAction,
    }

    let help = action()
        .run_inner(bpaf::Args::from(&["add", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "Add a new TODO item\n\nUsage: <ARG>\n\nAvailable options:\n    -h, --help   Prints help information\n";
    assert_eq!(help, expected_help);
}*/
