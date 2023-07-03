//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Display detailed information
    #[bpaf(short, long)]
    verbose: bool,

    // You must place positional items and commands after
    // all other parsers
    #[bpaf(positional("CRATE"))]
    /// Crate name to use
    crate_name: String,

    #[bpaf(positional("FEATURE"))]
    /// Display information about this feature
    feature_name: Option<String>,
}
