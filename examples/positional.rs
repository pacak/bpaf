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
        .argument("VAL")
        .from_str::<u32>()
        .fallback(42);
    let files = positional_os("FILE").map(PathBuf::from).many();
    let opts = construct!(Options { value, files }).to_options().run();

    println!("{:#?}", opts);
}
