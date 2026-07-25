use crate::*;
#[test]
fn parse_failed_msg() {
    let parser = short('a').argument::<usize>("A").to_options();

    let r = parser.run_inner("-a 34x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '34x': invalid digit found in string\n");

    let r = parser.run_inner("-a=34x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '34x': invalid digit found in string\n");

    let r = parser.run_inner("-a34x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '34x': invalid digit found in string\n");
}

#[test]
fn parse_simple_flag_works_0() {
    let parser = short('a').switch().to_options();
    let r = parser.run_inner(["-a"]).unwrap();
    assert!(r);
}

#[test]
fn parse_simple_flag_works_1() {
    let a = short('a').switch();
    let b = short('b').flag(1, 2);
    let c = short('c').req_flag(());
    let parser = construct!(a, b, c).to_options();
    let r = parser.run_inner(["-a", "-b", "-c"]).unwrap();
    assert_eq!(r, (true, 1, ()));
}

#[test]
fn parse_simple_flag_works_2() {
    let a = short('a').switch();
    let b = short('a').switch();
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner(["-a", "-a"]).unwrap();
    assert_eq!(r, (true, true));
}

#[test]
fn optional_tuple_works() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!(a, b).optional().to_options();
    let r = parser.run_inner(["-a", "-b"]).unwrap();
    assert_eq!(r, Some(('a', 'b')));
}

#[test]
fn parse_simple_arg_works_1() {
    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a=10 -b 20").unwrap();
    assert_eq!(r, (10, 20));
}

#[test]
fn from_any_str_works() {
    let a = any_from_str::<i32>("I");
    let parser = a.to_options();

    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-42").unwrap();
    assert_eq!(r, -42);
}

#[test]
fn parse_optional_temp() {
    let a = short('a').argument::<usize>("ARG").optional();
    let b = short('b').argument::<usize>("ARG2");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-b10 -a 12").unwrap();
    assert_eq!(r, (Some(12), 10));

    let r = parser.run_inner("-b4").unwrap();
    assert_eq!(r, (None, 4));

    let r = parser.run_inner("-a pi").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'pi': invalid digit found in string\n");

    let r = parser.run_inner("-a=pi").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'pi': invalid digit found in string\n");

    let r = parser.run_inner("-api").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'pi': invalid digit found in string\n");
}

#[test]
fn many_works() {
    let parser = short('a').req_flag(()).many().to_options();
    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[(), (), ()]);
}

#[test]
fn many_error_handling() {
    let p = short('p').argument::<u32>("P").many().to_options();

    let res = p.run_inner("").unwrap();
    assert_eq!(res, Vec::new());

    let res = p.run_inner("-p 3").unwrap();
    assert_eq!(&res, &[3]);

    let res = p.run_inner("-p pi").unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse 'pi': invalid digit found in string\n");
}

#[test]
fn flag_group_works_reqflag() {
    let parser = short('a').req_flag(()).many().to_options();
    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[(), (), ()]);
}

#[test]
fn flag_group_works_switch() {
    let parser = short('a').switch().many().to_options();

    let r = parser.run_inner("-aaa").unwrap();
    assert_eq!(r, &[true, true, true]);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, &[false]);

    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[true, true, true]);
}

#[test]
fn combine_flags_by_order() {
    let a = short('a').req_flag(true);
    let b = short('A').req_flag(false);
    let parser = construct!([a, b]).many().to_options();

    let r = parser.run_inner("-a -A -A -A -a").unwrap();
    assert_eq!(&r, &[true, false, false, false, true]);

    let r = parser.run_inner("-aAAAa").unwrap();
    assert_eq!(&r, &[true, false, false, false, true]);
}

#[test]
fn many_with_optional() {
    let a = short('a').req_flag(()).optional();
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b).many().to_options();

    let r = parser.run_inner("-b30").unwrap();
    assert_eq!(r, &[(None, 30)]);

    let r = parser.run_inner("-a -b=10 -b20 -a -b 30").unwrap();
    assert_eq!(r, &[(Some(()), 10), (Some(()), 20), (None, 30)]);
}

