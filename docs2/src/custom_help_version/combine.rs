//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    package: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let help = long("help").short('H').help("Renders help information");
    let version = long("version")
        .short('v')
        .help("Renders version information");
    let package = short('p')
        .help("Package to check")
        .argument("SPEC")
        .optional();

    construct!(Options { package })
        .to_options()
        .descr("Command with custom flags for help and version")
        .version("0.42")
        .help_parser(help)
        .version_parser(version)
}
