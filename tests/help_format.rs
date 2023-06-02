use bpaf::*;

#[test]
fn fancy_meta() {
    let a = long("trailing-comma").argument::<String>("all|es5|none");
    let b = long("stdin-file-path").argument::<String>("PATH");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: --trailing-comma=<all|es5|none> --stdin-file-path=PATH

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
        .env("BPAF_VARIABLE")
        .argument::<String>("ARG")
        .to_options()
        .descr("descr\n descr")
        .header("header\n header")
        .footer("footer\n footer")
        .version("version")
        .usage("custom usage");

    let r = p.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
descr
descr

custom usage

header
header

Available options:
    -p, --parser=ARG  [env:BPAF_VARIABLE: N/A]
    -h, --help        Prints help information
    -V, --version     Prints version information

footer
footer
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c").switch();
    let c2 = short('c').help("c").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c
    -b
    -c          c
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_items_dif_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = short('c').help("c1").switch();
    let c2 = short('c').help("c2").switch();
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: (-a [-c] | -b [-c])

Available options:
    -a
    -c          c1
    -b
    -c          c2
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
// no longer deduplicated
fn duplicate_pos_items_same_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C");
    let c2 = positional::<String>("C").help("C");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: (-a C | -b C)

Available positional items:
    C           C
    C           C

Available options:
    -a
    -b
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn duplicate_pos_items_diff_help() {
    let a = short('a').req_flag(());
    let b = short('b').req_flag(());
    let c1 = positional::<String>("C").help("C1");
    let c2 = positional::<String>("C").help("C2");
    let ac = construct!(a, c1);
    let bc = construct!(b, c2);
    let parser = construct!([ac, bc]).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: (-a C | -b C)

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
    #[derive(Debug, Clone, Bpaf)]
    /// group help
    enum Mode {
        /// help
        ///
        /// absent
        Intel,

        /// help
        ///
        /// Hidden
        Att,
    }

    let r = mode()
        .to_options()
        .run_inner(&["--help"])
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
Usage: (--intel | --att)

Available options:
  group help
        --intel  help
        --att    help

    -h, --help   Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn anywhere_invariant_check() {
    #[derive(Debug, Clone, Bpaf)]
    #[allow(dead_code)]
    #[bpaf(adjacent)]
    struct Fooo {
        tag: (),
        #[bpaf(positional("NAME"))]
        /// help for name
        name: String,
        #[bpaf(positional("VAL"))]
        /// help for val
        val: String,
    }

    let a = short('a').help("help for a").switch();
    let b = short('b').help("help for b").switch();
    let parser = construct!(a, fooo(), b).to_options();

    let expected = "\
Usage: [-a] --tag NAME VAL [-b]

Available options:
    -a          help for a
  --tag NAME VAL
    NAME        help for name
    VAL         help for val

    -b          help for b
    -h, --help  Prints help information
";

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);

    // this shouldn't crash
    parser.check_invariants(true);
}

#[test]
fn multi_arg_help() {
    let a = short('f').long("flag").help("flag help").req_flag(());
    let b = short('e').long("extra").help("extra strange").switch();
    let c = positional::<String>("NAME").help("pos1 help");
    let d = positional::<bool>("STATE").help("pos2 help");
    let combo = construct!(a, b, c, d).adjacent().optional();
    let verbose = short('v').long("verbose").help("verbose").switch();
    let detailed = long("detailed").short('d').help("detailed").switch();
    let parser = construct!(verbose, combo, detailed).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [-v] [-f [-e] NAME STATE] [-d]

Available options:
    -v, --verbose   verbose
  -f [-e] NAME STATE
    -f, --flag      flag help
    -e, --extra     extra strange
    NAME            pos1 help
    STATE           pos2 help

    -d, --detailed  detailed
    -h, --help      Prints help information
";

    assert_eq!(r, expected);
}

#[test]
fn multi_pos_help() {
    let a = positional::<String>("NAME").help("name help");
    let b = positional::<String>("VAL").help("val help");
    let combo = construct!(a, b).adjacent();
    let verbose = short('v').long("verbose").switch();
    let parser = construct!(verbose, combo).to_options();
    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [-v] NAME VAL

Available positional items:
  NAME VAL
    NAME           name help
    VAL            val help

Available options:
    -v, --verbose
    -h, --help     Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn fallback_display_simple_arg() {
    let parser = long("a")
        .help("help for a")
        .argument("NUM")
        .fallback(42)
        .display_fallback()
        .to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "\
Usage: [--a=NUM]

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

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [NUM]

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

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [--a=NUM --b=NUM]

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

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "\
Usage: [--a=NUM]

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

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [--fonts=DIR] [--system-fonts]

Available options:
        --fonts=DIR     Load fonts from this directory
                        [env:OIKOS_FONTS: N/A]
        --system-fonts  Search for additional fonts in system directories
                        [env:OIKOS_SYSTEM_FONTS: not set]
    -h, --help          Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn with_group_help() {
    let a = short('a').help("option a").switch();
    let b = short('b').help("option b").switch();
    let c = short('c').help("option c").switch();

    let ab = construct!(a, b).with_group_help(|meta| {
        let mut b = Doc::default();
        b.emphasis("Uses either of those ");
        b.meta(meta, false);
        b
    });
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "\
Usage: [-a] [-b] [-c]

Available options:
  Uses either of those [-a] [-b]
    -a          option a
    -b          option b

    -c          option c
    -h, --help  Prints help information
";

    assert_eq!(r, expected);

    let r = parser.run_inner(&["-a", "-c"]).unwrap();
    assert_eq!(r, ((true, false), true));
}

#[test]
fn custom_help_and_version() {
    let h = short('H').long("halp").help("halps you");
    let v = short('v').long("release").help("prints release id");
    let a = short('a').switch();
    let parser = a.to_options().help_parser(h).version_parser(v);

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "`--help` is not expected in this context");

    let r = parser.run_inner(&["--halp"]).unwrap_err().unwrap_stdout();
    let expected = "Usage: [-a]\n\nAvailable options:\n    -a\n    -H, --halp  halps you\n";
    assert_eq!(r, expected);
}
