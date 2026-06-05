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
}

#[test]
fn many_works() {
    let parser = short('a').req_flag(()).many().to_options();
    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[(), (), ()]);
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
