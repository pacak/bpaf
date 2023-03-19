#![allow(deprecated)]
use bpaf::*;

#[test]
fn help_with_default_parse() {
    set_override(false);
    use bpaf::Parser;
    #[derive(Debug, Clone, Bpaf)]
    enum Action {
        /// Add a new TODO item
        #[bpaf(command)]
        Add(String),

        /// Does nothing
        #[bpaf(command)]
        NoAction,
    }

    let parser = action().or_else(bpaf::pure(Action::NoAction)).to_options();

    let help = parser
        .run_inner(bpaf::Args::from(&["add", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: <ARG>

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(bpaf::Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    add        Add a new TODO item
    no_action  Does nothing
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
        .run_inner(bpaf::Args::from(&["add", "--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Add a new TODO item

Usage: <ARG>

Available options:
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);

    let help = parser
        .run_inner(bpaf::Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();

    let expected_help = "\
Usage: [COMMAND ...]

Available options:
    -h, --help  Prints help information

Available commands:
    add        Add a new TODO item
    no_action  Does nothing in two lines
";
    assert_eq!(expected_help, help);
}
