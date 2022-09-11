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
    On,
    Off,
    Undecided,
}

// user can specify either --on or --off, parser would fallback to `Undecided`
fn parse_decision() -> impl Parser<Decision> {
    let on = long("on").help("Positive decision").req_flag(Decision::On);
    let off = long("off")
        .help("Negative decision")
        .req_flag(Decision::Off);
    construct!([on, off]).fallback(Decision::Undecided)
}

pub fn options() -> OptionParser<Options> {
    let decision = parse_decision();
    construct!(Options { decision }).to_options()
}
