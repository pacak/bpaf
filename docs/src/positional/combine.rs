//
use std::{path::PathBuf, str::FromStr};
//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    coin: Coin,
    file: PathBuf,
    name: Option<String>,
}

/// A custom datatype that doesn't implement [`FromOsStr`] but implements [`FromStr`]
#[derive(Debug, Clone, Copy)]
enum Coin {
    Heads,
    Tails,
}
impl FromStr for Coin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heads" => Ok(Coin::Heads),
            "tails" => Ok(Coin::Tails),
            _ => Err(format!("Expected 'heads' or 'tails', got '{}'", s)),
        }
    }
}

pub fn options() -> OptionParser<Options> {
    let file = positional::<PathBuf>("FILE").help("File to use");
    // sometimes you can get away with not specifying type in positional's turbofish
    let coin = long("coin")
        .help("Coin toss results")
        .argument::<FromUtf8<Coin>>("COIN")
        .fallback(Coin::Heads);
    let name = positional::<String>("NAME")
        .help("Name to look for")
        .optional();
    construct!(Options { coin, file, name }).to_options()
}
