//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(command)]
/// Command to do something
pub struct Cmd {
    /// This flag is specific to command
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// This flag is specific to the outer layer
    flag: bool,
    #[bpaf(external)]
    cmd: Cmd,
}
