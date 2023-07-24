//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command("run"))]
    /// Run a binary
    Run {
        /// Name of a binary crate
        name: String,
    },

    /// Run a self test
    #[bpaf(command)]
    Test,
}
