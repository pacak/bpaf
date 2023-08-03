//
use bpaf::*;
fn try_to_get_version() -> Result<usize, &'static str> {
    Ok(42)
}

#[derive(Debug, Clone)]
pub struct Options {
    version: usize,
}

pub fn options() -> OptionParser<Options> {
    let version = long("version")
        .help("Specify protocol version")
        .argument("VERS")
        .fallback_with(try_to_get_version)
        .debug_fallback();
    construct!(Options { version }).to_options()
}
