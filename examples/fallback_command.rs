//! Parse some commands manually or collect anything else as
//! for manual parsing
use bpaf::*;
use std::ffi::OsString;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
enum Commands {
    #[bpaf(command)]
    Build {
        /// Optimization level
        opt: u32,
    },
    Fallback(#[bpaf(external(fallback), hide)] Fallback),
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct Fallback {
    #[bpaf(positional("COMMAND"))]
    name: String,

    #[bpaf(any("ARG", Some))]
    args: Vec<OsString>,
}

fn main() {
    let opts = commands().run();
    println!("{:?}", opts);
}
