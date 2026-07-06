use crate::*;

#[test]
fn this_or_that_odd() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let ab = construct!(a, b);
    let a = short('a').req_flag(());
    let c = short('c').req_flag(());
    let cd = construct!(a, c);
    let parser = construct!([ab, cd]).to_options();

    let res = parser.run_inner("-a -b -c").unwrap_err().unwrap_stderr();
    assert_eq!(res, "'-c' cannot be used at the same time as '-b'\n");
}

#[test]
fn unsigned_argument() {
    let a = short('a').argument::<u32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -2").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'-a' requires an argument 'N', got '-2', try '-a=-2' to use it as an argument\n"
    );

    let r = parser.run_inner("-2 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' expects a value 'N'\n");

    // -2 is a valid flag, -42 is not
    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'-a' requires an argument 'N', got '-42', try '-a=-42' to use it as an argument\n"
    );

    let r = parser.run_inner("-a=-42 -2").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '-42': invalid digit found in string\n");
}

#[test]
fn signed_argument() {
    let a = short('a').argument::<i32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -2").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'-a' requires an argument 'N', got '-2', try '-a=-2' to use it as an argument\n"
    );

    let r = parser.run_inner("-2 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' expects a value 'N'\n");

    // -2 is a valid flag, -42 is not
    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'-a' requires an argument 'N', got '-42', try '-a=-42' to use it as an argument\n"
    );

    let r = parser.run_inner("-a=-42 -2").unwrap();
    assert_eq!(r, (-42, true));
}

#[test]
fn cannot_be_used_partial_arg() {
    let a = short('a').req_flag(10);
    let b = short('b').argument::<usize>("ARG");
    let parser = construct!([a, b]).to_options();

    let res = parser.run_inner("-a -b").unwrap_err().unwrap_stderr();
    assert_eq!(res, "'-b' cannot be used at the same time as '-a'\n");

    let res = parser.run_inner("-b -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        res,
        "'-b' requires an argument 'ARG', got '-a', try '-b=-a' to use it as an argument\n"
    );
}

#[test]
fn option_requires_other_option_v1() {
    let a = short('a').switch();
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).optional().to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-b=B'\n");
}

#[test]
// same as '_v1', legacy test - order was much more previously
fn option_requires_other_option_v2() {
    let a = short('a').switch();
    let b = short('b').argument::<String>("B");
    let parser = construct!(b, a).optional().to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-b=B'\n");
}

#[test]
fn better_error_with_enum() {
    #[derive(Debug, Clone, Copy)]
    enum Foo {
        Alpha,
        Beta,
        Gamma,
    }
    let alpha = long("alpha").req_flag(Foo::Alpha);
    let beta = long("beta").req_flag(Foo::Beta);
    let gamma = long("gamma").req_flag(Foo::Gamma);
    let foo = construct!([alpha, beta, gamma]).to_options();

    let res = foo
        .run_inner(["--alpha", "--beta"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "'--beta' cannot be used at the same time as '--alpha'\n"
    );

    let res = foo
        .run_inner(["--alpha", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "'--gamma' cannot be used at the same time as '--alpha'\n"
    );

    let res = foo
        .run_inner(["--beta", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "'--gamma' cannot be used at the same time as '--beta'\n"
    );

    let res = foo
        .run_inner(["--alpha", "--beta", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "'--beta' cannot be used at the same time as '--alpha'\n"
    );
}

#[test]
fn guard_on_arg() {
    let parser = short('a')
        .argument::<u32>("N")
        .guard(|n| *n <= 10u32, "too high")
        .to_options();

    let res = parser.run_inner("-a 30").unwrap_err().unwrap_stderr();

    assert_eq!(res, "'-a 30': too high\n");
}

#[test]
fn guard_on_pair() {
    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b)
        .guard(|(a, b)| a + b < 10, "too high")
        .to_options();

    // this can't include range since two independent leaf nodes can be nowhere
    // near each other. Good error message here relies on user
    let res = parser.run_inner("-a 10 -b 20").unwrap_err().unwrap_stderr();
    assert_eq!(res, "too high\n");
}

#[test]
fn strict_positional_argument() {
    let a = short('a').argument::<usize>("N");
    let parser = a.to_options();

    // '-' and '--' are positional items
    // TODO - old version (and smarter parsers) treat it as absent
    let r = parser.run_inner("-a -- 10").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '--', try '-a=--' to use it as an argument\n";
    assert_eq!(r, expected);
}

#[test]
fn passing_ddash_to_arg_works() {
    let a = short('a').argument::<String>("DD");
    let parser = a.to_options();

    let r = parser.run_inner("-a --").unwrap_err().unwrap_stderr();

    let expected =
        "'-a' requires an argument 'DD', got '--', try '-a=--' to use it as an argument\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=--").unwrap();
    assert_eq!(r, "--");
}

#[test]
fn not_expected_at_all() {
    let a = short('a').switch();
    let parser = a.to_options();

    let r = parser
        .run_inner("--megapotato")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "'--megapotato' is not expected in this context\n");

    let r = parser.run_inner("megapotato").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'megapotato' is not expected in this context\n");
}

#[test]
fn cannot_be_used_twice() {
    let a = short('a').switch();
    let b = short('b').switch().many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument '-a' cannot be used multiple times in this context\n"
    );

    let r = parser.run_inner("-a -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument '-a' cannot be used multiple times in this context\n"
    );

    let r = parser.run_inner("-abba").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "can't parse 'a' (item 4) while parsing '-abba' as a set of stacked short flags\n"
    );
}

