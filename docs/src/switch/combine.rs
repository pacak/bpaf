//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    decision: bool,
}

fn parse_decision() -> impl Parser<bool> {
    long("decision").help("Positive decision").switch()
}

pub fn options() -> OptionParser<Options> {
    let decision = parse_decision();
    construct!(Options { decision }).to_options()
}
