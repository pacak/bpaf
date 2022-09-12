//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
//
#[allow(dead_code)]
pub struct Options {
    #[bpaf(positional("VERS"), catch)]
    version: Option<usize>,
    #[bpaf(positional("FEAT"), catch)]
    feature: Option<String>,
}