#[test]
fn should_not_split_adjacent_options() {
    let a = short('a').req_flag(0);
    let b = pure(()).to_options().command("hello");
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner("-ahello").unwrap_err().unwrap_stderr();
    // can probably suggest splitting here too: '-a' 'hello'
    let expected = "the app can accept '-a' as a flag, but got '-ahello'\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("hell").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such command: 'hell', did you mean 'hello'?\n");
}

#[test]
fn should_not_split_adjacent_ambig_options() {
    let a1 = short('a').req_flag(0);
    let a2 = short('a').argument::<usize>("x");
    let a = construct!([a1, a2]);
    let c = pure(()).to_options().command("hello");
    let parser = construct!(a, c).to_options();

    // this happens inside of the command context - a is not known.
    let r = parser.run_inner("hello -a 3").unwrap_err().unwrap_stderr();
    let expected = "'-a' is not expected in this context\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-ahello").unwrap_err().unwrap_stderr();
    let expected = "the app can accept '-a' as a flag, but got '-ahello'\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=hello").unwrap_err().unwrap_stderr();
    let expected = "the app can accept '-a' as a flag, but got '-a=hello'\n";
    assert_eq!(r, expected);

    // this one is okay, try to parse -a as argument - it fails because "hello" is not a number, then
    // try to parse -a as a flag - this works
    let r = parser.run_inner("-a hello").unwrap();
    assert_eq!(r, (0, ()));
}

#[test]
fn adjacent_option_complains_to() {
    let parser = short('a').argument::<usize>("A").to_options();

    let r = parser.run_inner("-ayam").unwrap_err().unwrap_stderr();

    // TODO - this should point to the whole "-ayam" thing
    assert_eq!(r, "couldn't parse 'yam': invalid digit found in string\n");
}

#[test]
fn missing_flag() {
    let a = short('a').req_flag(());
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a'\n");
}

#[test]
fn missing_arg() {
    let a = short('a').argument::<usize>("A");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a=A'\n");
}

#[test]
fn command_with_req_parameters() {
    let p = positional::<String>("X")
        .to_options()
        .command("cmd")
        .fallback(String::new())
        .to_options();

    let r = p.run_inner("").unwrap();
    assert_eq!(r, "");

    let r = p.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'X'\n");

    let r = p.run_inner("cmd bob").unwrap();
    assert_eq!(r, "bob");
}

#[test]
fn missing_pos() {
    let a = positional::<usize>("A");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A'\n");
}

#[test]
fn missing_cmd() {
    let a = pure(()).to_options().command("cmd");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'COMMAND ...'\n");
}

#[test]
fn some_pos_with_invalid_flag() {
    let a = short('a').switch();
    let b = positional::<usize>("B").some("You have to specify at least one B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-c 12").unwrap_err().unwrap_stderr();
    assert_eq!(r, "You have to specify at least one B\n");

    let r = parser.run_inner("12 -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-c' is not expected in this context\n");
}

#[test]
fn pos_with_invalid_arg() {
    let a = short('a').argument::<usize>("A").optional();
    let b = positional::<usize>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-c 12").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'B', got '-c'\n");

    let r = parser.run_inner("12 -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-c' is not expected in this context\n");

    let r = parser.run_inner("-c t").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'B', got '-c'\n");

    let r = parser.run_inner("t -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 't': invalid digit found in string\n");
}

#[test]
fn strictly_positional_help() {
    let parser = long("hhhh").switch().to_options();
    let r = parser.run_inner("-- --help").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'--help' is not expected in this context\n");
}

