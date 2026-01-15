//
use bpaf::*;
//
use std::{num::ParseIntError, str::FromStr};
fn twice_the_num(s: String) -> Result<u32, ParseIntError> {
    Ok(u32::from_str(&s)? * 2)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<String>("N"), parse(twice_the_num))]
    number: u32,
}
