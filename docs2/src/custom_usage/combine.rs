//
use bpaf::{doc::*, *};
#[derive(Debug, Clone)]
pub struct Options {
    binary: Option<String>,
    package: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let binary = short('b')
        .long("binary")
        .help("Binary to run")
        .argument("BIN")
        .optional()
        .custom_usage(
            &[
                ("--binary", Style::Literal),
                ("=", Style::Text),
                ("BINARY", Style::Metavar),
            ][..],
        );

    let package = short('p')
        .long("package")
        .help("Package to check")
        .argument("PACKAGE")
        .optional();

    construct!(Options { binary, package }).to_options()
}
