//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), switch)]
    switches: Vec<bool>,
}
