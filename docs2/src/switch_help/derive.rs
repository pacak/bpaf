//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Output detailed help information, you can specify it multiple times
    ///
    ///  when used once it outputs basic diagnostic info,
    ///  when used twice or three times - it includes extra debugging.
    verbose: bool,

    #[bpaf(argument("NAME"))]
    /// Use this as a task name
    name: String,

    #[bpaf(positional("OUTPUT"))]
    /// Save output to a file
    output: Option<String>,
}
