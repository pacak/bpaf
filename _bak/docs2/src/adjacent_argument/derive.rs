//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("SPEC"), adjacent)]
    /// Package to use
    package: String,
}
