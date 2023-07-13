//! deriving for tri-state enabled/disabled/undecided switch, combinatoric version would use
//! combination of `req_flag` and `construct!([on, off])`

#![allow(dead_code)]
use bpaf::Bpaf;

// By default bpaf tries to parse booleans as flags, do something smart
// about Strings and file names and handles Option/Vec.
// Everything else is handled as a textual named argument.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, fallback(Opts::Undecided))]
enum Opts {
    /// enabled
    On,
    /// disabled
    Off,
    /// undecined
    #[bpaf(skip)]
    Undecided,
}

fn main() {
    println!("{:?}", opts().run())
}
