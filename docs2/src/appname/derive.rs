//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Specify user name
    #[bpaf(short, long, argument::<String>("NAME"))]
    user: String,

    /// Specify user age
    #[bpaf(external)]
    appname: String,
}
