//! pretty basic derive example with external function

use bpaf::{short, Bpaf, Parser};
use std::path::PathBuf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
#[allow(dead_code)]
struct Opts {
    /// Activate debug mode
    #[bpaf(short, long)]
    debug: bool,
    /// this comment is ignored
    #[bpaf(external(verbose))]
    verbose: usize,
    /// Set speed
    #[bpaf(argument("SPEED"), fallback(42.0))]
    speed: f64,
    /// Output file
    output: PathBuf,

    #[bpaf(guard(positive, "must be positive"), fallback(1))]
    nb_cars: u32,
    files_to_process: Vec<PathBuf>,
}

fn verbose() -> impl Parser<usize> {
    // number of occurrences of the v/verbose flag capped at 3
    short('v')
        .long("verbose")
        .help("Increase the verbosity\nYou can specify it up to 3 times\neither as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this")
}

fn positive(input: &u32) -> bool {
    *input > 0
}

fn main() {
    println!("{:#?}", opts().run());
}
