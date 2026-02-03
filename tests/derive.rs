#![deny(missing_docs)]

//! hi
use bpaf::{Bpaf, Parser};

#[test]
fn help_with_default_parse() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options, fallback(Action::CheckConnection))]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(
            /// Item to track
            #[bpaf(positional("ITEM"))]
            String,
        ),

        /// Test connection to the server
        #[bpaf(command)]
        CheckConnection,
    }

    let parser = action();

    let help = parser
        .run_inner(&["add", "--help"])
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: add ITEM

Available positional items:
    ITEM        Item to track

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
Usage: [COMMAND ...]

Available options:
    -h, --help        Prints help information

Available commands:
    add               Add a new TODO item
    check-connection  Test connection to the server
";
    assert_eq!(expected_help, help);
}

#[test]
fn command_and_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(String),

        /// Does nothing
        /// in two lines
        #[bpaf(command)]
        NoAction,
    }

    use bpaf::Parser;
    let parser = action().fallback(Action::NoAction).to_options();

    let help = parser
        .run_inner(&["add", "--help"])
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: add ARG

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    let expected_help = "\
Usage: [COMMAND ...]

Available options:
    -h, --help  Prints help information

Available commands:
    add         Add a new TODO item
    no-action   Does nothing
";
    assert_eq!(expected_help, help);
}

#[test]
fn single_unit_command() {
    #[derive(Bpaf, Debug, Clone, Eq, PartialEq)]
    #[bpaf(command)]
    struct One;

    let parser = one().to_options();
    let help = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();
    let expected = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    one
";
    assert_eq!(help, expected);

    let r = parser.run_inner(&["one"]).unwrap();
    assert_eq!(r, One);
}

#[test]
fn pure_optional() {
    #[derive(Bpaf, Debug, Clone)]
    #[bpaf(options)]
    struct Opts {
        #[bpaf(pure(Default::default()))]
        foo: Option<Vec<u32>>,
    }

    assert_eq!(opts().run().foo, None);
}