#[test]
fn double_dash_is_pos_only_just_once() {
    let parser = positional::<String>("POS").many().to_options();

    let r = parser.run_inner("--").unwrap();
    assert_eq!(r, Vec::<String>::new());

    let r = parser.run_inner("-- --").unwrap();
    assert_eq!(r, vec!["--".to_string()]);
}

#[test]
fn hidden_required_field_is_valid_but_strange() {
    // hidden stuff shows up in error messages when it is needed
    // to explain stuff, but not in help or usage
    let parser = short('a').req_flag(()).hide().to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-a'\n");

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app\n\nAvailable options:\n    -h, --help  Prints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn guard_on_fallback() {
    let parser = short('a')
        .argument::<usize>("A")
        .fallback(10)
        .guard(|a| *a < 10, "too big")
        .to_options();
    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "too big\n");
}

#[test]
fn two_required_fields_first_missing() {
    let a = long("a").argument::<u32>("A");
    let b = long("b").argument::<u32>("B");
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner("--b 1").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '--a=A'\n");
}

#[test]
fn used_only_once_is_more_important_error() {
    let format = long("format").switch();
    let sort = long("sort").switch();
    let filter = long("filter").switch();
    let parser = construct!(format, sort, filter).to_options();

    let err = parser
        .run_inner("--filter --filter")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument '--filter' cannot be used multiple times in this context\n"
    );

    let err = parser
        .run_inner("--sort --sort")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument '--sort' cannot be used multiple times in this context\n"
    );
}

#[test]
fn ambiguity() {
    #[derive(Debug, PartialEq)]
    enum A {
        V(Vec<bool>),
        W(String),
    }

    let a0 = short('a').switch().many().map(A::V);
    let a1 = short('a').argument::<String>("AAAAAA").map(A::W);
    let parser = construct!([a0, a1]).to_options();

    // argument parser wins since it consumes everything at once
    let r = parser.run_inner("-aaaaaa").unwrap();
    assert_eq!(r, A::W("aaaaa".into()));

    let r = parser.run_inner("-b").unwrap_err().unwrap_stderr();
    // single char typos are too random
    assert_eq!(r, "'-b' is not expected in this context\n");
}

#[test]
fn ambiguity_2() {
    #[derive(Debug, PartialEq)]
    enum A {
        V(Vec<bool>),
        W(String),
    }

    let a0 = short('a').switch().many().map(A::V);
    let a1 = short('a')
        .argument::<String>("AAAAAA")
        .map(A::W)
        .guard(|_| false, "nope");
    let parser = construct!([a0, a1]).to_options();

    let r = parser.run_inner("-aaaaaa").unwrap_err().unwrap_stderr();
    // TODO - this is actually ambiguity error asking you to split it
    assert_eq!(r, "the app can accept '-a' as a flag, but got '-aaaaaa'\n");
}

#[test]
fn reject_fbar() {
    let parser = short('f').argument::<String>("F").to_options();

    let r = parser.run_inner("-fbar baz").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'baz' is not expected in this context\n");

    let r = parser.run_inner("-fbar").unwrap();
    assert_eq!(r, "bar");
}

#[test]
fn short_cmd() {
    let parser = long("alpha")
        .req_flag(())
        .to_options()
        .command("beta")
        .short('b')
        .to_options();

    let r = parser.run_inner("bet").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such command: 'bet', did you mean 'beta'?\n");

    let r = parser.run_inner("c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'COMMAND ...', got 'c'\n");
}

#[test]
fn did_you_mean_inside_command() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = construct!([a, b]).to_options().command("cmd").to_options();

    let r = parser.run_inner("cmd --fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--fla', did you mean '--flag'?\n");

    let r = parser
        .run_inner("cmd --flag --parametr")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--parametr', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("cmd --parametr --flag")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--parametr', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("cmd --parameter --flag")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "'--flag' cannot be used at the same time as '--parameter'\n"
    );

    let r = parser
        .run_inner("cmd --flag --parameter")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "'--parameter' cannot be used at the same time as '--flag'\n"
    );

    let r = parser.run_inner("--fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'COMMAND ...', got '--fla'\n");
}

#[test]
fn double_dashes_fallback() {
    let a = long("llvm").req_flag(()).optional();
    let parser = a.to_options();

    let r = parser.run_inner("-llvm").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: '-llvm' (with one dash), did you mean '--llvm'?\n"
    );
}

#[test]
fn double_dashes_no_fallback() {
    let a = long("llvm").req_flag(());
    let parser = a.to_options();

    let r = parser.run_inner("-llvm").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: '-llvm' (with one dash), did you mean '--llvm'?\n"
    );
}

