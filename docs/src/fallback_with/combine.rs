//
use bpaf::*;
fn try_to_get_version() -> Result<usize, &'static str> {
    Ok(42)
}

#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    version: usize,
}

pub fn options() -> OptionParser<Options> {
    let version = long("version")
        .argument("VERS")
        .fallback_with(try_to_get_version);
    construct!(Options { version }).to_options()
}
