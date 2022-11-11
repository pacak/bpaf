//! deriving for tri-state enabled/disabled/undecided switch, non derive version would use
//! combination of `req_flag` and `construct!([on, off, undecined])`

#![allow(dead_code)]
use bpaf::Bpaf;

// By default bpaf tries to parse booleans as flags, do something smart
// about Strings and file names and handles Option/Vec.
// Everything else is handled as a textual named argument.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum Opts {
    /// enabled
    On,
    /// disabled
    Off,
    /// undecined
    #[bpaf(hide, default)]
    Undecided,
}

fn main() {
    println!("{:?}", opts().run())
}