#[test]
fn suggestion_for_equals_1() {
    let parser = short('p').long("par").argument::<String>("P").to_options();

    let r = parser.run_inner("-p --bar").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'-p' requires an argument 'P', got '--bar', try '-p=--bar' to use it as an argument\n"
    );

    let r = parser.run_inner("--par --bar").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "'--par' requires an argument 'P', got '--bar', try '--par=--bar' to use it as an argument\n"
    );

    let r = parser
        .run_inner("--par --bar=baz")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "'--par' requires an argument 'P', got '--bar=baz', try '--par=--bar=baz' to use it as an argument\n"
    );
}

#[test]
fn inside_out_command_parser() {
    let parser = long("oneline")
        .switch()
        .to_options()
        .command("cmd")
        .to_options();

    let r = parser.run_inner("cmd --oneline").unwrap();
    assert!(r);

    // Can't parse "--oneline log" because oneline could be an argument instead of a flag
    // so log might not be a command, but we can try to make a better suggestion.
    let r = parser
        .run_inner("--oneline log")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "flag '--oneline' is not valid in this context, did you mean to pass it to command 'cmd'?\n"
    );
}

#[test]
fn double_dash_with_optional_positional() {
    let a = long("llvm").req_flag(());
    let pos = positional::<String>("FILE").optional();
    let parser = construct!(pos, a).to_options();

    let r = parser.run_inner("make -llvm").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '-llvm' (with one dash), did you mean '--llvm'?\n"
    );
}

// #[test]
// fn inside_out_command_parser() {
//     #[derive(Debug, Bpaf, Clone, PartialEq)]
//     #[bpaf(options)]
//     enum Cmd {
//         #[bpaf(command)]
//         Log {
//             #[bpaf(long)]
//             oneline: bool,
//         },
//     }
//
//     let ok = cmd().run_inner(&["log", "--oneline"]).unwrap();
//     assert_eq!(ok, Cmd::Log { oneline: true });
//
//     // Can't parse "--oneline log" because oneline could be an argument instead of a flag
//     // so log might not be a command, but we can try to make a better suggestion.
//     let r = cmd()
//         .run_inner(&["--oneline", "log"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(
//         r,
//         "flag '--oneline' is not valid in this context, did you mean to pass it to command 'log'?"
//     );
// }
//
//

#[test]
fn nested_in_flag() {
    let key = positional::<String>("key").help("config key");
    let val = positional::<bool>("bool").help("config value");
    let inner = construct!(key, val);
    let set = long("setBool").help("Set <key> to <value>").nest(inner);

    let aa = long("bool-flag").switch();
    let parser = construct!(set, aa).to_options();

    let r = parser
        .run_inner("--setBool key tru")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        // everything before ":" comes from bpaf, after ":" - it's an error specific
        // to FromStr instance.
        "couldn't parse 'tru': provided string was not `true` or `false`\n"
    );

    let r = parser.run_inner("--bool-fla").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: '--bool-fla', did you mean '--bool-flag'?\n"
    );

    let r = parser.run_inner("--setBoo").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--setBoo', did you mean '--setBool'?\n");

    let r = parser
        .run_inner("--bool-flag --bool-flag")
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "argument '--bool-flag' cannot be used multiple times in this context\n"
    );
}

#[test]
fn nested_in_keyword() {
    let key = positional::<String>("key").help("config key");
    let val = positional::<bool>("bool").help("config value");
    let inner = construct!(key, val);
    let set = literal("setBool").help("Set <key> to <value>").nest(inner);

    let aa = long("bool-flag").switch();
    let parser = construct!(set, aa).to_options();

    let r = parser
        .run_inner("setBool key tru")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        // everything before ":" comes from bpaf, after ":" - it's an error specific
        // to FromStr instance.
        "couldn't parse 'tru': provided string was not `true` or `false`\n"
    );

    let r = parser.run_inner("--bool-fla").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: '--bool-fla', did you mean '--bool-flag'?\n"
    );

    let r = parser.run_inner("setBoo").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'setBool', got 'setBoo'\n"); // TODO - improve?

    let r = parser
        .run_inner("--bool-flag --bool-flag")
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "argument '--bool-flag' cannot be used multiple times in this context\n"
    );
}

#[test]
fn suggest_typo_fix() {
    let p = long("flag").switch().to_options();

    let r = p.run_inner("--fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--fla', did you mean '--flag'?\n");

    let r = p.run_inner("--fla --fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--fla', did you mean '--flag'?\n");

    let r = p.run_inner("--flag --flag").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument '--flag' cannot be used multiple times in this context\n"
    );
}

