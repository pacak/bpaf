//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("VERS"))]
    version: Option<usize>,
    #[bpaf(argument("FEAT"))]
    feature: Option<String>,
}
