//! Multi level fallback example:
//!
//! Fallback to one of several values
//! - the command line argument
//! - the environmental variable
//! - the config file
//! - the hard-coded default

#![allow(dead_code)]
use bpaf::*;

#[derive(Clone, Debug)]
struct Config {
    field1: u32,
    field2: u64,
}

/// Here this is a constant but it can be OnceCell from `once_cell`
/// that reads the config and deals with substituting missing falues with defaults
const DEFAULT_CONFIG: Config = Config {
    field1: 42,
    field2: 10,
};

pub fn main() {
    let field1 = long("field1")
        .env("FIELD1")
        .help("Field 1")
        .argument::<u32>("ARG")
        .fallback(DEFAULT_CONFIG.field1);
    let field2 = long("field2")
        .env("FIELD2")
        .help("Field 2")
        .argument::<u64>("ARG")
        .fallback(DEFAULT_CONFIG.field2);

    let opts = construct!(Config { field1, field2 }).to_options().run();

    // At this point if you get opts - it should be taken from one of
    // - the command line argument
    // - the environmental variable
    // - the config file
    // - the hard-coded default (from config parser)
    println!("{:?}", opts);
}