#[test]
fn did_you_mean_argument() {
    let parser = long("flag").argument::<String>("VAL").to_options();

    let res = parser.run_inner("--fla").unwrap_err().unwrap_stderr();
    assert_eq!(res, "no such flag: '--fla', did you mean '--flag'?\n");

    let res = parser
        .run_inner("--flg=hellop")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "no such flag: '--flg', did you mean '--flag'?\n");
}

// #[test]
// fn better_error_message_with_typos() {
//     #[derive(Bpaf, Clone, Debug)]
//     #[bpaf(options)]
//     enum Commands {
//         /// Multi
//         ///  Line
//         ///  Comment
//         #[bpaf(command)]
//         Lines {},
//
//         #[bpaf(command)]
//         Arguments(#[bpaf(external(arguments))] Arguments),
//     }
//
//     #[derive(Bpaf, Clone, Debug)]
//     struct Arguments {
//         #[bpaf(short('e'), argument("Arg"))]
//         env: Vec<String>,
//
//         #[bpaf(positional("POS"))]
//         args: Vec<String>,
//     }
//
//     let r = arguments()
//         .to_options()
//         .run_inner(&["-a", "erg"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(r, "'-a' is not expected in this context");
//
//     let r = commands()
//         .run_inner(&["arguments", "-a", "erg"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(r, "'-a' is not expected in this context");
//
//     let r = arguments()
//         .to_options()
//         .run_inner(&["--help"])
//         .unwrap_err()
//         .unwrap_stdout();
//     let expected = "\
// Usage: [-e=<Arg>]... [POS]...
//
// Available options:
//     -e=<Arg>
//     -h, --help  Prints help information
// ";
//     assert_eq!(r, expected);
//
//     let r = commands()
//         .run_inner(&["--help"])
//         .unwrap_err()
//         .unwrap_stdout();
//     let expected = "\
// Usage: COMMAND ...
//
// Available options:
//     -h, --help  Prints help information
//
// Available commands:
//     lines       Multi
//     arguments
// ";
//     assert_eq!(r, expected);
// }
//

#[test]
fn big_conflict() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = short('d').switch();
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!([ab, cd]).to_options();

    let r = parser.run_inner("-a -b -c -d").unwrap_err().unwrap_stderr();
    let expected = "'-c' cannot be used at the same time as '-a'\n";
    assert_eq!(r, expected);
}

#[test]
fn conflict_flag_pos() {
    let a = short('a').flag(1, 0);
    let b = positional::<usize>("B");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    assert_eq!(r, "can't parse '42', likely conflicts with '-a'\n");

    let r = parser.run_inner("42 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '42'\n");
}

#[test]
fn conflict_flag_command() {
    let a = short('a').flag(1, 0);
    let b = pure(42).to_options().command("42").lazy();
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("42 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' cannot be used at the same time as '42'\n");

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    assert_eq!(r, "can't parse '42', likely conflicts with '-a'\n");
}

#[test]
fn conflict_pos_command() {
    let a = pure(42).to_options().command("42").lazy();
    let b = positional::<usize>("B");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("32 42").unwrap_err().unwrap_stderr();
    assert_eq!(r, "can't parse '42', likely conflicts with '32'\n");

    // 42 is both a valid positional and a valid command name, so both succeed
    let r = parser.run_inner("42 32").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'32' is not expected in this context\n");
}

#[test]
fn this_shouldnt_pass() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = short('d').switch();
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!([ab, cd]).to_options();
    let r = parser.run_inner("-abcd").unwrap_err().unwrap_stderr();

    // TODO: Can I make it "'-c' cannot be used at the same time as '-a'"?
    let expected =
        "can't parse 'c' (item 3) while parsing '-abcd' as a set of stacked short flags\n";
    assert_eq!(r, expected);
}

// TODO - move to primitive parsers?
#[test]
fn pure_conflicts() {
    // pure goes first
    let a = short('a').flag('a', 'b');
    let b = pure('c');
    let parser = construct!([b, a]).to_options();

    // flag consumed - it takes priority
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');
    // same consumed length, pure takes priority since it's first
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'c');

    // flag goes first
    let a = short('a').flag('a', 'b');
    let b = pure('c');
    let parser = construct!([a, b]).to_options();

    // flag consumed - it takes priority
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');
    // same consumed length, flag takes priority since it's first
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn pure_works() {
    let parser = pure('b').to_options();
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'b');

    let r = parser.run_inner("-b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-b' is not expected in this context\n");
}

