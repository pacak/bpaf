//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    version: usize,
}
pub fn options() -> OptionParser<Options> {
    let version = long("version").argument("VERS").fallback(42);
    construct!(Options { version }).to_options()
}
