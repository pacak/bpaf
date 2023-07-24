use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    switch: bool,

    /// A custom argument
    argument: usize,
}
