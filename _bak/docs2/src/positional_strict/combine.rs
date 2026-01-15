//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    binary: String,
    args: Vec<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce detailed report")
        .switch();
    let binary = long("bin").help("Binary to execute").argument("BIN");
    let args = positional("ARG")
        .help("Arguments for the binary")
        .strict()
        .many();
    construct!(Options {
        verbose,
        binary,
        args
    })
    .to_options()
}
