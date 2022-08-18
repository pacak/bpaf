//! deriving for tri-state enabled/disabled/undecided switch, non derive version would use
//! combination of `req_flag` and `construct!([on, off, undecined])`

#![allow(dead_code)]
use bpaf::{Bpaf, OptionParser};

// By default bpaf tries to parse booleans as flags, do something smart
// about Strings and file names and handles Option/Vec.
// Everything else is handled as a textual named argument. In this case
// we want to use external (with respect to Opts) function `flags`
// derived for `Flags`. Overall it looks like this:

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum Opts {
    /// enabled
    On,
    /// disabled
    Off,
    /// undecined
    #[bpaf(long("undecided"), hide, default)]
    Undecided,
}

#[derive(Debug, Clone, Bpaf)]
enum Flags {
    This,
    That,
}

fn main() {
    println!("{:?}", opts().run())
}
