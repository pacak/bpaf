use crate::*;

#[test]
fn smallest() {
    let a = short('a').help("A simple flag").req_flag(());
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -a          A simple flag
    -h, --help  Prints help information
";

    assert_eq!(r, expected)
}

#[test]
fn simple_flag() {
    let a = short('a').help("A simple flag").req_flag(());
    let parser = a
        .to_options()
        .header("This is a header")
        .descr("This is a description")
        .footer("And this is a footer");

    let r = parser.run_inner("-a --help").unwrap_err().unwrap_stdout();
    let expected = "\
This is a description

Usage: app -a

This is a header

Available options:
    -a          A simple flag
    -h, --help  Prints help information

And this is a footer
";

    assert_eq!(r, expected);
}

#[test]
fn complex_descr() {
    let descr = "\
fooo

    bar1
    bar2
    bar3

baz";
    let a = short('a').switch();
    let parser = a.to_options().descr(descr);
    let r = parser.run_inner("-hh").unwrap_err().unwrap_stdout();
    let expected = "\
fooo

    bar1
    bar2
    bar3

baz

Usage: app [-a]

Available options:
    -a
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_after_switch() {
    let parser = short('a').switch().help("this is help").to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [-a]

Available options:
    -a          this is help
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn fallback_to_usage() {
    let a = short('a')
        .argument::<usize>("A")
        .to_options()
        .fallback_to_usage();

    let r = a.run_inner("").unwrap_err().unwrap_stdout();
    let expected = "Usage: app -a=A\n\nAvailable options:\n    -a=A\n    -h, --help  Prints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn fallback_to_usage_nested() {
    let a = short('a')
        .argument::<usize>("A")
        .to_options()
        .fallback_to_usage()
        .descr("help for cmd")
        .command("cmd")
        .short('c')
        .to_options();

    let r = a.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    c, cmd      help for cmd
";
    assert_eq!(r, expected);

    let r = a.run_inner("cmd").unwrap_err().unwrap_stdout();
    let expected = "\
help for cmd

Usage: app cmd -a=A

Available options:
    -a=A
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn fancy_meta() {
    let a = long("trailing-comma").argument::<String>("all|es5|none");
    let b = long("stdin-file-path").argument::<String>("PATH");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app --trailing-comma=<all|es5|none> --stdin-file-path=PATH

Available options:
        --trailing-comma=<all|es5|none>
        --stdin-file-path=PATH
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn decorations() {
    let p = short('p')
        .long("parser")
        .help("help")
        .env("BPAF_VARIABLE")
        .argument::<String>("ARG")
        .to_options()
        .descr("descr\n descr")
        .header("header\n header")
        .footer("footer\n footer")
        .version("version")
        .usage("custom usage");

    let r = p.run_inner("--help").unwrap_err().unwrap_stdout();

    println!("{r}");
    let expected = "\
descr
descr

Usage: app custom usage

header
header

Available options:
    -p, --parser=ARG  help
                      [env:BPAF_VARIABLE: N/A]
    -h, --help        Prints help information
    -V, --version     Prints version information

footer
footer
";

    assert_eq!(r, expected);
}

#[test]
fn very_long_switch() {
    let a = short('p')
        .long("ppppppppppppppppppppppppppppppppppppp")
        .help("this is help for megapotato")
        .argument::<usize>("MEGAPOTATO");
    let b = short('b').long("batman").help("help for batman").switch();
    let p = construct!(a, b).to_options();

    let r = p.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -p=MEGAPOTATO [-b]

Available options:
    -p, --ppppppppppppppppppppppppppppppppppppp=MEGAPOTATO  this is help for megapotato
    -b, --batman  help for batman
    -h, --help    Prints help information
";
    assert_eq!(r, expected);
}

// #[test]
// fn duplicate_items_same_help() {
//     let a = short('a').req_flag(());
//     let b = short('b').req_flag(());
//     let c1 = short('c').help("c").switch();
//     let c2 = short('c').help("c").switch();
//     let ac = construct!(a, c1);
//     let bc = construct!(b, c2);
//     let parser = construct!([ac, bc]).to_options();
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//
//     let expected = "\
// Usage: app (-a [-c] | -b [-c])
//
// Available options:
//     -a
//     -c          c
//     -b
//     -h, --help  Prints help information
// ";
//
//     assert_eq!(r, expected);
// }

#[test]
fn duplicate_items_dif_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c1").switch();
    let c2 = short('c').help("c2").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app (-a [-c] | -b [-c])

Available options:
    -a
    -c          c1
    -b
    -c          c2
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

// #[test]
// fn duplicate_pos_items_same_help() {
//     let a = short('a').req_flag(());
//     let b = short('b').req_flag(());
//     let c1 = positional::<String>("C").help("C");
//     let c2 = positional::<String>("C").help("C");
//     let ac = construct!(a, c1);
//     let bc = construct!(b, c2);
//     let parser = construct!([ac, bc]).to_options();
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//
//     let expected = "\
// Usage: app (-a C | -b C)
//
// Available positional items:
//     C           C
//
// Available options:
//     -a
//     -b
//     -h, --help  Prints help information
// ";
//
//     assert_eq!(r, expected);
// }

#[test]
fn duplicate_pos_items_diff_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C1");
    let c2 = positional::<String>("C").help("C2");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app (-a C | -b C)

Available positional items:
    C           C1
    C           C2

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn enum_with_docs() {
    #[derive(Debug, Clone)]
    enum Mode {
        Intel,
        Att,
    }
    let intel = long("intel").help("help\n\nabsent").req_flag(Mode::Intel);
    let att = long("att").help("help\n\nHidden").req_flag(Mode::Att);
    let mode = construct!([intel, att]).group_help("Pick mode:");

    let r = mode
        .to_options()
        .run_inner("--help")
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: app (--intel | --att)

Pick mode:
        --intel  help
        --att    help

Available options:
    -h, --help   Prints help information
";
    assert_eq!(r, expected);
}

// #[test]
// fn anywhere_invariant_check() {
//     #[derive(Debug, Clone, Bpaf)]
//     #[allow(dead_code)]
//     #[bpaf(adjacent)]
//     struct Fooo {
//         tag: (),
//         #[bpaf(positional("NAME"))]
//         /// help for name
//         name: String,
//         #[bpaf(positional("VAL"))]
//         /// help for val
//         val: String,
//     }
//
//     let a = short('a').help("help for a").switch();
//     let b = short('b').help("help for b").switch();
//     let parser = construct!(a, fooo(), b).to_options();
//
//     let expected = "\
// Usage: app [-a] --tag NAME VAL [-b]
//
// Available options:
//     -a          help for a
//   --tag NAME VAL
//     NAME        help for name
//     VAL         help for val
//
//     -b          help for b
//     -h, --help  Prints help information
// ";
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//     assert_eq!(r, expected);
//
//     // this shouldn't crash
//     parser.check_invariants(true);
// }

// #[test]
// fn multi_arg_help() {
//     let a = short('f').long("flag").help("flag help").req_flag(());
//     let b = short('e').long("extra").help("extra strange").switch();
//     let c = positional::<String>("NAME").help("pos1 help");
//     let d = positional::<bool>("STATE").help("pos2 help");
//     let combo = construct!(a, b, c, d).adjacent().optional();
//     let verbose = short('v').long("verbose").help("verbose").switch();
//     let detailed = long("detailed").short('d').help("detailed").switch();
//     let parser = construct!(verbose, combo, detailed).to_options();
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//
//     let expected = "\
// Usage: app [-v] [-f [-e] NAME STATE] [-d]
//
// Available options:
//     -v, --verbose   verbose
//   -f [-e] NAME STATE
//     -f, --flag      flag help
//     -e, --extra     extra strange
//     NAME            pos1 help
//     STATE           pos2 help
//
//     -d, --detailed  detailed
//     -h, --help      Prints help information
// ";
//
//     assert_eq!(r, expected);
// }

// #[test]
// fn multi_pos_help() {
//     let a = positional::<String>("NAME").help("name help");
//     let b = positional::<String>("VAL").help("val help");
//     let combo = construct!(a, b).adjacent();
//     let verbose = short('v').long("verbose").switch();
//     let parser = construct!(verbose, combo).to_options();
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//
//     let expected = "\
// Usage: app [-v] NAME VAL
//
// Available positional items:
//   NAME VAL
//     NAME           name help
//     VAL            val help
//
// Available options:
//     -v, --verbose
//     -h, --help     Prints help information
// ";
//     assert_eq!(r, expected);
// }

#[test]
fn fallback_display_simple_arg() {
    let parser = long("a")
        .help("help for a")
        .argument("NUM")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [--a=NUM]

Available options:
        --a=NUM  help for a
                 [default: 42]
    -h, --help   Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_simple_pos() {
    let parser = positional("NUM")
        .help("help for pos")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [NUM]

Available positional items:
    NUM         help for pos
                [default: 42]

Available options:
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_tuple() {
    #[derive(Copy, Clone, Debug)]
    struct Pair(u32, u32);
    impl std::fmt::Display for Pair {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Pair {}, {}", self.0, self.1)
        }
    }

    let a = long("a").help("help for a").argument("NUM");
    let b = long("b").help("help for b").argument("NUM");
    let parser = construct!(a, b)
        .map(|(a, b)| Pair(a, b))
        .fallback(Pair(42, 333))
        .display_fallback()
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [--a=NUM --b=NUM]

Available options:
        --a=NUM  help for a
        --b=NUM  help for b
                 [default: Pair 42, 333]
    -h, --help   Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn fallback_display_no_help() {
    let parser = long("a")
        .argument("NUM")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [--a=NUM]

Available options:
        --a=NUM
                 [default: 42]
    -h, --help   Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn env_fallback_visible() {
    let fonts_dir = long("fonts")
        .env("OIKOS_FONTS")
        .help("Load fonts from this directory")
        .argument::<String>("DIR")
        .optional();

    let system_fonts = long("system-fonts")
        .env("OIKOS_SYSTEM_FONTS")
        .help("Search for additional fonts in system directories")
        .switch();
    let parser = construct!(fonts_dir, system_fonts).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [--fonts=DIR] [--system-fonts]

Available options:
        --fonts=DIR     Load fonts from this directory
                        [env:OIKOS_FONTS: N/A]
        --system-fonts  Search for additional fonts in system directories
                        [env:OIKOS_SYSTEM_FONTS is not set]
    -h, --help          Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn fallback_format_simple_arg() {
    let parser = long("a")
        .help("help for a")
        .argument("NUM")
        .fallback(42)
        .format_fallback(|i, f| write!(f, "**{i}**"))
        .to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [--a=NUM]

Available options:
        --a=NUM  help for a
                 [default: **42**]
    -h, --help   Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn nested_group_help() {
    let a = short('a').help("help for a").switch().group_help("inner");
    let b = short('b').help("help for b").switch();

    let parser = construct!(a, b).group_help("outer").to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    println!("{r}");
    let expected = "\
Usage: app [-a] [-b]

outer
    -a          help for a
    -b          help for b

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

// #[test]
// fn with_group_help() {
//     let a = short('a').help("option a").switch();
//     let b = short('b').help("option b").switch();
//     let c = short('c').help("option c").switch();
//
//     let ab = construct!(a, b).with_group_help(|meta| {
//         let mut b = Doc::default();
//         b.emphasis("Uses either of those ");
//         b.meta(meta, false);
//         b
//     });
//     let parser = construct!(ab, c).to_options();
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
//     let expected = "\
// Usage: app [-a] [-b] [-c]
//
// Uses either of those [-a] [-b]
//     -a          option a
//     -b          option b
//
// Available options:
//     -c          option c
//     -h, --help  Prints help information
// ";
//
//     assert_eq!(r, expected);
//
//     let r = parser.run_inner(&["-a", "-c"]).unwrap();
//     assert_eq!(r, ((true, false), true));
// }
//
// #[test]
// fn custom_help_and_version() {
//     let h = short('H').long("halp").help("halps you");
//     let v = short('v').long("release").help("prints release id");
//     let a = short('a').switch();
//     let parser = a.to_options().help_parser(h).version_parser(v);
//
//     let r = parser.run_inner("--help").unwrap_err().unwrap_stderr();
//     assert_eq!(r, "`--help` is not expected in this context");
//
//     let r = parser.run_inner(&["--halp"]).unwrap_err().unwrap_stdout();
//     let expected = "Usage: app [-a]\n\nAvailable options:\n    -a\n    -H, --halp  halps you\n";
//     assert_eq!(r, expected);
// }
//
#[test]
fn various_name_lengths_under() {
    let parser = short('a')
        .long("123456789012345")
        .help("A")
        .switch()
        .to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [-a]

Available options:
    -a, --123456789012345  A
    -h, --help             Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn various_name_lengths_at() {
    let parser = short('a')
        .long("1234567890123456")
        .help("A")
        .switch()
        .to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [-a]

Available options:
    -a, --1234567890123456  A
    -h, --help              Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn various_name_lengths_over1() {
    let parser = short('a')
        .long("12345678901234567")
        .help("A")
        .switch()
        .to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [-a]

Available options:
    -a, --12345678901234567  A
    -h, --help               Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn various_name_lengths_over2() {
    let parser = short('a')
        .long("1234567890123456789")
        .help("A")
        .switch()
        .to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [-a]

Available options:
    -a, --1234567890123456789  A
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_and_version_newline() {
    let parser = short('a').switch().to_options().version("1");

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: app [-a]\n\nAvailable options:\n    -a\n    -h, --help     Prints help information\n    -V, --version  Prints version information\n"
    );

    let r = parser.run_inner("--version").unwrap_err().unwrap_stdout();
    assert_eq!(r, "Version: 1\n");
}

#[test]
fn fallback_to_usage_and_commands() {
    let parser = pure(())
        .to_options()
        .descr("inner")
        .command("cmd")
        .to_options()
        .descr("outer")
        .fallback_to_usage();

    let r = parser.run_inner("cmd --help").unwrap_err().unwrap_stdout();
    let expected = "\
inner

Usage: app cmd

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(&[]).unwrap_err().unwrap_stdout();
    let expected = "\
outer

Usage: app COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    cmd         inner
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_flag() {
    let a = short('a').help("Do A").req_flag('a');
    let halp = short('H')
        .long("halp")
        .help("Verbose help!")
        .req_flag(crate::info::Help::Full);
    let parser = a.to_options().help_parser(halp).fallback_to_usage();

    let r = parser.run_inner("--halp").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -a          Do A
    -H, --halp  Verbose help!
";
    assert_eq!(r, expected);

    let r = parser.run_inner("").unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stderr();
    let expected = "`--help` is not expected in this context\n";
    assert_eq!(r, expected);
}

#[test]
fn custom_version() {
    let a = short('a').switch();
    let parser = a.to_options().version("3.14");

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [-a]

Available options:
    -a
    -h, --help     Prints help information
    -V, --version  Prints version information
";
    assert_eq!(r, expected);

    let r = parser.run_inner("--version").unwrap_err().unwrap_stdout();
    let expected = "Version: 3.14\n";
    assert_eq!(r, expected);
}

#[test]
fn custom_version_flag() {
    let a = short('a').switch();
    let vf = short('v')
        .long("ver")
        .help("For version")
        .req_flag(())
        .then_exit(|_| success("v 3.14"));
    let parser = a.to_options().version("3.14").version_parser(vf);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app [-a]

Available options:
    -a
    -h, --help  Prints help information
    -v, --ver   For version
";
    assert_eq!(r, expected);

    let r = parser.run_inner("--ver").unwrap_err().unwrap_stdout();
    let expected = "v 3.14\n";
    assert_eq!(r, expected);
}
