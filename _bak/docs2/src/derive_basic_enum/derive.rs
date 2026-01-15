//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    File {
        /// Read input from a file
        name: String,
    },

    Url {
        /// Read input from URL
        url: String,
        /// Authentication method to use for the URL
        auth_method: String,
    },
}
