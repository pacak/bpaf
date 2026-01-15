//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}

fn parse_decision() -> impl Parser<Decision> {
    long("decision")
        .help("Positive decision")
        .flag(Decision::Yes, Decision::No)
}

pub fn options() -> OptionParser<Options> {
    let decision = parse_decision();
    construct!(Options { decision }).to_options()
}
