use bpaf::*;

#[test]
fn ambiguity() {
    set_override(false);
    #[derive(Debug, Clone)]
    enum A {
        V(Vec<bool>),
        W(String),
    }

    let a0 = short('a').switch().many().map(A::V);
    let a1 = short('a').argument::<String>("AAAAAA").map(A::W);
    let parser = construct!([a0, a1]).to_options();

    let r = parser
        .run_inner(Args::from(&["-aaaaaa"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Parser supports -a as both option and option-argument, try to split -aaaaaa into individual options (-a -a ..) or use -a=aaaaa syntax to disambiguate");

    let r = parser
        .run_inner(Args::from(&["-b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `-b`, did you mean `-a`?");
}

#[test]
fn short_cmd() {
    set_override(false);
    let parser = long("alpha")
        .req_flag(())
        .to_options()
        .command("beta")
        .short('b')
        .to_options();

    let r = parser
        .run_inner(Args::from(&["c"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(r, "No such command: `c`, did you mean `b`?");
}

#[test]
fn double_dashes_no_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options)]
    enum Opts {
        Llvm,
        Att,
        #[bpaf(hide)]
        Dummy,
    }

    let r = opts()
        .run_inner(Args::from(&["-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    // TODO: can we point out at -llvm here?
    assert_eq!(
        r,
        "Expected (--llvm | --att), pass --help for usage information"
    );
}

#[test]
fn double_dashes_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options, fallback(Opts::Dummy))]
    enum Opts {
        Llvm,
        Att,
        Dummy,
    }

    let r = opts()
        .run_inner(Args::from(&["-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such flag: -llvm (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn double_dash_with_optional_positional() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(fallback(Opts::Dummy))]
    enum Opts {
        Llvm,
        Att,
        Dummy,
    }

    let pos = positional::<String>("FILE").optional();
    let parser = construct!(opts(), pos).to_options();

    let r = parser
        .run_inner(Args::from(&["make", "-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such flag: -llvm (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn inside_out_command_parser() {
    #![allow(dead_code)]

    #[derive(Debug, Bpaf, Clone, PartialEq)]
    #[bpaf(options)]
    enum Cmd {
        #[bpaf(command)]
        Log {
            #[bpaf(long)]
            oneline: bool,
        },
    }

    let ok = cmd().run_inner(Args::from(&["log", "--oneline"])).unwrap();
    assert_eq!(ok, Cmd::Log { oneline: true });

    // Can't parse "--oneline log" because oneline could be an argument instead of a flag
    // so log might not be a command, but we can try to make a better suggestion.
    let r = cmd()
        .run_inner(Args::from(&["--oneline", "log"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "flag: `--oneline` is not valid in this context, did you mean to pass it to command \"log\"?");
}
