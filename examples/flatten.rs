//! How to nest things

use bpaf::*;

#[derive(Debug, Clone)]
struct Cmdline {
    /// switch verbosity on
    verbose: bool,
    daemon_opts: DaemonOpts,
}

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
    let cmdline = construct!(Cmdline {
        verbose,
        daemon_opts
    });
    let opt = Info::default().for_parser(cmdline).run();
    println!("{:?}", opt);
}
