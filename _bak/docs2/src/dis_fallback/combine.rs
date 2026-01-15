//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    jobs: usize,
}

pub fn options() -> OptionParser<Options> {
    let jobs = long("jobs")
        .help("Number of jobs")
        .argument("JOBS")
        .fallback(42)
        .display_fallback();
    construct!(Options { jobs }).to_options()
}
