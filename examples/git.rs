//! `git.rs` serves as a demonstration of how to use subcommands,
//! as well as a demonstration of adding documentation to subcommands.

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
        files: Vec<String>,
    },
}

fn main() {
    let dry_run = long("dry_run").switch();
    let all = long("all").switch();
    let repository = positional("SRC").fallback("origin".to_string());
    let fetch = construct!(Opt::Fetch {
        dry_run,
        all,
        repository
    });
    let fetch_info = Info::default().descr("fetches branches from remote repository");
    let fetch_cmd = command("fetch", fetch_info.for_parser(fetch))
        .help("fetch branches from remote repository");

    let interactive = short('i').switch();
    let all = long("all").switch();
    let files = positional("FILE").many();
    let add = construct!(Opt::Add {
        interactive,
        all,
        files
    });
    let add_info = Info::default().descr("add files to the staging area");
    let add_cmd = command("add", add_info.for_parser(add)).help("add files to the staging area");

    let opt = Info::default()
        .descr("The stupid content tracker")
        .for_parser(construct!([fetch_cmd, add_cmd]))
        .run();

    println!("{:?}", opt);
}
