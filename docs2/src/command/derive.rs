//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
// `command` annotation with no name gets the name from the object it is attached to,
// but you can override it using something like #[bpaf(command("my_command"))]
// you can chain more short and long names here to serve as aliases
#[bpaf(command("cmd"), short('c'))]
/// Command to do something
pub struct Cmd {
    /// This flag is specific to command
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// This flag is specific to the outer layer
    flag: bool,
    #[bpaf(external)]
    cmd: Cmd,
}
