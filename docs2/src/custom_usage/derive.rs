//
use bpaf::{doc::*, *};
const BINARY_USAGE: &[(&str, Style)] = &[
    ("--binary", Style::Literal),
    ("=", Style::Text),
    ("BINARY", Style::Metavar),
];

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Binary to run
    #[bpaf(short, long, argument("BIN"), custom_usage(BINARY_USAGE))]
    binary: Option<String>,

    /// Package to check
    #[bpaf(short, long, argument("PACKAGE"))]
    package: Option<String>,
}
