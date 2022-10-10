//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument("ARG"), some("want at least one argument"))]
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), switch, some("want at least one switch"))]
    switches: Vec<bool>,
}
