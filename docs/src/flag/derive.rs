//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(flag(Decision::Yes, Decision::No))]
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}
