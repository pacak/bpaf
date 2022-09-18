//
use std::path::PathBuf;
//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// File to use
    #[bpaf(positional::<PathBuf>("FILE"))]
    file: PathBuf,
    /// Name to look for
    #[bpaf(positional("NAME"))]
    name: Option<String>,
}
