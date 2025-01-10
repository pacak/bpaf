use bpaf_core::*;

#[test]
fn simple_command() {
    let a = short('a').switch().to_options();

    let parser = a.command("alice").long("bob").to_options();

    let r = parser.run_inner(["-a"]).unwrap_err().unwrap_stderr();
    let expected = "`-a` is not valid in this context, did you mean to pass it to command `alice`?";
    assert_eq!(r, expected);
}

#[test]
fn alt_of_req_flags() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let p = construct!([a, b]).to_options();
    let r = p.run_inner(["-a", "-b"]).unwrap_err().unwrap_stderr();
    assert_eq!("`-b` cannot be used at the same time as `-a`", r);
}

#[test]
fn sum_of_flag_arg2() {
    let a = short('a').argument::<usize>("A");
    let b = short('a').req_flag('a').map(|_| 0);
    let p = construct!(a, b).to_options();
    // items are consumed left to right, so first -a is an argument
    let r = p.run_inner(["-a", "-b", "1"]).unwrap_err().unwrap_stderr();

    assert_eq!(r, "`-a` wants a value A, got `-b`, try using -a=-b");

    //assert_eq!((1, 0), run_parser(&p, ["-a", "1", "-a"]).unwrap());
}

#[test]
fn this_or_that_odd() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let ab = construct!(a, b);
    let a = short('a').req_flag(());
    let c = short('c').req_flag(());
    let cd = construct!(a, c);
    let parser = construct!([ab, cd]).to_options();

    let res = parser
        .run_inner(["-a", "-b", "-c"])
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(res, "`-c` cannot be used at the same time as `-b`");
}