#[test]
fn simple_alt_with_one_flag() {
    let a = short('a').req_flag('a');
    let a1 = short('A').switch();
    let a = construct!(a, a1);
    let b = short('b').req_flag('b');
    let b1 = short('B').switch();
    let b = construct!(b, b1);
    let parser = construct!([a, b]).to_options();
    let r = parser.run_inner("-a -A").unwrap();
    assert_eq!(r, ('a', true));
}

#[test]
fn simple_alt_with_flags() {
    let a = short('a').req_flag('a');
    let a1 = short('A').switch();
    let a2 = short('A').switch();
    let a3 = short('A').switch();
    let a4 = short('A').switch();
    let a = construct!(a, a1, a2, a3, a4);
    let b = short('b').req_flag('b');
    let b1 = short('B').switch();
    let b2 = short('B').switch();
    let b3 = short('B').switch();
    let b4 = short('B').switch();
    let b = construct!(b, b1, b2, b3, b4);
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-a -AAAA").unwrap();
    assert_eq!(r, ('a', true, true, true, true));
}

#[test]
fn simple_verbosity() {
    let a = short('v').req_flag(()).count();
    let parser = a.to_options();

    let r = parser.run_inner("-vvv").unwrap();
    assert_eq!(r, 3);
}

#[test]
fn nested_alt_works() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let d = short('d').req_flag('d');
    let ab = construct!([a, b]);
    let cd = construct!([c, d]);
    let parser = construct!([ab, cd]).to_options();

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn bare_parser() {
    let parser = short('b').req_flag('b').to_options();
    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn very_very_simple_alt() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).to_options();
    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');
}

#[test]
fn simple_alt_with_option() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).optional().to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, None);

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, Some('b'));
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Some('a'));
}

#[test]
fn simple_command() {
    let a = short('a').req_flag('a');
    let inner = a.to_options().command("hello");
    let b = short('b').req_flag('b');
    let parser = construct!([inner, b]).to_options();

    let r = parser.run_inner("hello -a").unwrap();
    assert_eq!(r, 'a');
    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn simple_literal() {
    let a = literal("hello")
        .help("This is sample command")
        .flag("lit", "no lit");
    let b = long("hello")
        .help("This is a switch")
        .flag("switch", "no switch");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("--hello").unwrap();
    assert_eq!(r, "switch");

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, "lit");

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, "no lit");
}

#[test]
fn simple_nested() {
    let inner = short('a').req_flag('a');
    let parser = short('b').nest(inner).to_options();
    let r = parser.run_inner("-b -a").unwrap();
    assert_eq!(r, 'a');
}

#[test]
fn flag_or_arg() {
    let a = short('a').req_flag(0);
    let b = short('a').argument::<usize>("A");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-a4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 0);

    let r = parser.run_inner("-a=4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("-a 4").unwrap();
    assert_eq!(r, 4);
}