#[test]
fn pair_of_pos() {
    let a = positional::<i32>("A");
    let b = positional::<f32>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("4.0 33").unwrap_err().unwrap_stderr();

    let expected = "couldn't parse '4.0': invalid digit found in string\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("33 4.0").unwrap();
    assert_eq!(r, (33, 4.0));
}

#[test]
fn strict_pos_msg() {
    let a = positional::<i32>("A").strict();
    let parser = a.to_options();

    let r = parser.run_inner("32").unwrap_err().unwrap_stderr();
    let expected = "expected '32' (A) to follow '--'\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-- -32").unwrap();
    assert_eq!(r, -32);
}

#[test]
fn lazy_command_conflict() {
    let a = short('a').switch().to_options().command("alpha").lazy();
    let b = short('b').switch().to_options().command("beta").lazy();
    let c = short('c').switch();

    let ab = a.or_else(b);
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner("alpha beta").unwrap_err().unwrap_stderr();
    let expected = "can't parse 'beta', likely conflicts with 'alpha'\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("alpha -c").unwrap();
    assert_eq!(r, (false, true));
}

#[test]
fn literal_conflict() {
    let a = literal("alpha").flag('a', 'A');
    let b = literal("beta").flag('b', 'B');
    let parser = a.or_else(b).to_options();

    let r = parser.run_inner("alpha beta").unwrap_err().unwrap_stderr();
    let expected = "can't parse 'beta', likely conflicts with 'alpha'\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("beta").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn conflict_with_argument() {
    let nn = long("noname").req_flag(None);
    let n = long("name").argument::<String>("NAME").map(Some);
    let parser = n.or_else(nn).to_options();

    let r = parser
        .run_inner("--name Bob --noname")
        .unwrap_err()
        .unwrap_stderr();
    let expected = "'--noname' cannot be used at the same time as '--name'\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner("--noname --name Bob")
        .unwrap_err()
        .unwrap_stderr();
    let expected = "'--name' cannot be used at the same time as '--noname'\n";
    assert_eq!(r, expected);
}

#[test]
fn no_misleading_no_such_flag() {
    // <flox/flox#3411>
    // Simulates an enum with alternative variants (e.g. derive enum with or_else)
    let edit_manifest = long("file").argument::<String>("FILE").optional();

    let rename = long("name").argument::<String>("NAME").map(Some);

    let parser = construct!([edit_manifest, rename]).to_options();

    let r = parser.run_inner("--name").unwrap_err().unwrap_stderr();
    let expected = "'--name' expects a value 'NAME'\n";
    assert_eq!(r, expected);
}

#[test]
fn any_keeps_track_of_current_value() {
    let parser = any::<&str, usize>("N", |s: &str| s.parse::<usize>().ok())
        .parse(|n: usize| if n > 100 { Err("too large") } else { Ok(n) })
        .to_options();

    let r = parser.run_inner("200").unwrap_err().unwrap_stderr();
    let expected = "couldn't parse '200': too large\n";
    assert_eq!(r, expected);
}

#[test]
fn conflicts_with_any_are_okay() {
    let a = any::<&str, usize>("A", |s: &str| s.parse::<usize>().ok());
    let b = short('f').flag(1, 0);
    let parser = a.or_else(b).to_options();

    let r = parser.run_inner("-f").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("15").unwrap();
    assert_eq!(r, 15);
}

#[test]
fn pos1_vs_pos3() {
    let a = positional::<usize>("A").map(|x| x * 10);
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = positional::<usize>("d");
    let bcd = construct!(b, c, d).map(|(b, c, d)| b + c + d);
    let parser = construct!([a, bcd]).to_options();

    let r = parser.run_inner("2 3 4").unwrap_err().unwrap_stderr();
    // This error message can be confusing, but that's a fault of the parser.
    // While 'bcd' can succeed - the moment we parse "2" with both 'a' and 'b'
    // we must made a decision what to kill or keep running - without knowing
    // what parsers are left. To avoid data loss 'a' must return the result.
    assert_eq!(r, "can't parse '3', likely conflicts with '2'\n");

    let r = parser.run_inner("2 3").unwrap_err().unwrap_stderr();
    // this test case illustrates the problem that the previous case is trying to avoid.
    // Suppose we didn't produce the result with 'a' and proceeded parsing with 'c'.
    // So far so good, but then we've reached the end of the input. 'bcd' can't succeed
    // but 'a' can't succeed either since it didn't consume "3".
    assert_eq!(r, "can't parse '3', likely conflicts with '2'\n");

    let r = parser.run_inner("1").unwrap();
    assert_eq!(r, 10);
}

