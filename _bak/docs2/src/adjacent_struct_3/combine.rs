//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    exec: Option<Vec<OsString>>,
    switch: bool,
}

fn exec() -> impl Parser<Option<Vec<OsString>>> {
    // this defines starting token - "--exec"
    let start = long("exec")
        .help("Spawn a process for each file found")
        .req_flag(());
    // this consumes everything that is not ";"
    let body = any("COMMAND", |s| (s != ";").then_some(s))
        .help("Command and arguments, {} will be replaced with a file name")
        .some("You need to pass some arguments to exec");
    // this defines endint goken - ";"
    let end = literal(";");
    // this consumes everything between starting token and ending token
    construct!(start, body, end)
        // this makes it so everything between those tokens is consumed
        .adjacent()
        // drop the surrounding tokens leaving just the arguments
        .map(|x| x.1)
        // and make it optional so that instead of an empty Vec
        // it is `None` when no `--exec` flags was passed.
        .optional()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s')
        .long("switch")
        .help("Regular top level switch")
        .switch();
    construct!(Options { exec(), switch }).to_options()
}
