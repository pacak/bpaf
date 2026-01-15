//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, usage("Usage: my_program [--release] [--binary=BIN] ..."))]
pub struct Options {
    #[bpaf(short, long)]
    /// Perform actions in release mode
    release: bool,
    #[bpaf(short, long, argument("BIN"))]
    /// Use this binary
    binary: String,
}
