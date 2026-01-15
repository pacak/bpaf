use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Specify user name
    name: String,

    /// Specify user age
    age: usize,
}
