use std::convert::Infallible;

use bpaf::*;

#[test]
fn simple_cargo_helper() {
    let a = short('a').long("AAAAA").help("two lines\nof help").switch();
    let b = short('b').switch();
    let parser = construct!(a, b);
    let decorated = cargo_helper("simple", parser)
        .to_options()
        .descr("this is a test");

    // cargo run variant
    let ok = decorated.run_inner(&["-a"]).unwrap();
    assert_eq!((true, false), ok);

    // cargo simple variant
    let ok = decorated.run_inner(&["simple", "-b"]).unwrap();
    assert_eq!((false, true), ok);

    let err = decorated
        .run_inner(&["-a", "-a"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        "argument `-a` cannot be used multiple times in this context",
        err
    );

    let help = decorated.run_inner(&["-h"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
this is a test

Usage: [-a] [-b]

Available options:
    -a, --AAAAA  two lines of help
    -b
    -h, --help   Prints help information
";
    assert_eq!(expected_help, help);
}

#[test]
fn hidden_env() {
    let name = "BPAF_SECRET_API_KEY2";
    let visible = long("key")
        .help("use this secret key\n two lines")
        .argument::<String>("KEY");
    let hidden = env(name).argument("KEY");
    let parser = construct!([visible, hidden]).to_options();

    let help = parser.run_inner(&["-h"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: --key=KEY

Available options:
        --key=KEY  use this secret key
                   two lines
    -h, --help     Prints help information
";

    assert_eq!(help, expected);

    let r = parser.run_inner(&[]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "environment variable `BPAF_SECRET_API_KEY2` is not set");
}

#[test]
fn env_variable() {
    let name = "BPAF_SECRET_API_KEY";
    let parser = long("key")
        .env(name)
        .help("use this secret key\ntwo lines")
        .argument::<String>("KEY")
        .to_options();

    let help = parser.run_inner(&["-h"]).unwrap_err().unwrap_stdout();
    let expected_help = "\
Usage: --key=KEY

Available options:
        --key=KEY  use this secret key two lines
                   [env:BPAF_SECRET_API_KEY: N/A]
    -h, --help     Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser.run_inner(&["-h"]).unwrap_err().unwrap_stdout();
    let expected_help = "\
Usage: --key=KEY

Available options:
        --key=KEY  use this secret key two lines
                   [env:BPAF_SECRET_API_KEY = \"top s3cr3t\"]
    -h, --help     Prints help information
";
    assert_eq!(expected_help, help);

    let res = parser.run_inner(&["--key", "secret"]).unwrap();
    assert_eq!(res, "secret");

    let res = parser.run_inner(&[]).unwrap();
    assert_eq!(res, "top s3cr3t");
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

    let cmd = pure(Foo::Foo)
        .to_options()
        .descr("inner")
        .command("foo")
        .help("foo")
        .fallback(Default::default());

    let parser = cmd.to_options().descr("outer");

    let help = parser
        .run_inner(&["foo", "--help"])
        .unwrap_err()
        .unwrap_stdout();

    let expected_help =
        "inner\n\nUsage: foo \n\nAvailable options:\n    -h, --help  Prints help information\n";

    assert_eq!(expected_help, help);

    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
outer

Usage: [COMMAND ...]

Available options:
    -h, --help  Prints help information

Available commands:
    foo         foo
";

    assert_eq!(expected_help, help);
}

#[test]
fn command_with_aliases() {
    let inner = pure(()).to_options().descr("inner descr");
    let cmd = inner.command("foo").long("bar").short('f').short('b');
    let parser = cmd.to_options().descr("outer");

    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
outer

Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    foo, f      inner descr
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(&["f", "--help"])
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "inner descr\n\nUsage: foo \n\nAvailable options:\n    -h, --help  Prints help information\n";
    assert_eq!(expected_help, help);

    // hidden and visible aliases are working
    parser.run_inner(&["foo"]).unwrap();
    parser.run_inner(&["f"]).unwrap();
    parser.run_inner(&["bar"]).unwrap();
    parser.run_inner(&["b"]).unwrap();

    // and "k" isn't a thing
    parser.run_inner(&["k"]).unwrap_err();
}

#[test]
fn help_for_options() {
    let a = short('a').help("help for\na").switch();
    let b = short('c')
        .env("BbBbB")
        .help("help for\nb")
        .argument::<String>("B");
    let c = long("bbbbb")
        .env("ccccCCccc")
        .help("help for\nccc")
        .argument::<String>("CCC");
    let parser = construct!(a, b, c).to_options();
    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
Usage: [-a] -c=B --bbbbb=CCC

Available options:
    -a               help for a
    -c=B             help for b
                     [env:BbBbB: N/A]
        --bbbbb=CCC  help for ccc
                     [env:ccccCCccc: N/A]
    -h, --help       Prints help information
";

    assert_eq!(expected_help, help);
}

#[test]
fn help_for_commands() {
    let d = pure(())
        .to_options()
        .command("thing_d")
        .help("help for d\ntwo lines");
    let e = pure(())
        .to_options()
        .command("thing_e")
        .short('e')
        .help("help for e\ntwo lines");
    let h = pure(()).to_options().command("thing_h");
    let parser = construct!([d, e, h]).to_options();
    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    thing_d     help for d two lines
    thing_e, e  help for e two lines
    thing_h
";
    assert_eq!(expected_help, help);
}

#[test]
fn many_doesnt_panic() {
    let parser = short('a').switch().many().map(|m| m.len()).to_options();
    let r = parser.run_inner(&["-aaa"]).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn some_doesnt_panic() {
    let parser = short('a').switch().some("").map(|m| m.len()).to_options();
    let r = parser.run_inner(&["-aaa"]).unwrap();
    assert_eq!(r, 3);
}

#[test]
fn command_resets_left_head_state() {
    #[derive(Debug, Eq, PartialEq)]
    enum Foo {
        Bar1 { a: u32 },
        Bar2 { b: () },
    }

    let a = short('a').argument::<u32>("A").fallback(0);
    let b = short('b').req_flag(());

    let p1 = construct!(Foo::Bar1 { a });
    let p2 = construct!(Foo::Bar2 { b });
    let cmd = construct!([p1, p2])
        .to_options()
        .command("cmd")
        .to_options();

    let xx = cmd.run_inner(&["cmd", "-b"]).unwrap();
    assert_eq!(xx, Foo::Bar2 { b: () });
}

#[test]
fn command_preserves_custom_failure_message() {
    let msg = "need more cheese";
    let inner = fail::<()>(msg).to_options();

    let err = inner.run_inner(&[]).unwrap_err().unwrap_stderr();
    assert_eq!(err, "need more cheese");

    let outer = inner.command("feed").to_options();

    let err = outer.run_inner(&["feed"]).unwrap_err().unwrap_stderr();
    assert_eq!(err, "need more cheese");
}

#[test]
fn optional_error_handling() {
    let p = short('p').argument::<u32>("P").optional().to_options();

    let res = p.run_inner(&[]).unwrap();
    assert_eq!(res, None);

    let res = p.run_inner(&["-p", "3"]).unwrap();
    assert_eq!(res, Some(3));

    let res = p.run_inner(&["-p", "pi"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse `pi`: invalid digit found in string");
}

#[test]
fn many_error_handling() {
    let p = short('p').argument::<u32>("P").many().to_options();

    let res = p.run_inner(&[]).unwrap();
    assert_eq!(res, Vec::new());

    let res = p.run_inner(&["-p", "3"]).unwrap();
    assert_eq!(res, vec![3]);

    let res = p.run_inner(&["-p", "pi"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse `pi`: invalid digit found in string");
}

#[test]
fn failure_is_not_stupid_1() {
    let a = short('a').argument::<u32>("A");
    let b = pure(()).parse::<_, _, String>(|_| Err("nope".to_string()));
    let parser = construct!(a, b).to_options();

    let res = parser.run_inner(&["-a", "42"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse: nope");
}

#[test]
fn failure_is_not_stupid_2() {
    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b)
        .parse::<_, (), String>(|_| Err("nope".to_string()))
        .to_options();

    let res = parser
        .run_inner(&["-a", "42", "-b", "42"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "couldn't parse: nope");
}

#[test]
fn no_fallback_out_of_command_parser() {
    let alt1 = positional::<String>("NAME").to_options().command("cmd");
    let alt2 = pure(String::new());
    let parser = construct!([alt1, alt2]).to_options();

    let res = parser.run_inner(&["cmd"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "expected `NAME`, pass `--help` for usage information");

    let res = parser.run_inner(&["cmd", "a"]).unwrap();
    assert_eq!(res, "a");

    let res = parser.run_inner(&[]).unwrap();
    assert_eq!(res, "");
}

#[test]
fn did_you_mean_switch() {
    let a = short('f').long("flag").switch();
    let b = short('p').long("plag").switch();
    let parser = construct!([a, b]).to_options();

    let res = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--fla`, did you mean `--flag`?");

    let res = parser.run_inner(&["flag"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        res,
        "no such command or positional: `flag`, did you mean `--flag`?"
    );

    let res = parser.run_inner(&["--pla"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--pla`, did you mean `--plag`?");

    let res = parser.run_inner(&["--p"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        res,
        "no such flag: `--p` (with two dashes), did you mean `-p`?"
    );
}

#[test]
fn did_you_mean_req_flag() {
    let parser = long("flag").req_flag(()).to_options();
    let res = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--fla`, did you mean `--flag`?");
}

#[test]
fn did_you_mean_argument() {
    let parser = long("flag").argument::<String>("VAL").to_options();

    let res = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--fla`, did you mean `--flag`?");

    let res = parser
        .run_inner(&["--flg=hellop"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "no such flag: `--flg`, did you mean `--flag`?");
}

#[test]
fn did_you_mean_command() {
    let parser = pure(())
        .to_options()
        .command("command")
        .short('c')
        .to_options();

    let res = parser.run_inner(&["comman"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        res,
        "no such command or positional: `comman`, did you mean `command`?"
    );

    let res = parser.run_inner(&["--comman"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--comman`, did you mean `command`?");
}

#[test]
fn did_you_mean_two_and_arguments() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = cargo_helper("cmd", construct!(a, b)).to_options();

    let r = parser
        .run_inner(&["--flag", "--parametr"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--parametr`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["--flag", "--paramet=value"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--paramet`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["--parameter", "--flg"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--flg`, did you mean `--flag`?");

    let r = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: `--fla`, did you mean `--flag`?");
}

#[test]
fn did_you_mean_two_or_arguments() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = cargo_helper("cmd", construct!([a, b])).to_options();

    let r = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: `--fla`, did you mean `--flag`?");

    let r = parser
        .run_inner(&["--flag", "--parametr"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--parametr`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["--parametr", "--flag"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--parametr`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["--parameter", "--flag"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--flag` cannot be used at the same time as `--parameter`"
    );

    let r = parser
        .run_inner(&["--flag", "--parameter"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--parameter` cannot be used at the same time as `--flag`"
    );
}

// problematic steps look something like this:
// - "asm" is parsed as expected
// - "-t x" is consumed as expected
// - parsing of "x" fails
// - ParseWith rollbacks the arguments state - "asm" is back
// - suggestion looks for something it can complain at and finds "asm"
//
// parse/guard failures should "taint" the arguments and disable the suggestion logic

#[test]
fn cargo_show_asm_issue_guard() {
    let target_dir = short('t').argument::<String>("T").guard(|_| false, "nope");
    let verbosity = short('v').switch();
    let inner = construct!(target_dir, verbosity);
    let parser = cargo_helper("asm", inner).to_options();

    let res = parser
        .run_inner(&["asm", "-t", "x"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`x`: nope");

    let res = parser.run_inner(&["-t", "x"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "`x`: nope");
}

#[test]
fn cargo_show_asm_issue_from_str() {
    let target_dir = short('t').argument::<usize>("T");
    let verbosity = short('v').switch();
    let inner = construct!(target_dir, verbosity);
    let parser = cargo_helper("asm", inner).to_options();

    let res = parser
        .run_inner(&["asm", "-t", "x"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "couldn't parse `x`: invalid digit found in string");

    let res = parser.run_inner(&["-t", "x"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse `x`: invalid digit found in string");
}

#[test]
fn cargo_show_asm_issue_parse() {
    let target_dir = short('t')
        .argument::<String>("T")
        .parse::<_, (), String>(|_| Err("nope".to_string()));

    let verbosity = short('v').switch();
    let inner = construct!(target_dir, verbosity);
    let parser = cargo_helper("asm", inner).to_options();

    let res = parser
        .run_inner(&["asm", "-t", "x"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "couldn't parse `x`: nope");

    let res = parser.run_inner(&["-t", "x"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse `x`: nope");
}

// problematic case looks something like this:
// "asm" is consumed
// "--fla" is not consumed, but the output is not tainted
// parser fails and arguments are restored.

#[test]
fn cargo_show_asm_issue_unknown_switch() {
    let target = long("flag").switch();

    let verbosity = short('v').switch();
    let parser = cargo_helper("asm", construct!(target, verbosity)).to_options();

    let res = parser
        .run_inner(&["asm", "--fla"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "no such flag: `--fla`, did you mean `--flag`?");

    let res = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: `--fla`, did you mean `--flag`?");
}

#[test]
fn did_you_mean_inside_command() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = construct!([a, b]).to_options().command("cmd").to_options();

    let r = parser.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "expected `COMMAND ...`, got `--fla`. Pass `--help` for usage information"
    );

    let r = parser
        .run_inner(&["cmd", "--fla"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--fla`, did you mean `--flag`?");

    let r = parser
        .run_inner(&["cmd", "--flag", "--parametr"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--parametr`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["cmd", "--parametr", "--flag"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--parametr`, did you mean `--parameter`?");

    let r = parser
        .run_inner(&["cmd", "--parameter", "--flag"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--flag` cannot be used at the same time as `--parameter`"
    );

    let r = parser
        .run_inner(&["cmd", "--flag", "--parameter"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--parameter` cannot be used at the same time as `--flag`"
    );
}

#[test]
fn combine_flags_by_order() {
    let a = short('a').req_flag(true);
    let b = short('A').req_flag(false);
    let parser = construct!([a, b]).many().to_options();

    let r = parser.run_inner(&["-a", "-A", "-A", "-A", "-a"]).unwrap();
    assert_eq!(r, vec![true, false, false, false, true]);
}

#[test]
fn parse_many_errors_positional() {
    let p = positional::<u32>("N").many().to_options();

    let r = p.run_inner(&["1", "2", "3"]).unwrap();
    assert_eq!(r, vec![1, 2, 3]);

    let r = p.run_inner(&["1", "2", "x"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `x`: invalid digit found in string");
}

#[test]
fn parse_collect_flag() {
    let p = short('p')
        .argument::<u32>("N")
        .collect::<Vec<_>>()
        .to_options();

    let r = p.run_inner(&["-p", "1", "-p", "2"]).unwrap();
    assert_eq!(r, vec![1, 2]);

    let r = p
        .run_inner(&["-p", "1", "-p", "x"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "couldn't parse `x`: invalid digit found in string");
}

#[test]
fn parse_many_errors_flag() {
    let p = short('p').argument::<u32>("N").many().to_options();

    let r = p.run_inner(&["-p", "1", "-p", "2"]).unwrap();
    assert_eq!(r, vec![1, 2]);

    let r = p
        .run_inner(&["-p", "1", "-p", "x"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "couldn't parse `x`: invalid digit found in string");
}

#[test]
fn command_with_req_parameters() {
    let p = positional::<String>("X")
        .to_options()
        .command("cmd")
        .fallback(String::new())
        .to_options();

    let r = p.run_inner(&["cmd"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `X`, pass `--help` for usage information");
}

#[test]
fn suggestion_for_equals_1() {
    let parser = short('p').long("par").argument::<String>("P").to_options();

    let r = parser
        .run_inner(&["-p", "--bar"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`-p` requires an argument `P`, got a flag `--bar`, try `-p=--bar` to use it as an argument"
    );

    let r = parser
        .run_inner(&["--par", "--bar"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--par` requires an argument `P`, got a flag `--bar`, try `--par=--bar` to use it as an argument"
    );

    let r = parser
        .run_inner(&["--par", "--bar=baz"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "`--par` requires an argument `P`, got a flag `--bar=baz`, try `--par=--bar=baz` to use it as an\nargument"
    );
}

#[test]
fn double_dash_is_pos_only_just_once() {
    let parser = positional::<String>("POS").many().to_options();

    let r = parser.run_inner(&["--"]).unwrap();
    assert_eq!(r, Vec::<String>::new());

    let r = parser.run_inner(&["--", "--"]).unwrap();
    assert_eq!(r, vec!["--".to_string()]);
}

#[test]
fn reject_fbar() {
    let parser = short('f').argument::<String>("F").to_options();

    let r = parser
        .run_inner(&["-fbar", "baz"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`baz` is not expected in this context");

    let r = parser.run_inner(&["-fbar"]).unwrap();
    assert_eq!(r, "bar");
}

#[test]
fn custom_usage_override_fixed() {
    let parser = short('p').switch().to_options().usage("Usage: hey [-p]");
    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: hey [-p]\n\nAvailable options:\n    -p\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn custom_usage_override_with_fn() {
    let parser = short('p').switch().to_options().with_usage(|b| {
        let mut buf = Doc::default();
        buf.text("Usage: hey ");
        buf.doc(&b);
        buf
    });
    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: hey [-p]\n\nAvailable options:\n    -p\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn catch_works() {
    #[derive(Debug, Eq, PartialEq)]
    enum A {
        Num(usize),
        Str(String),
    }
    let a_n = short('a')
        .argument::<usize>("A")
        .map(A::Num)
        .optional()
        .catch();
    let a_s = short('a')
        .argument::<String>("A")
        .map(A::Str)
        .hide()
        .optional();
    let parser = construct!([a_n, a_s]).to_options();

    let r = parser.run_inner(&["-a", "1"]).unwrap();
    assert_eq!(r, Some(A::Num(1)));

    let r = parser.run_inner(&["-a", "x1"]).unwrap();
    assert_eq!(r, Some(A::Str("x1".to_owned())));
}

#[test]
fn sneaky_command() {
    #[derive(Debug, Eq, PartialEq)]
    enum Cmd {
        A(bool),
        B(bool),
    }
    let cmd = short('f')
        .switch()
        .to_options()
        .command("hello")
        .map(Cmd::A);
    let b = short('f').switch().map(Cmd::B);

    let sneaky = true;

    let maybe_cmd = if sneaky {
        construct!(cmd).into_rc()
    } else {
        let nope = fail("no sneaky");
        construct!(nope).into_rc()
    };
    let parser = construct!([maybe_cmd, b]).to_options();

    let r = parser.run_inner(&["hello"]).unwrap();
    assert_eq!(r, Cmd::A(false));
}

#[test]
fn default_for_many() {
    let parser = positional::<String>("ROOTS")
        .many()
        .map(|items| {
            if items.is_empty() {
                vec![String::from(".")]
            } else {
                items
            }
        })
        .to_options();

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, vec![String::from(".")]);
}

#[test]
fn parse_option_catch() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum A {
        U32(u32),
        S(String),
    }
    let a1 = short('a').argument("N").map(A::U32).optional().catch();
    let a2 = short('a').argument("S").map(A::S).optional().catch();
    let parser = construct!([a1, a2]).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap();
    assert_eq!(r, Some(A::U32(10)));

    let r = parser.run_inner(&["-a", "x"]).unwrap();
    assert_eq!(r, Some(A::S("x".to_string())));

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, None);
}

#[test]
fn parse_some_catch() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum A {
        U32(u32),
        S(String),
    }
    let a1 = short('a')
        .argument("N")
        .map(A::U32)
        .some("A")
        .catch()
        .hide();
    let a2 = short('a').argument("S").map(A::S).some("A").catch().hide();
    let parser = construct!([a1, a2]).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap();
    assert_eq!(r, vec![A::U32(10)]);

    let r = parser.run_inner(&["-a", "x"]).unwrap();
    assert_eq!(r, vec![A::S("x".to_string())]);

    let r = parser.run_inner(&[]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "A");
}

#[test]
fn empty_struct() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Foo {}
    let parser = construct!(Foo {}).to_options();

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, Foo {});

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: \n\nAvailable options:\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn empty_tuple() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Foo();
    let parser = construct!(Foo {}).to_options();

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, Foo());

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: \n\nAvailable options:\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn strange_short_option() {
    let parser = short('O').argument::<String>("ARG").to_options();
    let r = parser.run_inner(&["-Obits=2048"]).unwrap();
    assert_eq!(r, "bits=2048");
}

#[test]
fn optional_bool_states() {
    let parser = short('a').switch().optional().to_options();

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, Some(true));

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, Some(false));
}

#[test]
fn fancy_negative() {
    let a = short('a').req_flag(());
    #[allow(clippy::redundant_closure)]
    let b = any("A", |i: i32| Some(i));
    let ab = construct!(a, b).adjacent().map(|x| x.1);

    let c = short('c').argument::<usize>("C").fallback(42);

    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(&["-a", "-10"]).unwrap();
    assert_eq!(r, (-10, 42));

    let r = parser.run_inner(&["-a=-20", "-c", "110"]).unwrap();
    assert_eq!(r, (-20, 110));

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    // TODO - rendering sucks once you start inventing fancy combinations and don't provide help...
    let expected = "\
Usage: -a A [-c=C]

Available options:
  -a A

    -c=C
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn many_env() {
    let parser = short('v')
        .env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .many()
        .to_options();
    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, vec!["top s3cr3t".to_owned()]);
}

#[test]
fn env_hidden_arg() {
    let parser = env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .to_options();
    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, "top s3cr3t");
}

#[test]
fn env_hidden_switch() {
    let parser = env("CARGO_PKG_NAME").switch().to_options();
    let r = parser.run_inner(&[]).unwrap();
    assert!(r);
}

#[test]
fn env_hidden_flag() {
    let parser = env("CARGO_PKG_NAME").flag(true, false).to_options();
    let r = parser.run_inner(&[]).unwrap();
    assert!(r);
}

#[test]
fn some_env() {
    let parser = short('v')
        .env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .some("a")
        .to_options();
    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, vec!["top s3cr3t".to_owned()]);
}

#[test]
fn option_requires_other_option1() {
    let a = short('a').switch();
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).optional().to_options();

    let r = parser.run_inner(&["-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `-b=B`, pass `--help` for usage information");
}

#[test]
fn option_requires_other_option2() {
    let a = short('a').switch();
    let b = short('b').argument::<String>("B");
    let parser = construct!(b, a).optional().to_options();

    let r = parser.run_inner(&["-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `-b=B`, pass `--help` for usage information");
}

#[test]
fn default_for_some() {
    let parser = bpaf::positional::<u32>("ROOTS")
        .some("msg")
        .fallback_with(|| Ok::<_, Infallible>(vec![1, 2, 3]))
        .to_options();

    let r = parser.run_inner(&[]).unwrap();

    assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn adjacent_anywhere_needs_to_consume_something() {
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = construct!(a, b).adjacent().to_options();

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner(&["-b"]).unwrap();
    assert_eq!(r, (false, true));
}

#[test]
fn fallback_for_some() {
    let a = short('a')
        .argument::<u32>("ARG")
        .some("potatoes")
        .fallback(vec![1, 2, 3]);
    let parser = a.to_options();

    let r = parser.run_inner(&["-a", "1"]).unwrap();
    assert_eq!(r, vec![1]);

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn flag_like_commands() {
    let a = short('a').req_flag(1).to_options().command("--add");
    let b = short('b').req_flag(2).to_options().command("remove");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner(&["--add", "-a"]).unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner(&["remove", "-b"]).unwrap();
    assert_eq!(r, 2);

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "Usage: COMMAND ...\n\nAvailable options:\n    -h, --help  Prints help information\n\nAvailable commands:\n    --add\n    remove\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(&["--add", "--help"])
        .unwrap_err()
        .unwrap_stdout();
    let expected =
        "Usage: --add -a\n\nAvailable options:\n    -a\n    -h, --help  Prints help information\n";
    assert_eq!(r, expected);
}
