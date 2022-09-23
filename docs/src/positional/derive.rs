//
use std::{path::PathBuf, str::FromStr};
//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// Coin toss results
    #[bpaf(argument::<FromUtf8<Coin>>("COIN"), fallback(Coin::Heads))]
    coin: Coin,

    /// File to use
    #[bpaf(positional::<PathBuf>("FILE"))]
    file: PathBuf,
    /// Name to look for
    #[bpaf(positional("NAME"))]
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