#[test]
fn pos2_vs_pos3() {
    let a = positional::<usize>("A");
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = positional::<usize>("D");
    let e = positional::<usize>("E");
    let ab = construct!(a, b).map(|(a, b)| a + b);
    let cde = construct!(c, d, e).map(|(c, d, e)| c + d + e);
    let parser = construct!([ab, cde]).to_options();

    // let r = parser.run_inner("2 3 4").unwrap_err().unwrap_stderr();
    // // This error message can be confusing, but that's a fault of the parser.
    // // While 'bcd' can succeed - the moment we parse "2" with both 'a' and 'b'
    // // we must made a decision what to kill or keep running - without knowing
    // // what parsers are left. To avoid data loss 'a' must return the result.
    // assert_eq!(r, "'3' cannot be used at the same time as '2'\n");

    let r = parser.run_inner("1 2").unwrap();
    // this test case illustrates the problem that the previous case is trying to avoid.
    // Suppose we didn't produce the result with 'a' and proceeded parsing with 'c'.
    // So far so good, but then we've reached the end of the input. 'bcd' can't succeed
    // but 'a' can't succeed either since it didn't consume "3".
    assert_eq!(r, 3);

    let r = parser.run_inner("1 2 3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "can't parse '3', likely conflicts with '2'\n");
}

#[test]
fn missing_product() {
    let a = positional::<usize>("A");
    let b = positional::<usize>("B");
    let parser = (a, b).to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A', and more\n");
}

#[test]
fn missing_sum() {
    let a = positional::<usize>("A");
    let b = positional::<usize>("B");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A', or more\n");
}

#[test]
fn missing_mix() {
    let a = positional::<usize>("A");
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = positional::<usize>("D");
    let ab = (a, b);
    let cd = (c, d);
    let parser = construct!([ab, cd]).to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'A', and more\n");
}

#[test]
fn missing_some_takes_priority_and_unaffected() {
    let a = positional::<usize>("A").map(|x| vec![x]);
    let b = short('v').req_flag(0).some("several verbosities");
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "several verbosities\n");
}

#[test]
fn no_fallback_out_of_command_parser() {
    let alt1 = positional::<String>("NAME").to_options().command("cmd");
    let alt2 = pure(String::new());
    let parser = construct!([alt1, alt2]).to_options();

    let res = parser.run_inner("cmd").unwrap_err().unwrap_stderr();
    assert_eq!(res, "expected 'NAME'\n");

    let res = parser.run_inner("cmd a").unwrap();
    assert_eq!(res, "a");

    let res = parser.run_inner("").unwrap();
    assert_eq!(res, "");
}

#[test]
fn did_you_mean_two_and_arguments() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = cargo_helper("cmd", construct!(a, b)).to_options();

    let r = parser
        .run_inner("--flag --parametr")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--parametr', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("--flag --paramet=value")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--paramet', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("--parameter --flg")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: '--flg', did you mean '--flag'?\n");

    let r = parser.run_inner("--fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--fla', did you mean '--flag'?\n");
}

#[test]
fn did_you_mean_two_or_arguments() {
    let a = long("flag").switch();
    let b = long("parameter").switch();
    let parser = cargo_helper("cmd", construct!([a, b])).to_options();

    let r = parser.run_inner("--fla").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--fla', did you mean '--flag'?\n");

    let r = parser
        .run_inner("--flag --parametr")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--parametr', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("--parametr --flag")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "no such flag: '--parametr', did you mean '--parameter'?\n"
    );

    let r = parser
        .run_inner("--parameter --flag")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "'--flag' cannot be used at the same time as '--parameter'\n"
    );

    let r = parser
        .run_inner("--flag --parameter")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "'--parameter' cannot be used at the same time as '--flag'\n"
    );

    let r = parser.run_inner("cmd --flag").unwrap();
    assert!(r);

    let r = parser.run_inner("--flag").unwrap();
    assert!(r);

    let r = parser.run_inner("cm").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'cm' is not expected in this context\n");
}

#[test]
fn argument_missing_value_or_else_winner_consumed() {
    // When the winning branch DID consume the flag, the error should NOT propagate.
    // '--name' as a switch vs '--name' as an argument.
    let flag_parser = long("name").switch();
    let arg_parser = long("name").argument::<String>("NAME").map(|_| false);
    let parser = construct!([flag_parser, arg_parser]).to_options();

    let r = parser.run_inner("--name").unwrap();
    assert!(r);
}

