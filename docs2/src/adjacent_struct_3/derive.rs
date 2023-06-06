//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external(execs))]
    exec: Option<Vec<OsString>>,
    #[bpaf(long, short)]
    /// Regular top level switch
    switch: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Exec {
    /// Spawn a process for each file found
    exec: (),

    #[bpaf(
        any("COMMAND", not_semi),
        some("Command and arguments, {} will be replaced with a file name")
    )]
    /// Command and arguments, {} will be replaced with a file name
    body: Vec<OsString>,

    #[bpaf(external(is_semi))]
    end: (),
}

fn not_semi(s: OsString) -> Option<OsString> {
    (s != ";").then_some(s)
}

fn is_semi() -> impl Parser<()> {
    // TODO - support literal in bpaf_derive
    literal(";")
}

// a different alternative would be to put a singular Exec
fn execs() -> impl Parser<Option<Vec<OsString>>> {
    exec().map(|e| e.body).optional()
}
