//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    crate_name: String,
    feature_name: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Display detailed information")
        .switch();

    let crate_name = positional("CRATE").help("Crate name to use");

    let feature_name = positional("FEATURE")
        .help("Display information about this feature")
        .optional();

    construct!(Options {
        verbose,
        // You must place positional items and commands after
        // all other parsers
        crate_name,
        feature_name
    })
    .to_options()
}
