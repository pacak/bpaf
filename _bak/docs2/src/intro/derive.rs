use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Message to print in a big friendly letters
    #[bpaf(positional("MESSAGE"))]
    message: String,
}
