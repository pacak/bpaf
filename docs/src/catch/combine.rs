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
    let version = positional::<usize>("VERS").optional().catch();
    let feature = positional::<String>("FEAT").optional();
    construct!(Options { version, feature }).to_options()
}
