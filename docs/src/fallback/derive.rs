//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
//
#[allow(dead_code)]
pub struct Options {
    #[bpaf(argument("VERS"), fallback(42))]
    version: usize,
}
