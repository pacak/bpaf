//
use bpaf::*;
//
use std::{num::ParseIntError, str::FromStr};
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number")
        .argument::<String>("N")
        // normally you'd use argument::<u32> to get a numeric
        // value and `map` to double it
        .parse::<_, _, ParseIntError>(|s| Ok(u32::from_str(&s)? * 2));
    construct!(Options { number }).to_options()
}