#[test]
fn arg_or_flag() {
    // behavior should be identical to 'flag_or_arg'
    let a = short('a').req_flag(0);
    let b = short('a').argument::<usize>("A");
    let parser = construct!([b, a]).to_options();

    let r = parser.run_inner("-a4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 0);

    let r = parser.run_inner("-a=4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("-a 4").unwrap();
    assert_eq!(r, 4);
}

#[test]
fn multiple_any_parsers() {
    let a = any("A", |s: &str| s.parse::<i32>().ok().filter(|v| *v < -100));
    let b = any("B", |s: &str| {
        s.parse::<i32>().ok().filter(|v| (-100..100).contains(v))
    });
    let c = any("C", |s: &str| s.parse::<i32>().ok().filter(|v| *v > 100));
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner("-1000 1000 0").unwrap();
    assert_eq!(r, (-1000, 0, 1000));

    let r = parser.run_inner("1000 -1000 0").unwrap();
    assert_eq!(r, (-1000, 0, 1000));

    let r = parser.run_inner("0 -1000 1000").unwrap();
    assert_eq!(r, (-1000, 0, 1000));
}

#[test]
fn unexpected_in_pure_optional() {
    let parser = pure(12).optional().to_options();

    let r = parser.run_inner("asdf").unwrap_err().unwrap_stderr();

    let expected = "'asdf' is not expected in this context\n";
    assert_eq!(r, expected);
}

#[test]
fn pure_pair() {
    let a = pure(42);
    let b = pure(90);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, (42, 90));
}

#[test]
fn sneaky_command() {
    #[derive(Debug, Eq, PartialEq, Clone)]
    enum Cmd {
        A(bool),
        B(bool),
    }

    for sneaky in [false, true] {
        let cmd = short('f')
            .switch()
            .to_options()
            .command("hello")
            .map(Cmd::A);
        let b = short('f').switch().map(Cmd::B);

        // the point is to be able to dynamically select between different parsers
        let maybe_cmd = if sneaky {
            construct!(cmd).into_box()
        } else {
            let nope = fail("no sneaky");
            construct!(nope).into_box()
        };
        let parser = construct!([maybe_cmd, b]).to_options();

        let r = parser.run_inner("hello");
        if sneaky {
            assert_eq!(r.unwrap(), Cmd::A(false));
        } else {
            assert_eq!(r.unwrap_err().unwrap_stderr(), "no sneaky\n");
        }

        let r = parser.run_inner("-f").unwrap();
        assert_eq!(r, Cmd::B(true));
    }
}

#[test]
fn fail_vs_switch() {
    let a = short('f').flag(1, 2);
    let b = fail("oh noes");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-f").unwrap();
    assert_eq!(r, 1);
}

#[test]
fn command_inner_consumes_then_outer_continues() {
    let inner = positional::<String>("X").to_options().command("cmd").lazy();
    let outer = positional::<String>("Y");
    let parser = construct!(inner, outer).to_options();

    // cmd consumes "cmd" and "foo", outer consumes "bar"
    let r = parser.run_inner("cmd foo bar").unwrap();
    assert_eq!(r, ("foo".to_string(), "bar".to_string()));
}

#[test]
fn command_inner_consumes_nothing_then_outer_continues() {
    let inner = pure(42).to_options().command("cmd").lazy();
    let outer = positional::<String>("Y");
    let parser = construct!(inner, outer).to_options();

    // cmd consumes "cmd", outer consumes "foo"
    let r = parser.run_inner("cmd foo").unwrap();
    assert_eq!(r, (42, "foo".to_string()));
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

    let r = cmd.run_inner("cmd -b").unwrap();
    assert_eq!(r, Foo::Bar2 { b: () });
}

#[test]
fn command_inner_consumes_multiple_then_outer_continues() {
    let x = positional::<String>("X");
    let y = positional::<String>("Y");
    let inner = construct!(x, y).to_options().command("cmd").lazy();
    let z = positional::<String>("Z");
    let parser = construct!(inner, z).to_options();

    // cmd consumes "cmd" "a" "b", outer consumes "c"
    let r = parser.run_inner("cmd a b c").unwrap();
    assert_eq!(r, (("a".to_string(), "b".to_string()), "c".to_string()));
}

#[test]
fn anchor_start_with_keyword() {
    let anchor = literal("asm").req_flag(()).optional().anchor_start();
    let name = positional::<String>("NAME");
    let parser = construct!(anchor, name).to_options();

    let r = parser.run_inner("asm hello").unwrap();
    assert_eq!(r, (Some(()), "hello".to_string()));

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, (None, "hello".to_string()));
}

#[test]
fn anchor_start_with_optional_inner() {
    let anchor = literal("asm").switch().anchor_start();
    let name = positional::<String>("NAME");
    let parser = construct!(name, anchor).to_options();

    let r = parser.run_inner("asm hello").unwrap();
    assert_eq!(r, ("hello".to_string(), true));

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, ("hello".to_string(), false));
}

#[test]
fn anchor_start_with_short_literal() {
    // Test with a short literal
    let anchor = long("a").short('a').switch().anchor_start();
    let name = positional::<String>("NAME");
    let parser = construct!(anchor, name).to_options();

    let r = parser.run_inner("-a hello").unwrap();
    assert_eq!(r, (true, "hello".to_string()));

    let r = parser.run_inner("hello").unwrap();
    assert_eq!(r, (false, "hello".to_string()));

    let r = parser.run_inner("hello -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");
}

#[test]
fn many_doesnt_panic() {
    let parser = short('a').switch().many().count().to_options();

    let r = parser.run_inner("-aaa").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 1);
}

