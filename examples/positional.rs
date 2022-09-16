//! Non derive version for positional arguments
use bpaf::*;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Options {
    value: u32,
    files: Vec<PathBuf>,
}

fn main() {
    let value = long("value")
        .help("Mysterious value")
        .argument::<u32>("VAL")
        .fallback(42);
    let files = positional::<PathBuf>("FILE").many();
    let opts = construct!(Options { value, files }).to_options().run();

    println!("{:#?}", opts);
}
