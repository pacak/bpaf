use crate::*;

#[test]
fn simple_global_flag_no_command() {
    let g = short('g').switch().global();
    let parser = g.to_options();
    assert!(parser.run_inner("-g").is_ok());
}

#[test]
fn simple_global_parser() {
    let g = short('g').switch().global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("cmd -g").unwrap();
    assert_eq!(r, (true, 42));

    let r = parser.run_inner("-g cmd").unwrap();
    assert_eq!(r, (true, 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, (false, 42));
}

#[test]
fn stacked_global_local() {
    let g = short('g').switch().global();
    let p = short('a').switch().to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("cmd -g").unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner("cmd -ag").unwrap();
    assert_eq!(r, (true, true));

    let r = parser.run_inner("-g cmd").unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, (false, false));
}

#[test]
fn double_simple_global_parser() {
    let g = short('g').switch().global().global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("cmd -g").unwrap();
    assert_eq!(r, (true, 42));

    let r = parser.run_inner("-g cmd").unwrap();
    assert_eq!(r, (true, 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, (false, 42));
}

#[test]
fn global_with_conflict() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b').default();
    let g = construct!([a, b]).global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("cmd -a").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ('b', 42));
}

#[test]
fn product_of_two_global_parsers() {
    let a = short('a').switch().global();
    let b = short('b').switch().global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(a, b, p).to_options();

    let r = parser.run_inner("-a -b cmd").unwrap();
    assert_eq!(r, (true, true, 42));

    let r = parser.run_inner("cmd -a -b").unwrap();
    assert_eq!(r, (true, true, 42));

    let r = parser.run_inner("-a cmd -b").unwrap();
    assert_eq!(r, (true, true, 42));

    let r = parser.run_inner("-b cmd -a").unwrap();
    assert_eq!(r, (true, true, 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, (false, false, 42));

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, (true, false, 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, (false, true, 42));

    let r = parser.run_inner("-a -a cmd").unwrap_err().unwrap_stderr();
    let expected = "argument '-a' cannot be used multiple times in this context\n";
    assert_eq!(r, expected);

    // TODO - error conflicts are recorded locally. global conflicts should be recorded in two
    // places probably. Or just accept a slightly worse error message
    let r = parser.run_inner("-a cmd -a").unwrap_err().unwrap_stderr();
    let expected = "'-a' is not expected in this context\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("cmd -b -b").unwrap_err().unwrap_stderr();
    let expected = "'-b' is not expected in this context\n";
    assert_eq!(r, expected);
}

#[test]
fn conflict_of_sum() {
    let a = short('a').req_flag('a').global();
    let b = short('b').req_flag('b').global();
    let g = construct!([a, b]);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a -b cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("cmd -a -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");
}

#[test]
fn sum_of_two_global_parsers() {
    let a = short('a').req_flag('a').global();
    let b = short('b').req_flag('b').global();
    let g = construct!([a, b]);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("-b cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd -a").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("-a -b cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b -a cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("cmd -a -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("cmd -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a', or more\n");
}

#[test]
fn product_with_first_global() {
    let a = short('a').switch().global();
    let b = short('b').switch();
    let ab = (a, b);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(ab, p).to_options();

    let r = parser.run_inner("-a -b cmd").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("-b cmd -a").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");

    let r = parser.run_inner("cmd -a -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ((false, false), 42));

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ((true, false), 42));

    let r = parser.run_inner("-b cmd").unwrap();
    assert_eq!(r, ((false, true), 42));

    let r = parser.run_inner("-a -a cmd").unwrap_err().unwrap_stderr();
    let expected = "argument '-a' cannot be used multiple times in this context\n";
    assert_eq!(r, expected);
}

#[test]
fn product_with_second_global() {
    let a = short('a').switch();
    let b = short('b').switch().global();
    let ab = construct!(a, b);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(ab, p).to_options();

    let r = parser.run_inner("-a -b cmd").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("-a cmd -b").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("cmd -a -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");

    let r = parser.run_inner("-b cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ((false, false), 42));

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ((true, false), 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ((false, true), 42));

    let r = parser.run_inner("cmd -b -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");
}

#[test]
fn sum_with_first_global() {
    let a = short('a').req_flag('a').global();
    let b = short('b').req_flag('b').default();
    let g = construct!([a, b]);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -a").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("-b cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("-a -b cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b -a cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");

    let r = parser.run_inner("-b cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("cmd -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");
}

#[test]
fn sum_with_second_global() {
    let a = short('a').req_flag('a').default();
    let b = short('b').req_flag('b').global();
    let g = construct!([a, b]);
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-b cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");
}

#[test]
fn product_made_global() {
    let a = short('a').switch();
    let b = short('b').switch();
    let g = construct!(a, b).global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a -b cmd").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("cmd -a -b").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("-a cmd -b").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("-b cmd -a").unwrap();
    assert_eq!(r, ((true, true), 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ((false, false), 42));

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ((true, false), 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ((false, true), 42));

    let r = parser.run_inner("-a -a cmd").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument '-a' cannot be used multiple times in this context\n"
    );
}

#[test]
fn sum_made_global() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b').default();
    let g = construct!([a, b]).global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("cmd -a").unwrap();
    assert_eq!(r, ('a', 42));

    let r = parser.run_inner("-b cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd -b").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, ('b', 42));

    let r = parser.run_inner("-a -b cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b -a cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("-a cmd -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("-b cmd -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");

    let r = parser.run_inner("cmd -a -b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' cannot be used at the same time as '-a'\n");

    let r = parser.run_inner("cmd -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '-b'\n");
}

#[test]
fn global_takes_priority() {
    let ga = short('a').switch().global();
    let a = short('a').switch().to_options().command("cmd");
    let parser = (ga, a).to_options();

    let r = parser.run_inner("-a cmd").unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner("-a cmd -a").unwrap();
    assert_eq!(r, (true, true));

    let r = parser.run_inner("cmd -a").unwrap();
    assert_eq!(r, (true, false));
}

#[test]
fn global_argument() {
    let g = short('a').argument::<u32>("ARG").global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-a 10 cmd").unwrap();
    assert_eq!(r, (10, 42));

    let r = parser.run_inner("cmd -a 10").unwrap();
    assert_eq!(r, (10, 42));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a=ARG'\n");
}

#[test]
fn two_global_arguments() {
    let a = short('a').argument::<u32>("A").global();
    let b = short('b').argument::<u32>("B").global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(a, b, p).to_options();

    let r = parser.run_inner("-a 10 -b 20 cmd").unwrap();
    assert_eq!(r, (10, 20, 42));

    let r = parser.run_inner("cmd -a 10 -b 20").unwrap();
    assert_eq!(r, (10, 20, 42));

    let r = parser.run_inner("-a 10 cmd -b 20").unwrap();
    assert_eq!(r, (10, 20, 42));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a=A', and more\n");
}

#[test]
fn global_positional() {
    let g = positional::<String>("ARG").global();
    let sf = short('f').switch();
    let parser = construct!(g, sf).to_options();

    let r = parser.run_inner("-f hello").unwrap();
    assert_eq!(r, (String::from("hello"), true));

    let r = parser.run_inner("hello -f").unwrap();
    assert_eq!(r, (String::from("hello"), true));

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'ARG'\n");
}

#[test]
fn two_global_positionals() {
    let a = positional::<String>("A").global();
    let b = positional::<String>("B").global();
    let sf = short('f').switch();
    let parser = construct!(a, b, sf).to_options();

    let r = parser.run_inner("-f hello world").unwrap();
    assert_eq!(r, (String::from("hello"), String::from("world"), true));

    let r = parser.run_inner("hello -f world").unwrap();
    assert_eq!(r, (String::from("hello"), String::from("world"), true));

    let r = parser.run_inner("hello world -f").unwrap();
    assert_eq!(r, (String::from("hello"), String::from("world"), true));

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A', and more\n");
}

#[test]
fn global_any() {
    let g = any("ITEM", |s: &str| s.parse::<i32>().ok()).global();
    let p = pure(()).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("42 cmd").unwrap();
    assert_eq!(r, (42, ()));

    let r = parser.run_inner("cmd 42").unwrap();
    assert_eq!(r, (42, ()));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'ITEM'\n");
}

#[test]
fn two_global_anys() {
    let a = any("A", |s: &str| s.parse::<i32>().ok()).global();
    let b = any("B", |s: &str| s.parse::<i32>().ok()).global();
    let p = pure(()).to_options().command("cmd");
    let parser = construct!(a, b, p).to_options();

    let r = parser.run_inner("10 20 cmd").unwrap();
    assert_eq!(r, (10, 20, ()));

    let r = parser.run_inner("cmd 10 20").unwrap();
    assert_eq!(r, (10, 20, ()));

    let r = parser.run_inner("10 cmd 20").unwrap();
    assert_eq!(r, (10, 20, ()));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A', and more\n");
}

#[test]
fn global_literal() {
    let g = literal("lit").req_flag("lit").global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("lit cmd").unwrap();
    assert_eq!(r, ("lit", 42));

    let r = parser.run_inner("cmd lit").unwrap();
    assert_eq!(r, ("lit", 42));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'lit'\n");
}

#[test]
fn two_global_literals() {
    let a = literal("aaa").req_flag("aaa").global();
    let b = literal("bbb").req_flag("bbb").global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(a, b, p).to_options();

    let r = parser.run_inner("aaa bbb cmd").unwrap();
    assert_eq!(r, ("aaa", "bbb", 42));

    let r = parser.run_inner("cmd aaa bbb").unwrap();
    assert_eq!(r, ("aaa", "bbb", 42));

    let r = parser.run_inner("aaa cmd bbb").unwrap();
    assert_eq!(r, ("aaa", "bbb", 42));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'aaa', and more\n");
}

#[test]
fn global_command() {
    let g = pure(99).to_options().command("sub").global();
    let sf = short('f').switch().global();
    let parser = construct!(g, sf).to_options();

    let r = parser.run_inner("sub -f").unwrap();
    assert_eq!(r, (99, true));

    let r = parser.run_inner("-f sub").unwrap();
    assert_eq!(r, (99, true));

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'COMMAND ...'\n");
}

#[test]
fn global_req_flag_optional() {
    let g = short('r').req_flag('r').optional().global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-r cmd").unwrap();
    assert_eq!(r, (Some('r'), 42));

    let r = parser.run_inner("cmd -r").unwrap();
    assert_eq!(r, (Some('r'), 42));

    let r = parser.run_inner("cmd").unwrap();
    assert_eq!(r, (None, 42));
}

#[test]
fn global_req_flag_some() {
    let g = short('r').req_flag('r').some("need -r").global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-r cmd").unwrap();
    assert_eq!(r, (vec!['r'], 42));

    let r = parser.run_inner("-r -r cmd").unwrap();
    assert_eq!(r, (vec!['r', 'r'], 42));

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "need -r\n");
}

#[test]
fn global_arg_guard() {
    let g = short('r')
        .argument::<u32>("N")
        .guard(|v| *v < 10, "too big")
        .global();
    let p = pure(42).to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-r 3 cmd").unwrap();
    assert_eq!(r, (3, 42));

    let r = parser.run_inner("cmd -r 12").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-r 12': too big\n");

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-r=N'\n");
}

#[test]
fn global_arg_guard_inner() {
    let g = short('r')
        .argument::<u32>("N")
        .guard(|v| *v < 10, "too big")
        .global();
    let i = short('r').argument::<u32>("NN").optional();
    let p = i.to_options().command("cmd");
    let parser = construct!(g, p).to_options();

    let r = parser.run_inner("-r 3 cmd").unwrap();
    assert_eq!(r, (3, None));

    let r = parser.run_inner("cmd -r 12").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-r 12': too big\n");

    let r = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-r=N'\n");
}

#[test]
fn global_help_section() {
    let g = short('g').switch().help("A global flag").global();
    let i = short('i').switch().help("Inner flag");
    let o = short('o').switch().help("Outer flag");
    let cmd = i.to_options().command("cmd");
    let parser = construct!(o, g, cmd).to_options();

    let r = parser.run_inner("cmd --help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app cmd [-i]

Available options:
    -i          Inner flag
    -h, --help  Prints help information

Global options:
    -g          A global flag
";
    assert_eq!(r, expected);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app [-o] -g COMMAND ...

Available options:
    -o          Outer flag
    -h, --help  Prints help information

Available commands:
    cmd

Global options:
    -g          A global flag
";

    assert_eq!(r, expected);
}
