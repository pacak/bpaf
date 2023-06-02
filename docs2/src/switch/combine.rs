//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    release: bool,
    default_features: bool,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce verbose output")
        .switch();
    let release = long("release")
        .help("Build artifacts in release mode")
        .flag(true, false);
    let default_features = long("no-default-features")
        .help("Do not activate default features")
        // default_features uses opposite values,
        // producing `true` when value is absent
        .flag(false, true);

    construct!(Options {
        verbose,
        release,
        default_features,
    })
    .to_options()
}