#[test]
fn some_doesnt_panic() {
    let parser = short('a').switch().some("want").count().to_options();

    let r = parser.run_inner("-aaa").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 1);
}

#[test]
fn many_env() {
    let parser = short('v')
        .env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .many()
        .to_options();
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, vec!["bpaf_core".to_owned()]);
}

#[test]
fn env_hidden_arg() {
    let parser = env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .to_options();
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, "bpaf_core");
}

#[test]
fn env_hidden_switch() {
    let parser = env("CARGO_PKG_NAME").switch().to_options();
    let r = parser.run_inner("").unwrap();
    assert!(r);
}

#[test]
fn env_hidden_flag() {
    let parser = env("CARGO_PKG_NAME").flag(true, false).to_options();
    let r = parser.run_inner("").unwrap();
    assert!(r);
}

#[test]
fn some_env() {
    let parser = short('v')
        .env("CARGO_PKG_NAME")
        .argument::<String>("USER")
        .some("a")
        .to_options();
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, vec!["bpaf_core".to_owned()]);
}

#[test]
fn id_gap_from_immediate_parsers() {
    // Regression: a parser that completes immediately (e.g. fail) inside a
    // subcommand consumed a next_free ID but was never stored in the tasks
    // vector. The sub-executor then tried to assert_no_tasks_past_end at a
    // scope_start beyond the vector's actual length, causing a panic.
    let inner = fail::<()>("bad").to_options();
    let parser = inner.command("cmd").to_options();
    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "bad\n");
}

#[test]
fn exit_from_inner_parser_help() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').switch().then_exit(Exit::current_parser);
    let nest = short('i').nest((ia, ib, e)).optional();
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (nest, a, b).to_options();
    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [-i {[-a] [-b] [-e]}] [-a] [-b]

Available options:
    -i [-a] [-b] [-e]
    -a
    -b
    -e
    -a
    -b
    -h, --help  Prints help information
";
    assert_eq!(help, expected);
}

#[test]
fn exit_from_inner_parser() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').switch().then_exit(Exit::current_parser);
    let nest = short('i').nest((ia, ib, e)).optional();
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (nest, a, b).to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, (None, true, false));

    let r = parser.run_inner("-i -a -e -b").unwrap();
    assert_eq!(r, (Some((true, false, true)), false, true));
}

#[test]
fn exit_from_inner_parser_also_help() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').flag((), ()).then_exit(Exit::current_parser);
    let nest = short('i').nest((ia, ib).and_also(e)).optional();
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (nest, a, b).to_options();
    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [-i {[-a] [-b] [-e]}] [-a] [-b]

Available options:
    -i [-a] [-b] [-e]
    -a
    -b
    -e
    -a
    -b
    -h, --help  Prints help information
";
    assert_eq!(help, expected);
}

#[test]
fn exit_from_inner_parser_also() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').flag((), ()).then_exit(Exit::current_parser);
    let nest = short('i').nest((ia, ib).and_also(e)).optional();
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (nest, a, b).to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, (None, true, false));

    let r = parser.run_inner("-i -a -e -b").unwrap();
    assert_eq!(r, (Some((true, false)), false, true));
}

#[test]
fn exit_from_command_subparser_help() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').switch().then_exit(Exit::current_parser);
    let cmd = (ia, ib, e).to_options().command("cmd");
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (cmd, a, b).to_options();
    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app COMMAND ... [-a] [-b]

Available options:
    -a
    -b
    -h, --help  Prints help information

Available commands:
    cmd
";
    assert_eq!(help, expected);

    let help = parser.run_inner("cmd --help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app cmd [-a] [-b] [-e]

Available options:
    -a
    -b
    -e
    -h, --help  Prints help information
";
    assert_eq!(help, expected);
}

#[test]
fn exit_from_command_subparser() {
    let ia = short('a').switch();
    let ib = short('b').switch();
    let e = short('e').switch().then_exit(Exit::current_parser);
    let cmd = (ia, ib, e).to_options().command("cmd");
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = (cmd, a, b).to_options();

    let r = parser.run_inner("cmd -a -b").unwrap();
    assert_eq!(r, ((true, true, false), false, false));

    let r = parser.run_inner("cmd -a -e -b -a").unwrap();
    assert_eq!(r, ((true, false, true), true, true));
}
