//! Customizations for `--help` output and help/errors colorscheme
//!
//! `bpaf` uses doc comments on fields and strings passed to `help` methods (such
//! as [`Named::help`](crate::api::primitives::Named::help) in the help output by
//! following those rules:
//!
//! 1. Everything up to the first blank line is included into a "short" help message
//! 2. Everything is included into a "long" help message
//! 3. `bpaf` preserves linebreaks followed by a line that starts with a space:
//!    ```text
//!    this linebreak ->
//!    is removed
//!
//!    but this one ->
//!     is preserved
//!
//!    ^ note the whitespace!
//!    ```
//! 4. Linebreaks are removed otherwise

use crate::{BoxParser, Parser, long, macros::example_cd, short};
pub use crate::{
    console_writer::{Colorscheme, Style},
    help_cmd::help_command as command,
};

pub mod custom {
    pub use crate::custom_help::*;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Help {
    #[default]
    Brief,
    Full,
}

/// Pass `-h` for short and `--help` for long help version
///
#[doc = example_cd!("help_parser")]
pub fn short_long() -> BoxParser<Help> {
    let h = short('h').req_flag(Help::Brief);
    let hh = long("help").req_flag(Help::Full);
    h.or_else(hh)
        .help_literal("    \u{1B}[2m-h\u{1B}[0m, \u{1B}[2m--help\u{1B}[0m\tPrints help information")
        .hide_usage()
        .into_box()
}

/// Pass `-h` / `--help` once for short and twice - for long version
///
/// **This is what `bpaf` uses by default. It is provided primarily for completeness.**
///
#[doc = example_cd!("help_parser")]
pub fn once_twice() -> BoxParser<Help> {
    short('h')
        .long("help")
        .help("Prints help information")
        .req_flag(())
        .count()
        .parse(|c| match c {
            1 => Ok(Help::Brief),
            2 => Ok(Help::Full),
            _ => Err("not help"),
        })
        .hide_usage()
        .into_box()
}

/// Both `-h` and `--help` print long version
///
#[doc = example_cd!("help_parser")]
pub fn always_full() -> BoxParser<Help> {
    short('h')
        .long("help")
        .help("Prints help information")
        .req_flag(Help::Full)
        .hide_usage()
        .into_box()
}

/// Both `-h` and `--help` print short version
///
#[doc = example_cd!("help_parser")]
pub fn always_brief() -> BoxParser<Help> {
    short('h')
        .long("help")
        .help("Prints help information")
        .req_flag(Help::Brief)
        .hide_usage()
        .into_box()
}
