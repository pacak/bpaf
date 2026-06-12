use bpaf::*;

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
    -h, --help  Prints help information
";

    assert_eq!(r, expected);
}

#[test]
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
    /// Pick mode:
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

Pick mode:
        --intel  help
        --att    help

Available options:
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

Uses either of those [-a] [-b]
    -a          option a
    -b          option b

Available options:
    -c          option c
    -h, --help  Prints help information
";

    assert_eq!(r, expected);

    let r = parser.run_inner(&["-a", "-c"]).unwrap();
    assert_eq!(r, ((true, false), true));
}
