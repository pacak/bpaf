#![allow(dead_code)]

use bpaf::*;
use std::sync::Arc;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<String>("NAME"), map(Box::from))]
    /// Adventurer's name
    name: Box<str>,

    #[bpaf(positional::<usize>("COIN"), many, map(Arc::from))]
    /// A set of coins
    coins: Arc<[usize]>,
}

fn main() {
    let xs = options().fallback_to_usage().run();
    println!("{xs:?}");
}
