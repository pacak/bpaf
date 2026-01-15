//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
pub struct Options {
    /// Number of jobs
    #[bpaf(argument("JOBS"), fallback(42), display_fallback)]
    jobs: usize,
}
