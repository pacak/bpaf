//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    version: Option<usize>,
    feature: Option<String>,
}
pub fn options() -> OptionParser<Options> {
    let version = long("version").argument("VERS").optional();
    let feature = long("feature").argument("FEAT").optional();
    construct!(Options { version, feature }).to_options()
}
