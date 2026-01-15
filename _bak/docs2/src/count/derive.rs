//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Increase the verbosity level
    #[bpaf(short('v'), long("verbose"), req_flag(()), count)]
    verbosity: usize,
}
