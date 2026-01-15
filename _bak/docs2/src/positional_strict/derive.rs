//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Produce detailed report
    verbose: bool,
    #[bpaf(long("bin"), argument("BIN"))]
    /// Binary to execute
    binary: String,
    #[bpaf(positional("ARG"), strict, many)]
    /// Arguments for the binary
    args: Vec<String>,
}
