pub use crate::help_cmd::help_command as command;
use crate::{BoxParser, Parser, long, short};

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
pub fn short_long() -> BoxParser<Help> {
    let h = short('h').req_flag(Help::Brief);
    let hh = long("help").req_flag(Help::Full);
    h.or_else(hh)
        .help_literal("    \u{1B}[2m-h\u{1B}[0m, \u{1B}[2m--help\u{1B}[0m\tPrints help information")
        .hide_usage()
        .into_box()
}

/// Pass `-h` / `--help` once for short and twice - for long version
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
