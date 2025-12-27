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
    assert_eq!(res, "`-c` cannot be used at the same time as `-b`");
}

#[test]
fn unsigned_argument() {
    let a = short('a').argument::<u32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -2").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "`-a` requires an argument TODO, got a flag -2, try -a=-2"
    );

    let r = parser.run_inner("-2 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` expects a value");

    // -2 is a valid flag, -42 is not
    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "`-a` requires an argument TODO, got a flag -42, try -a=-42"
    );

    let r = parser.run_inner("-a=-42 -2").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `-42`: invalid digit found in string");
}

#[test]
fn signed_argument() {
    let a = short('a').argument::<i32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -2").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "`-a` requires an argument TODO, got a flag -2, try -a=-2"
    );

    let r = parser.run_inner("-2 -a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` expects a value");

    // -2 is a valid flag, -42 is not
    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "`-a` requires an argument TODO, got a flag -42, try -a=-42"
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
    assert_eq!(res, "`-b` cannot be used at the same time as `-a`");

    let res = parser.run_inner("-b -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        res,
        "`-b` requires an argument TODO, got a flag -a, try -b=-a"
    );
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
    assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");

    let res = foo
        .run_inner(["--alpha", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        res,
        "`--gamma` cannot be used at the same time as `--alpha`"
    );

    let res = foo
        .run_inner(["--beta", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`--gamma` cannot be used at the same time as `--beta`");

    let res = foo
        .run_inner(["--alpha", "--beta", "--gamma"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");
}

#[test]
fn guard_on_arg() {
    let parser = short('a')
        .argument::<u32>("N")
        .guard(|n| *n <= 10u32, "too high")
        .to_options();

    let res = parser.run_inner("-a 30").unwrap_err().unwrap_stderr();

    assert_eq!(res, "`-a 30`: too high");
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
    assert_eq!(res, "too high");
}

#[test]
fn strict_positional_argument() {
    let a = short('a').argument::<usize>("N");
    let parser = a.to_options();

    let r = parser.run_inner("-a -- 10").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `--`: invalid digit found in string");
}

#[test]
fn not_expected_at_all() {
    let a = short('a').switch();
    let parser = a.to_options();

    let r = parser
        .run_inner("--megapotato")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`--megapotato` is not expected in this context");

    let r = parser.run_inner("megapotato").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`megapotato` is not expected in this context");
}

#[test]
fn cannot_be_used_twice() {
    let a = short('a').switch();
    let b = short('b').switch().many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -b -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument `-a` cannot be used multiple times in this context"
    );

    let r = parser.run_inner("-a -a").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument `-a` cannot be used multiple times in this context"
    );

    let r = parser.run_inner("-abba").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "can't parse `a` (item 4) while parsing `-abba` as a set of short flags"
    );
}

#[test]
fn should_not_split_adjacent_options() {
    let a = short('a').req_flag(0);
    let b = pure(()).to_options().command("hello");
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner("-ahello").unwrap_err().unwrap_stderr();
    // can probably suggest splitting here too: `-a` `hello`
    let expected = "the app can accept `-a` as a flag, but got `-ahello`";
    assert_eq!(r, expected);
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
    let expected = "`-a` is not expected in this context";
    assert_eq!(r, expected);

    let r = parser.run_inner("-ahello").unwrap_err().unwrap_stderr();
    let expected = "the app can accept `-a` as a flag, but got `-ahello`";
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
    assert_eq!(r, "couldn't parse `yam`: invalid digit found in string");
}

#[test]
fn missing_flag() {
    let a = short('a').req_flag(());
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `-a`");
}

#[test]
fn missing_arg() {
    let a = short('a').argument::<usize>("A");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `-a A`");
}

#[test]
fn missing_pos() {
    let a = positional::<usize>("A");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `A`");
}

#[test]
fn missing_cmd() {
    let a = pure(()).to_options().command("cmd");
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `cmd`");
}

#[test]
fn some_pos_with_invalid_flag() {
    let a = short('a').switch();
    let b = positional::<usize>("B").some("Want B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(&["-c", "12"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");

    let r = parser.run_inner(&["12", "-c"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");
}

#[test]
fn pos_with_invalid_arg() {
    let a = short('a').argument::<usize>("A").optional();
    let b = positional::<usize>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-c 12").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");

    let r = parser.run_inner("12 -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");

    let r = parser.run_inner("-c t").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");

    let r = parser.run_inner("t -c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `t`: invalid digit found in string");
}

#[test]
fn strictly_positional_help() {
    let parser = long("hhhh").switch().to_options();
    let r = parser
        .run_inner(&["--", "--help"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`--help` is not expected in this context");
}

#[test]
fn hidden_required_field_is_valid_but_strange() {
    // hidden stuff shows up in error messages when it is needed
    // to explain stuff, but not in help or usage
    let parser = short('a').req_flag(()).hide().to_options();
    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `-a`");
}

#[test]
fn guard_on_fallback() {
    let parser = short('a')
        .argument::<usize>("A")
        .fallback(10)
        .guard(|a| *a < 10, "too big")
        .to_options();
    let r = parser.run_inner(&[]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "too big");
}

#[test]
fn two_required_fields_first_missing() {
    let a = long("a").argument::<u32>("A");
    let b = long("b").argument::<u32>("B");
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner("--b 1").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `--a A`");
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
        "argument `--filter` cannot be used multiple times in this context"
    );

    let err = parser
        .run_inner("--sort --sort")
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument `--sort` cannot be used multiple times in this context"
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
    let r = parser.run_inner(&["-aaaaaa"]).unwrap();
    assert_eq!(r, A::W("aaaaa".into()));

    let r = parser.run_inner(&["-b"]).unwrap_err().unwrap_stderr();
    // single char typos are too random
    assert_eq!(r, "`-b` is not expected in this context");
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
    assert_eq!(r, "the app can accept `-a` as a flag, but got `-aaaaaa`");
}

#[test]
fn short_cmd() {
    let parser = long("alpha")
        .req_flag(())
        .to_options()
        .command("beta")
        //        .short('b') // TODO
        .to_options();

    let r = parser.run_inner("bet").unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such command: `bet`, did you mean `beta`?");

    let r = parser.run_inner("c").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`c` is not expected in this context");
}

#[test]
fn double_dashes_fallback() {
    let a = long("llvm").req_flag(()).optional();
    let parser = a.to_options();

    let r = parser.run_inner("-llvm").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: `-llvm` (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn double_dashes_no_fallback() {
    let a = long("llvm").req_flag(());
    let parser = a.to_options();

    let r = parser.run_inner("-llvm").unwrap_err().unwrap_stderr();

    assert_eq!(
        r,
        "no such flag: `-llvm` (with one dash), did you mean `--llvm`?"
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
        "no such flag: `-llvm` (with one dash), did you mean `--llvm`?"
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
//         "flag `--oneline` is not valid in this context, did you mean to pass it to command `log`?"
//     );
// }
//
//
// #[test]
// fn ux_discussion() {
//     #[derive(Debug, Clone, Bpaf)]
//     #[bpaf(adjacent)]
//     pub struct ConfigSetBool {
//         /// Set <key> to <bool>
//         #[bpaf(long("setBool"))]
//         set_bool: (),
//         /// Configuration key
//         #[bpaf(positional("key"))]
//         key: String,
//         /// Configuration Value (bool)
//         #[bpaf(positional("bool"))]
//         value: bool,
//     }
//
//     let aa = long("bool-flag").switch();
//     let parser = construct!(config_set_bool(), aa).to_options();
//
//     let r = parser
//         .run_inner(&["--setBool", "key", "tru"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(
//         r,
//         // everything before ":" comes from bpaf, after ":" - it's an error specific
//         // to FromStr instance.
//         "couldn't parse `tru`: provided string was not `true` or `false`"
//     );
//
//     let r = parser
//         .run_inner(&["--bool-fla"])
//         .unwrap_err()
//         .unwrap_stderr();
//
//     assert_eq!(r, "no such flag: `--bool-fla`, did you mean `--bool-flag`?");
//
//     let r = parser
//         .run_inner(&["--bool-flag", "--bool-flag"])
//         .unwrap_err()
//         .unwrap_stderr();
//
//     assert_eq!(
//         r,
//         "expected `--setBool`, got `--bool-flag`. Pass `--help` for usage information"
//     );
// }
//
#[test]
fn suggest_typo_fix() {
    let p = long("flag").switch().to_options();

    let r = p.run_inner(&["--fla"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: `--fla`, did you mean `--flag`?");

    let r = p
        .run_inner(&["--fla", "--fla"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--fla`, did you mean `--flag`?");

    let r = p
        .run_inner(&["--flag", "--flag"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "argument `--flag` cannot be used multiple times in this context"
    );
}
//
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
//     assert_eq!(r, "`-a` is not expected in this context");
//
//     let r = commands()
//         .run_inner(&["arguments", "-a", "erg"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(r, "`-a` is not expected in this context");
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
    let expected = "`-c` cannot be used at the same time as `-a`";
    assert_eq!(r, expected);
}

#[test]
fn conflict_flag_pos_and_command() {
    let a = short('a').flag(1, 0);
    let b = positional::<usize>("B");
    let c = pure(42).to_options().command("second").lazy();
    let parser = construct!([a, b, c]).to_options();

    let r = parser.run_inner("second -a").unwrap_err().unwrap_stderr();
    let expected = "`-a` cannot be used at the same time as `second`";
    assert_eq!(r, expected);

    let r = parser.run_inner("23 -a").unwrap_err().unwrap_stderr();
    let expected = "`-a` cannot be used at the same time as `23`";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a 23").unwrap_err().unwrap_stderr();
    let expected = "`23` cannot be used at the same time as `-a`";
    assert_eq!(r, expected);
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

    // TODO: Can I make it "`-c` cannot be used at the same time as `-a`"?
    let expected = "can't parse `c` (item 3) while parsing `-abcd` as a set of short flags";
    assert_eq!(r, expected);
}

// TODO - move to primitive parsers?
#[test]
fn pure_conflicts() {
    // pure goes first
    let a = short('a').flag('a', 'b');
    let b = pure('c');
    let parser = construct!([b, a]).to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'c');
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'c');

    // flag goes first
    let a = short('a').flag('a', 'b');
    let b = pure('c');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'b');
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');
}

#[test]
fn pure_works() {
    let parser = pure('b').to_options();
    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'b');

    let r = parser.run_inner("-b").unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-b` is not expected in this context");
}

#[test]
fn pair_of_pos() {
    let a = positional::<i32>("A");
    let b = positional::<f32>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("3.14 33").unwrap_err().unwrap_stderr();

    let expected = "couldn't parse `3.14`: invalid digit found in string";
    assert_eq!(r, expected);

    let r = parser.run_inner("33 3.14").unwrap();
    assert_eq!(r, (33, 3.14));
}
