//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external)]
    decision: Decision,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(fallback(Decision::Undecided))]
pub enum Decision {
    /// Positive decision
    On,
    /// Negative decision
    Off,
    #[bpaf(skip)]
    Undecided,
}