#[test]
fn no_argument() {
    let a = short('a').argument::<i32>("N");
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(["-a", "-42"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-a` wants a value N, got `-42`, try using -a=-42");

    let r = parser.run_inner(["-a=-4", "-2"]).unwrap();
    assert_eq!(r, (-4, true));

    let r = parser.run_inner(["-a", "-2"]).unwrap_err().unwrap_stderr();

    assert_eq!(r, "`-a` wants a value N, got `-2`, try using -a=-2");
}

#[test]
fn cannot_be_used_partial_arg() {
    let a = short('a').req_flag(10);
    let b = short('b').argument::<usize>("ARG");
    let parser = construct!([a, b]).to_options();

    let res = parser.run_inner(["-b", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "`-b` wants a value ARG, got `-a`, try using -b=-a");

    let res = parser.run_inner(["-a", "-b"]).unwrap_err().unwrap_stderr();
    assert_eq!(res, "`-b` cannot be used at the same time as `-a`");
}

// #[test]
// fn better_error_with_enum() {
//     #[derive(Debug, Clone, Bpaf)]
//     #[bpaf(options)]
//     enum Foo {
//         Alpha,
//         Beta,
//         Gamma,
//     }
//
//     let res = foo()
//         .run_inner(&["--alpha", "--beta"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");
//
//     let res = foo()
//         .run_inner(&["--alpha", "--gamma"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(
//         res,
//         "`--gamma` cannot be used at the same time as `--alpha`"
//     );
//
//     let res = foo()
//         .run_inner(&["--beta", "--gamma"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(res, "`--gamma` cannot be used at the same time as `--beta`");
//
//     let res = foo()
//         .run_inner(&["--alpha", "--beta", "--gamma"])
//         .unwrap_err()
//         .unwrap_stderr();
//     assert_eq!(res, "`--beta` cannot be used at the same time as `--alpha`");
// }

#[test]
fn guard_message() {
    let parser = short('a')
        .argument::<u32>("N")
        .guard(|n| *n <= 10u32, "too high")
        .to_options();

    let res = parser.run_inner(["-a", "30"]).unwrap_err().unwrap_stderr();

    assert_eq!(res, "`30`: too high");
}

#[test]
fn strict_positional_argument() {
    let a = short('a').argument::<usize>("N");
    let parser = a.to_options();

    let r = parser
        .run_inner(["-a", "--", "10"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`-a` requires an argument `N`");
}

#[test]
fn not_expected_at_all() {
    let a = short('a').switch();
    let parser = a.to_options();

    let r = parser
        .run_inner(["--megapotato"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`--megapotato` is not expected in this context");

    let r = parser
        .run_inner(["megapotato"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`megapotato` is not expected in this context");
}

#[test]
fn cannot_be_used_twice() {
    let a = short('a').switch();
    let b = short('b').switch().many::<Vec<_>>();
    let parser = construct!(a, b).to_options();

    let r = parser
        .run_inner(["-a", "-b", "-a"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        "argument `-a` cannot be used multiple times in this context"
    );

    let r = parser.run_inner(["-a", "-a"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument `-a` cannot be used multiple times in this context"
    );

    let r = parser.run_inner(["-abba"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "argument `-a` cannot be used multiple times in this context"
    );
}

// #[test]
// fn should_not_split_adjacent_options() {
//     let a = short('a').req_flag(0);
//     let b = pure(()).to_options().command("hello");
//     let parser = construct!(a, b).to_options();
//     let r = parser.run_inner(&["-ahello"]).unwrap_err().unwrap_stderr();
//     // can probably suggest splitting here too: `-a` `hello`
//     let expected = "no such flag: `-ahello`, did you mean `hello`?";
//     assert_eq!(r, expected);
// }

// #[test]
// fn should_not_split_adjacent_ambig_options() {
//     let a1 = short('a').req_flag(0);
//     let a2 = short('a').argument::<usize>("x");
//     let a = construct!([a2, a1]);
//     let c = pure(()).to_options().command("hello");
//     let parser = construct!(a, c).to_options();
//
//     let r = parser.run_inner(&["-a=hello"]).unwrap_err().unwrap_stderr();
//     let expected = "expected `COMMAND ...`, got `hello`. Pass `--help` for usage information";
//     assert_eq!(r, expected);
//
//     let r = parser.run_inner(&["-ahello"]).unwrap_err().unwrap_stderr();
//     let expected = "app supports `-a` as both an option and an option-argument, try to split `-ahello` into individual\noptions (-a -h ..) or use `-a=hello` syntax to disambiguate";
//     assert_eq!(r, expected);
//
//     // this one is okay, try to parse -a as argument - it fails because "hello" is not a number, then
//     // try to parse -a as a flag - this works
//     let r = parser.run_inner(&["-a", "hello"]).unwrap();
//     assert_eq!(r, (0, ()));
// }

#[test]
fn adjacent_option_complains_to() {
    let parser = short('a').argument::<usize>("A").to_options();

    let r = parser.run_inner(["-ayam"]).unwrap_err().unwrap_stderr();

    // TODO - this should point to the whole "-ayam" thing
    assert_eq!(r, "couldn't parse `yam`: invalid digit found in string");
}

// #[test]
// fn some_pos_with_invalid_flag() {
//     let a = short('a').switch();
//     let b = positional::<usize>("B").some("Want B");
//     let parser = construct!(a, b).to_options();
//
//     let r = parser.run_inner(&["-c", "12"]).unwrap_err().unwrap_stderr();
//     assert_eq!(r, "`-c` is not expected in this context");
//
//     let r = parser.run_inner(&["12", "-c"]).unwrap_err().unwrap_stderr();
//     assert_eq!(r, "`-c` is not expected in this context");
// }

#[test]
fn pos_with_invalid_arg() {
    let a = short('b').long("alice").argument::<usize>("A").optional();
    let b = positional::<usize>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(["12", "-c"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`-c` is not expected in this context");

    let r = parser.run_inner(["-c", "12"]).unwrap_err().unwrap_stderr();
    let expected = "Parser expects a positional `B`, got a named `-c`. If you meant to use it as `B` - try inserting `--` in front of it";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(["-a=lice", "t"])
        .unwrap_err()
        .unwrap_stderr();
    let expected = "Parser expects a positional `B`, got a named `-a=lice`. If you meant to use it as `B` - try inserting `--` in front of it";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(["-alice", "t"])
        .unwrap_err()
        .unwrap_stderr();
    let expected = "no such flag: `-alice` (with one dash), did you mean `--alice`?";
    assert_eq!(r, expected);

    let r = parser.run_inner(["-c", "t"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `t`: invalid digit found in string");

    let r = parser
        .run_inner(["--alic", "t"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "no such flag: `--alic`, did you mean `--alice`?");

    let r = parser.run_inner(["t", "-c"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `t`: invalid digit found in string");
}

#[test]
fn strictly_positional_help() {
    let parser = long("hhhh").switch().to_options();

    let r = parser
        .run_inner(["--", "--help"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "`--help` is not expected in this context");
}

// #[test]
// fn hidden_required_field_is_valid_but_strange() {
//     let parser = short('a').req_flag(()).hide().to_options();
//
//     let r = parser.run_inner(&[]).unwrap_err().unwrap_stderr();
//
//     assert_eq!(r, "parser requires an extra flag, argument or parameter, but its name is hidden by the author");
// }

// #[test]
// fn guard_on_fallback() {
//     let parser = short('a')
//         .argument::<usize>("A")
//         .fallback(10)
//         .guard(|a| *a < 10, "too big")
//         .to_options();
//     let r = parser.run_inner(&[]).unwrap_err().unwrap_stderr();
//     assert_eq!(r, "check failed: too big");
// }

#[test]
fn two_required_fields_first_missing() {
    let a = long("a").argument::<u32>("A");
    let b = long("b").argument::<u32>("B");
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner(["--b", "1"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `--a=A`, pass `--help` for usage information");
}

#[test]
fn used_only_once_is_more_important_error() {
    let format = long("format").switch();
    let sort = long("sort").switch();
    let filter = long("filter").switch();
    let opts = construct!(format, sort, filter,).to_options();

    let err = opts
        .run_inner(["--filter", "--filter"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument `--filter` cannot be used multiple times in this context"
    );

    let err = opts
        .run_inner(["--sort", "--sort"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument `--sort` cannot be used multiple times in this context"
    );
    let err = opts
        .run_inner(["--sort", "--filter", "--sort", "--filter"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        err,
        "argument `--sort` cannot be used multiple times in this context"
    );
}
