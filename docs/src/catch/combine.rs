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
    let version = positional("VERS").from_str::<usize>().optional().catch();
    let feature = positional("FEAT").optional();
    construct!(Options { version, feature }).to_options()
}
