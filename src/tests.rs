#![allow(deprecated)]
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

    fn a() -> impl Parser<bool> {
        short('a').switch()
    }

    let b = short('b').switch();

    fn c() -> impl Parser<bool> {
        short('c').switch()
    }

    let parser = construct!(Opts { a(), b, c() }).to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [-a] [-b] [-c]

Available options:
    -a
    -b
    -c
    -h, --help  Prints help information
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
    let decorated = x.to_options().descr("this is a test");

    // no version information given - no version field generated
    let err = decorated
        .run_inner(Args::from(&["-a", "-V"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-V is not expected in this context", err);

    // accept only one copy of -a
    let err = decorated
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
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn simple_two_optional_flags_with_one_hidden() {
    let a = short('a').long("AAAAA").switch();
    let b = short('b').switch().hide();
    let decorated = construct!(a, b).to_options().descr("this is a test");

    // no version information given - no version field generated
    let err = decorated
        .run_inner(Args::from(&["-a", "-V"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("-V is not expected in this context", err);

    // accepts only one copy of -a
    let err = decorated
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
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn either_of_three_required_flags() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let p = a.or_else(b).or_else(c);
    let decorated = p.to_options().version("1.0");

    // version help requires version meta
    let ver = decorated
        .run_inner(Args::from(&["-V"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: (-a | -b | -c)

Available options:
    -a
    -b
    -c
    -h, --help     Prints help information
    -V, --version  Prints version information
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
    let decorated = p.to_options().version("1.0");

    let ver = decorated
        .run_inner(Args::from(&["-V"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: (-a | -b | -c)

Available options:
    -a
    -b
    -c
    -h, --help     Prints help information
    -V, --version  Prints version information
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
    let decorated = p.to_options().version("1.0");

    let ver = decorated
        .run_inner(Args::from(&["-V"]))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!("Version: 1.0", ver);

    // help is always generated
    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: (-a | -b | [-c])

Available options:
    -a
    -b
    -c
    -h, --help     Prints help information
    -V, --version  Prints version information
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
    let decorated = a.to_options();

    let help = decorated
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a ARG]

Available options:
    -a <ARG>
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let err = decorated
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
    let decorated = short('a')
        .argument("ARG")
        .parse(|s| i32::from_str(&s))
        .to_options();

    let err = decorated
        .run_inner(Args::from(&["-a", "123x"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected_err = "Couldn't parse \"123x\": invalid digit found in string";
    assert_eq!(expected_err, err);

    let err = decorated
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
fn custom_usage() {
    let a = short('a').long("long").argument("ARG");
    let parser = a.to_options().usage("Usage: -a <ARG> or --long <ARG>");
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: -a <ARG> or --long <ARG>

Available options:
    -a, --long <ARG>
    -h, --help        Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn long_usage_string() {
    let a = short('a').long("a-very-long-flag-with").argument("ARG");
    let b = short('b').long("b-very-long-flag-with").argument("ARG");
    let c = short('c').long("c-very-long-flag-with").argument("ARG");
    let d = short('d').long("d-very-long-flag-with").argument("ARG");
    let e = short('e').long("e-very-long-flag-with").argument("ARG");
    let f = short('f').long("f-very-long-flag-with").argument("ARG");

    let parser = construct!(a, b, c, d, e, f).to_options();

    let help = parser
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
            .run_inner(Args::from(&["-a", "-b"]))
            .unwrap_err()
            .unwrap_stderr()
    );

    drop(parser);
}

#[test]
fn group_help_args() {
    let a = short('a').help("flag A, related to B").switch();
    let b = short('b').help("flag B, related to A").switch();
    let c = short('c').help("flag C, unrelated").switch();
    let ab = construct!(a, b).group_help("Explanation applicable for both A and B");
    let parser = construct!(ab, c).to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a] [-b] [-c]

Available options:
  Explanation applicable for both A and B
    -a          flag A, related to B
    -b          flag B, related to A

    -c          flag C, unrelated
    -h, --help  Prints help information
";

    assert_eq!(expected_help, help);
}

#[test]
fn group_help_commands() {
    let a = short('a')
        .switch()
        .to_options()
        .command("cmd_a")
        .help("command that does A");
    let b = short('a')
        .switch()
        .to_options()
        .command("cmd_b")
        .help("command that does B");
    let c = short('a')
        .switch()
        .to_options()
        .command("cmd_c")
        .help("command that does C");
    let parser = construct!([a, b]).group_help("Explanation applicable for both A and B");

    let parser = construct!([parser, c]).to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
  Explanation applicable for both A and B
    cmd_a  command that does A
    cmd_b  command that does B

    cmd_c  command that does C
";
    assert_eq!(expected_help, help);
}

#[test]
fn from_several_alternatives_pick_more_meaningful() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c = short('c').req_flag(());
    let parser = construct!([a, b, c]).to_options();

    let err1 = parser
        .run_inner(Args::from(&["-a", "-b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err1, "-b is not expected in this context");

    let err2 = parser
        .run_inner(Args::from(&["-b", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err2, "-a is not expected in this context");

    let err3 = parser
        .run_inner(Args::from(&["-c", "-a"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err3, "-a is not expected in this context");

    let err4 = parser
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
    let bar = short('b').switch();

    let bar_cmd = command("bar", bar.to_options().descr("This is local info"));

    let parser = bar_cmd.to_options().descr("This is global info");

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
This is global info

Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    bar  This is local info
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
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn multiple_aliases() {
    let a = short('a').short('b').short('c').req_flag(());
    let parser = a.to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: -a

Available options:
    -a
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
    parser.run_inner(Args::from(&["-a"])).unwrap();
    parser.run_inner(Args::from(&["-b"])).unwrap();
    parser.run_inner(Args::from(&["-c"])).unwrap();
}

#[test]
fn positional_argument() {
    let p = positional("FILE").group_help("File to process");
    let parser = p.to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: <FILE>

Available options:
    -h, --help  Prints help information
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
        let fetch_inner = fetch
            .to_options()
            .descr("fetches branches from remote repository");
        let fetch_cmd = command("fetch", fetch_inner);

        let interactive = short('i').switch();
        let all = long("all").switch();
        let files = positional("FILE").many();
        let add = construct!(Opt::Add {
            interactive,
            all,
            files
        });
        let add_inner = add.to_options().descr("add files to the staging area");
        let add_cmd = command("add", add_inner);

        construct!([fetch_cmd, add_cmd])
            .to_options()
            .descr("The stupid content tracker")
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
    -h, --help  Prints help information

Available commands:
    fetch  fetches branches from remote repository
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
    -h, --help     Prints help information
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
    -h, --help  Prints help information
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
        .help("Sets a number\nin two lines")
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
    })
    .to_options();

    assert_eq!(
        AppArgs {
            number: 42,
            opt_number: None,
            width: 10,
            input: vec![PathBuf::from("foo"), PathBuf::from("foo2")],
        },
        parser
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
        parser.run_inner(Args::from(&["--number", "42"])).unwrap()
    );

    drop(parser);
}

#[test]
fn simple_cargo_helper() {
    let a = short('a').long("AAAAA").help("two lines\nof help").switch();
    let b = short('b').switch();
    let parser = construct!(a, b);
    let decorated = cargo_helper("simple", parser)
        .to_options()
        .descr("this is a test");

    // cargo run variant
    let ok = decorated.run_inner(Args::from(&["-a"])).unwrap();
    assert_eq!((true, false), ok);

    // cargo simple variant
    let ok = decorated.run_inner(Args::from(&["simple", "-b"])).unwrap();
    assert_eq!((false, true), ok);

    let err = decorated
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
    -a, --AAAAA  two lines
                 of help
    -b
    -h, --help   Prints help information
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
    let parser = positional("FOO")
        .some("You need to specify at least one FOO")
        .to_options();

    let err = parser
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!("You need to specify at least one FOO", err);
}

#[test]
fn env_variable() {
    let name = "BPAF_SECRET_API_KEY";
    let parser = long("key")
        .env(name)
        .help("use this secret key\ntwo lines")
        .argument("KEY")
        .to_options();

    let help = parser
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: --key KEY

Available options:
        --key <KEY>  [env:BPAF_SECRET_API_KEY: N/A]
                     use this secret key
                     two lines
    -h, --help       Prints help information
";
    assert_eq!(expected_help, help);
    std::env::set_var(name, "top s3cr3t");

    let help = parser
        .run_inner(Args::from(&["-h"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: --key KEY

Available options:
        --key <KEY>  [env:BPAF_SECRET_API_KEY = \"top s3cr3t\"]
                     use this secret key
                     two lines
    -h, --help       Prints help information
";
    assert_eq!(expected_help, help);

    let res = parser.run_inner(Args::from(&["--key", "secret"])).unwrap();
    assert_eq!(res, "secret");

    let res = parser.run_inner(Args::from(&[])).unwrap();
    assert_eq!(res, "top s3cr3t");
}

#[test]
fn help_with_default_parse() {
    use bpaf::Parser;
    #[derive(Debug, Clone, Bpaf)]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(String),

        /// Does nothing
        #[bpaf(command)]
        NoAction,
    }

    let parser = action().or_else(bpaf::pure(Action::NoAction)).to_options();

    let help = parser
        .run_inner(bpaf::Args::from(&["add", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: <ARG>

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(bpaf::Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    add        Add a new TODO item
    no_action  Does nothing
";
    assert_eq!(expected_help, help);
}

#[test]
fn command_and_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(String),

        /// Does nothing
        /// in two lines
        #[bpaf(command)]
        NoAction,
    }

    use bpaf::Parser;
    let parser = action().fallback(Action::NoAction).to_options();

    let help = parser
        .run_inner(bpaf::Args::from(&["add", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: <ARG>

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(bpaf::Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [COMMAND ...]

Available options:
    -h, --help  Prints help information

Available commands:
    add        Add a new TODO item
    no_action  Does nothing
               in two lines
";
    assert_eq!(expected_help, help);
}

#[test]
fn optional_req_select() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let ab = a.or_else(b).optional();
    let parser = ab.to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: [-a | -b]

Available options:
    -a
    -b
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn dash_is_positional() {
    let a = positional("FILE");
    let parser = a.to_options();
    assert_eq!("-", parser.run_inner(Args::from(&["-"])).unwrap());
}

#[test]
fn default_plays_nicely_with_command() {
    #[derive(Debug, Clone)]
    enum Foo {
        Foo,
        Bar,
    }
    impl Default for Foo {
        fn default() -> Self {
            Foo::Bar
        }
    }

    let cmd = command("foo", pure(Foo::Foo).to_options().descr("inner"))
        .help("foo")
        .fallback(Default::default());

    let parser = cmd.to_options().descr("outer");

    let help = parser
        .run_inner(Args::from(&["foo", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
inner


Available options:
    -h, --help  Prints help information
";

    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
outer

Usage: [COMMAND ...]

Available options:
    -h, --help  Prints help information

Available commands:
    foo  foo
";

    assert_eq!(expected_help, help);
}

#[test]
fn command_with_aliases() {
    let inner = pure(()).to_options().descr("inner descr");
    let cmd = command("foo", inner).long("bar").short('f').short('b');
    let parser = cmd.to_options().descr("outer");

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
outer

Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    foo, f  inner descr
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(Args::from(&["f", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
inner descr


Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    // hidden and visible aliases are working
    parser.run_inner(Args::from(&["foo"])).unwrap();
    parser.run_inner(Args::from(&["f"])).unwrap();
    parser.run_inner(Args::from(&["bar"])).unwrap();
    parser.run_inner(Args::from(&["b"])).unwrap();

    // and "k" isn't a thing
    parser.run_inner(Args::from(&["k"])).unwrap_err();
}

#[test]
fn positional_with_help() {
    let user = positional("USER").help("github user\nin two lines");
    let api = positional("API_KEY").help("api key to use");
    let parser = construct!(user, api).to_options();

    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage: <USER> <API_KEY>

Available positional items:
    <USER>     github user
               in two lines
    <API_KEY>  api key to use

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn help_for_positional() {
    let c = positional("C").help("help for\nc");
    let d = positional("DDD").help("help for\nddd");
    let parser = construct!(c, d).to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: <C> <DDD>

Available positional items:
    <C>    help for
           c
    <DDD>  help for
           ddd

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn help_for_options() {
    let a = short('a').help("help for\na").switch();
    let b = short('c').env("BbBbB").help("help for\nb").argument("B");
    let c = long("bbbbb")
        .env("ccccCCccc")
        .help("help for\nccc")
        .argument("CCC");
    let parser = construct!(a, b, c).to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [-a] -c B --bbbbb CCC

Available options:
    -a                 help for
                       a
    -c <B>             [env:BbBbB: N/A]
                       help for
                       b
        --bbbbb <CCC>  [env:ccccCCccc: N/A]
                       help for
                       ccc
    -h, --help         Prints help information
";

    assert_eq!(expected_help, help);
}

#[test]
fn help_for_commands() {
    let d = command("thing_d", pure(()).to_options()).help("help for d\ntwo lines");
    let e = command("thing_e", pure(()).to_options())
        .short('e')
        .help("help for e\ntwo lines");
    let h = command("thing_h", pure(()).to_options());
    let parser = construct!(d, e, h).to_options();
    let help = parser
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    thing_d     help for d
                two lines
    thing_e, e  help for e
                two lines
    thing_h
";
    assert_eq!(expected_help, help);
}

#[test]
fn many_doesnt_panic() {
    let parser = short('a').switch().many().map(|m| m.len()).to_options();
    let r = parser.run_inner(Args::from(&["-aaa"])).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn some_doesnt_panic() {
    let parser = short('a').switch().some("").map(|m| m.len()).to_options();
    let r = parser.run_inner(Args::from(&["-aaa"])).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn command_resets_left_head_state() {
    #[derive(Debug, Eq, PartialEq)]
    enum Foo {
        Bar1 { a: u32 },
        Bar2 { b: () },
    }

    let a = short('a').argument("A").from_str::<u32>().fallback(0);
    let b = short('b').req_flag(());

    let p1 = construct!(Foo::Bar1 { a });
    let p2 = construct!(Foo::Bar2 { b });
    let cmd = construct!([p1, p2])
        .to_options()
        .command("cmd")
        .to_options();

    let xx = cmd.run_inner(Args::from(&["cmd", "-b"])).unwrap();
    assert_eq!(xx, Foo::Bar2 { b: () });
}

#[test]
fn command_preserves_custom_failure_message() {
    let msg = "need more cheese";
    let inner = fail::<()>(msg).to_options();

    let err = inner
        .run_inner(Args::from(&[]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err, msg);

    let outer = inner.command("feed").to_options();

    let err = outer
        .run_inner(Args::from(&["feed"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(err, msg);
}

#[test]
fn optional_error_handling() {
    let p = short('p')
        .argument("P")
        .from_str::<u32>()
        .optional()
        .to_options();

    let res = p.run_inner(Args::from(&[])).unwrap();
    assert_eq!(res, None);

    let res = p.run_inner(Args::from(&["-p", "3"])).unwrap();
    assert_eq!(res, Some(3));

    let res = p
        .run_inner(Args::from(&["-p", "pi"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "Couldn't parse \"pi\": invalid digit found in string");
}

#[test]
fn many_error_handling() {
    let p = short('p')
        .argument("P")
        .from_str::<u32>()
        .many()
        .to_options();

    let res = p.run_inner(Args::from(&[])).unwrap();
    assert_eq!(res, Vec::new());

    let res = p.run_inner(Args::from(&["-p", "3"])).unwrap();
    assert_eq!(res, vec![3]);

    let res = p
        .run_inner(Args::from(&["-p", "pi"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "Couldn't parse \"pi\": invalid digit found in string");
}

#[test]
fn failure_is_not_stupid_1() {
    let a = short('a').argument("A").from_str::<u32>();
    let b = pure(()).parse::<_, _, String>(|_| Err("nope".to_string()));
    let parser = construct!(a, b).to_options();

    let res = parser
        .run_inner(Args::from(&["-a", "42"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "Couldn't parse: nope");
}

#[test]
fn failure_is_not_stupid_2() {
    let a = short('a').argument("A").from_str::<u32>();
    let b = short('b').argument("B").from_str::<u32>();
    let parser = construct!(a, b)
        .parse::<_, (), String>(|_| Err("nope".to_string()))
        .to_options();

    let res = parser
        .run_inner(Args::from(&["-a", "42", "-b", "42"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "Couldn't parse: nope");
}

#[test]
fn no_fallback_out_of_command_parser() {
    let alt1 = positional("NAME").to_options().command("cmd");
    let alt2 = pure(String::new());
    let parser = construct!([alt1, alt2]).to_options();

    let res = parser
        .run_inner(Args::from(&["cmd"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "Expected <NAME>, pass --help for usage information");

    let res = parser.run_inner(Args::from(&["cmd", "a"])).unwrap();
    assert_eq!(res, "a");

    let res = parser.run_inner(Args::from(&[])).unwrap();
    assert_eq!(res, "");
}
