//! How to nest things

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
    let user = short('u').help("daemon user").argument("USER");
    let group = short('g').help("daemon group").argument("GROUP");
    let daemon_opts = construct!(DaemonOpts { user, group });
    let opt = construct!(Cmdline {
        verbose,
        daemon_opts
    })
    .to_options()
    .run();
    println!("{:?}", opt);
}
