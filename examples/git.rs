//! `git.rs` serves as a demonstration of how to use subcommands,
//! as well as a demonstration of adding documentation to subcommands.
//!
//! Note, this "fetch" command uses fallback to inner description to get the help message, "add"
//! uses explicit override with the same value.

use std::path::PathBuf;

use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
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

fn main() {
    let dry_run = long("dry_run").switch();
    let all = long("all").switch();
    let repository = positional::<String>("SRC").fallback("origin".to_string());
    let fetch = construct!(Opt::Fetch {
        dry_run,
        all,
        repository
    })
    .to_options()
    .descr("fetches branches from remote repository");

    let fetch_cmd = fetch.command("fetch");

    let interactive = short('i').switch();
    let all = long("all").switch();
    let files = positional::<PathBuf>("FILE").many();
    let add = construct!(Opt::Add {
        interactive,
        all,
        files
    })
    .to_options()
    .descr("add files to the staging area");

    let add_cmd = add.command("add").help("add files to the staging area");

    let opt = construct!([fetch_cmd, add_cmd])
        .to_options()
        .descr("The stupid content tracker")
        .run();

    println!("{:?}", opt);
}
