//
use std::path::PathBuf;
//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    file: PathBuf,
    name: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let file = positional::<PathBuf>("FILE").help("File to use");
    // sometimes you can get away with not specifying type in positional's turbofish
    let name = positional("NAME").help("Name to look for").optional();
    construct!(Options { file, name }).to_options()
}
