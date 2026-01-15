//
use bpaf::{doc::*, *};
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number")
        .help(
            &[
                ("Very", Style::Emphasis),
                (" important argument", Style::Text),
            ][..],
        )
        .argument::<u32>("N");
    construct!(Options { number }).to_options()
}
