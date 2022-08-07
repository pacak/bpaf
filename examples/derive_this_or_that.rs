#![allow(dead_code)]
use bpaf::{Bpaf, OptionParser};

// By default bpaf tries to parse booleans as flags, do something smart
// about Strings and file names and handles Option/Vec.
// Everything else is handled as a textual named argument. In this case
// we want to use external (with respect to Opts) function `flags`
// derived for `Flags`. Overall it looks like this:

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Opts {
    #[bpaf(external(flags))]
    flag: Flags,
}

#[derive(Debug, Clone, Bpaf)]
enum Flags {
    This,
    That,
}

fn main() {
    println!("{:?}", opts().run())
}
