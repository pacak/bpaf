//
use bpaf::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument::<u32>("ARG"), collect)]
    argument: BTreeSet<u32>,
    /// some switch
    #[bpaf(long("switch"), switch, collect)]
    switches: BTreeSet<bool>,
}
