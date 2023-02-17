//! A way to customize usage for a nested command
//!
//! Usually you would go with generated usage or by overriding it using `usage` attribute
//! to the top level or command level bpaf annotation. By taking advantage of command being just a
//! set of options with it's own help message and a custom prefix you can override the usage with
//! an arbitrary string, including one generated at runtime by doing something like this:

use bpaf::*;

// this defines top level set of options and refers to an external parser `cmd_usage`
// At this point cmd_usage can be any parser that produces Cmd
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
enum Opts {
    Cmd(#[bpaf(external(cmd_usage))] Cmd),
}

// bpaf defines command as something with its own help message that can be accessed with a
// positional command name - inside of the command there is an OptionParser, this struct
// defines the parser we are going to use later
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
struct Cmd {
    opt: bool,
}

// At this point we have OptionParser<Cmd> and we want to turn that into a regular parser
// with custom usage string - for that we are using two functions from combinatoric api:
// `usage` and `command`
fn cmd_usage() -> impl Parser<Cmd> {
    cmd().usage("A very custom usage goes here").command("cmd")
}

fn main() {
    println!("{:?}", opts().run());
}
