//
use bpaf::*;

#[derive(Debug, Clone)]
pub struct Options {
    desert: Option<&'static str>,
}

pub fn options() -> OptionParser<Options> {
    let desert = ["apple", "banana", "orange", "grape", "strawberry"]
        .iter()
        .map(|name| {
            long(name)
                .help("Pick one of the options")
                .req_flag(*name)
                .boxed()
        });
    let desert = choice(desert).optional();
    construct!(Options { desert }).to_options()
}
