//! All the flags don't have to live in the same structure, this example uses non derive version.
//! with derive API you would use `external` annotation

use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Cmdline {
    /// switch verbosity on
    verbose: bool,
    daemon_opts: DaemonOpts,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DaemonOpts {
    /// daemon user
    user: String,

    /// daemon group
    group: String,
}

// 1. need to know where to get the binary from
// 2. need to know how to get shell completions for different shells
// 3. need a place to store the cache

fn main() {
    let verbose = short('v').help("switch verbosity on").switch();
    let user = short('u')
        .long("user")
        .help("daemon user")
        .argument::<String>("USER")
        .complete(|f: &str| {
            let f = f.strip_prefix('"').unwrap_or(f);
            ["alice", "bob"]
                .iter()
                .filter(|&u| u.starts_with(f))
                .map(|u| (u.to_string(), None))
                .collect::<Vec<_>>()
        });
    let group = short('g')
        .long("group")
        .help("daemon group")
        .argument::<String>("GROUP");
    let daemon_opts = construct!(DaemonOpts { user, group });
    let opt = construct!(Cmdline {
        verbose,
        daemon_opts
    })
    .to_options()
    .run();
    println!("{:?}", opt);
}
