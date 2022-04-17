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
    let parser = construct!(Options { value, files });

    let opts = Info::default().for_parser(parser).run();

    println!("{opts:#?}");
}