#[test]
fn argument_missing_value_with_catch() {
    // .optional().catch() should not swallow NoArgument - the user explicitly
    // provided the flag, so the missing value should always be reported.
    let name = long("name").argument::<usize>("NAME").optional().catch();
    let file = long("file").argument::<usize>("FILE").optional();
    let parser = construct!(name, file).to_options();

    let r = parser.run_inner("--name").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'--name' expects a value 'NAME'\n");

    let r = parser.run_inner("--name nope").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'nope': invalid digit found in string\n");
}

#[test]
fn cargo_show_asm_issue_guard() {
    let target_dir = short('t').argument::<String>("T").guard(|_| false, "nope");
    let verbosity = short('v').switch();
    let inner = construct!(target_dir, verbosity);
    let parser = cargo_helper("asm", inner).to_options();

    let res = parser.run_inner("asm -t x").unwrap_err().unwrap_stderr();
    assert_eq!(res, "'-t x': nope\n");

    let res = parser.run_inner("-t x").unwrap_err().unwrap_stderr();
    assert_eq!(res, "'-t x': nope\n");
}

#[test]
fn cargo_show_asm_issue_from_str() {
    let target_dir = short('t').argument::<usize>("T");
    let verbosity = short('v').switch();
    let inner = construct!(target_dir, verbosity);
    let parser = cargo_helper("asm", inner).to_options();

    let res = parser.run_inner("asm -t x").unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse 'x': invalid digit found in string\n");

    let res = parser.run_inner("-t x").unwrap_err().unwrap_stderr();
    assert_eq!(res, "couldn't parse 'x': invalid digit found in string\n");
}

#[test]
fn better_error_message_with_typos() {
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum Commands {
        Lines {},
        Arguments(Arguments),
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct Arguments {
        env: Vec<String>,
        args: Vec<String>,
    }
    let args = positional::<String>("POS")
        .help("Multi\n Line\n Comments")
        .many();
    let env = short('e').argument("Arg").many();

    let arguments = construct!(Arguments { env, args }).into_rc();

    let parser = arguments
        .clone()
        .to_options()
        .command("arguments")
        .map(Commands::Arguments)
        .or_else(
            pure(Commands::Lines {})
                .to_options()
                .command("lines")
                .help("Multi\n Line\n Comment"),
        )
        .to_options();

    let r = parser.run_inner("-a erg").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'COMMAND ...', got '-a'\n");

    let r = parser.run_inner("-e erg").unwrap_err().unwrap_stderr();
    let expected =
        "flag '-e' is not valid in this context, did you mean to pass it to command 'arguments'?\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner("arguments -a erg")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "'-a' is not expected in this context\n");

    let r = arguments
        .to_options()
        .run_inner("--help")
        .unwrap_err()
        .unwrap_stdout();
    let expected = "Usage: app [-e=<Arg>]... [POS]...

Available positional items:
    POS         Multi
                Line
                Comments

Available options:
    -e=<Arg>
    -h, --help  Prints help information
";

    assert_eq!(r, expected);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "Usage: app COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    arguments
    lines       Multi
                Line
                Comment
";
    assert_eq!(r, expected);
}

#[test]
fn nested_flag_error() {
    let inner = short('i').flag('i', 'I');
    let a = short('a').nest(inner);
    let b = short('b').flag('b', 'B');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-i").unwrap_err().unwrap_stderr();
    let expected = "flag '-i' is not valid in this context, but it can be used after '-a'\n";
    assert_eq!(r, expected);
}

#[test]
fn nested_keyword_error() {
    let inner = short('i').flag('i', 'I');
    let kw = literal("all").nest(inner);
    let b = short('b').flag('b', 'B');
    let parser = construct!([kw, b]).to_options();

    let r = parser.run_inner("-i").unwrap_err().unwrap_stderr();
    let expected = "flag '-i' is not valid in this context, but it can be used after 'all'\n";
    assert_eq!(r, expected);
}

#[test]
fn anchor_start_error() {
    let a = long("alpha").switch().anchor_start();
    let b = long("beta").switch();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner("--alph --beta")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: '--alph', did you mean '--alpha'?\n");

    let r = parser
        .run_inner("--beta --alph")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "'--alph' is not expected in this context\n");

    let r = parser
        .run_inner("--beta --alpha")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "'--alpha' is not expected in this context\n");

    let r = parser.run_inner("--alpha").unwrap();
    assert_eq!(r, (true, false));

    let r = parser.run_inner("--beta").unwrap();
    assert_eq!(r, (false, true));
}
