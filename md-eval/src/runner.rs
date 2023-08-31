use std::path::PathBuf;

use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Format generated code with prettyplease
    pub pretty: bool,

    /// Include generators that spawn a shell for autocomplete
    pub slow: bool,

    /// A directory to write generated documentation
    #[bpaf(short, long, fallback("./src/_docs".into()), debug_fallback)]
    pub out_dir: PathBuf,
}
