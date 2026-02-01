use crate::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Opt {
    Fetch {
        dry_run: bool,
        all: bool,
        repository: String,
    },
    Add {
        interactive: bool,
        all: bool,
        files: Vec<PathBuf>,
    },
}

fn setup() -> OptionParser<Opt> {
    let dry_run = long("dry_run").switch();
    let all = long("all").switch();
    let repository = positional::<String>("SRC").fallback("origin".to_string());
    let fetch = construct!(Opt::Fetch {
        dry_run,
        all,
        repository
    });
    let fetch_inner = fetch
        .to_options()
        .descr("fetches branches from remote repository");
    let fetch_cmd = fetch_inner.command("fetch");

    let interactive = short('i').switch();
    let all = long("all").switch();
    let files = positional::<PathBuf>("FILE").many();
    let add = construct!(Opt::Add {
        interactive,
        all,
        files
    });
    let add_inner = add.to_options().descr("add files to the staging area");
    let add_cmd = add_inner.command("add");

    construct!([fetch_cmd, add_cmd])
        .to_options()
        .descr("The stupid content tracker")
}

#[test]
fn no_command() {
    let parser = setup();

    let expected_err = "missing 'COMMAND ...'\n";
    assert_eq!(
        expected_err,
        parser.run_inner("").unwrap_err().unwrap_stderr()
    );
}

#[test]
fn root_help() {
    let parser = setup();
    let expected_help = "\
The stupid content tracker

Usage: app COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    fetch       fetches branches from remote repository
    add         add files to the staging area
";

    assert_eq!(
        expected_help,
        parser.run_inner("--help").unwrap_err().unwrap_stdout()
    );
}

#[test]
fn fetch_help() {
    let parser = setup();
    let expected_help = "\
fetches branches from remote repository

Usage: app fetch [--dry_run] [--all] [SRC]

Available positional items:
    SRC

Available options:
        --dry_run
        --all
    -h, --help     Prints help information
";
    assert_eq!(
        expected_help,
        parser
            .run_inner("fetch --help")
            .unwrap_err()
            .unwrap_stdout()
    );
}

#[test]
fn add_help() {
    let parser = setup();
    let expected_help = "\
add files to the staging area

Usage: app add [-i] [--all] [FILE]...

Available positional items:
    FILE

Available options:
    -i
        --all
    -h, --help  Prints help information
";
    assert_eq!(
        expected_help,
        parser.run_inner("add --help").unwrap_err().unwrap_stdout()
    );
}
