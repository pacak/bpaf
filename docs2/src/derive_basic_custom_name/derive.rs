//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, long)]
    switch: bool,

    /// A custom argument
    #[bpaf(long("my-argument"), short('A'))]
    argument: usize,
}
