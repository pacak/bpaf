use bpaf::*;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    /// Mysterious value
    #[bpaf(argument("VAL"), fallback(42))]
    value: u32,
    #[bpaf(positional_os("FILE"))]
    files: Vec<PathBuf>,
}

fn main() {
    let opts = options().run();
    println!("{:#?}", opts);
}
