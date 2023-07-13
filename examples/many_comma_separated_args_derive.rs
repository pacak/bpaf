/// To parse comma separated values it's easier to treat them as strings
use bpaf::*;
use std::{num::ParseIntError, str::FromStr};

fn split_and_parse(s: String) -> Result<Vec<u16>, ParseIntError> {
    s.split(',')
        .map(u16::from_str)
        .collect::<Result<Vec<_>, _>>()
}

fn flatten_vec(vv: Vec<Vec<u16>>) -> Vec<u16> {
    vv.into_iter().flatten().collect()
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct Opts {
    #[bpaf(
        long,
        argument::<String>("PORTS"),
        parse(split_and_parse),
        many,
        map(flatten_vec)
    )]
    /// Comma separated list of ports
    ports: Vec<u16>,
}

fn main() {
    println!("{:?}", opts().to_options().run());
}
