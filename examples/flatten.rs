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

fn main() {
    let verbose = short('v').help("switch verbosity on").switch();
    let user = short('u').help("daemon user").argument::<String>("USER");
    let group = short('g').help("daemon group").argument::<String>("GROUP");
    let daemon_opts = construct!(DaemonOpts { user, group });
    let opt = construct!(Cmdline {
        verbose,
        daemon_opts
    })
    .to_options()
    .run();
    println!("{:?}", opt);
}
