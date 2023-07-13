//! With minimal changes you can parse directly into smart pointers.
//!
//! While Bpaf derive macro knows nothing about Box or Arc it lets you
//! to parse into them by overriding the parsing sequence.
//!
//! Here `name` first parsed into a regular `String` then converted
//! into `Box<str>` using `map`.
//!
//! Similarly a set of coins is parsed into a regular vector and later converted into Arc still
//! inside the parser

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
