//
use bpaf::*;
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// not that important switch
    #[bpaf(hide_usage)]
    switch: bool,
}
