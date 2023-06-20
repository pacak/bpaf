//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbosity: usize,
}

pub fn options() -> OptionParser<Options> {
    let verbosity = short('v')
        .long("verbose")
        .help("Increase the verbosity level")
        .req_flag(())
        .count();

    construct!(Options { verbosity }).to_options()
}
