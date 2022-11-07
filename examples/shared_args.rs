//! It is possible to have shared args returned alongside a subcommand

use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct Action {
    verbose: bool,
    number: u32,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct Build {
    verbose: bool,
}

#[derive(Debug, Clone)]
enum Command {
    Action(Action),
    Build(Build),
}

fn shared() -> impl Parser<Vec<String>> {
    positional("ARG").many()
}

fn parse_command() -> impl Parser<(Command, Vec<String>)> {
    let action = action().map(Command::Action);
    let action = construct!(action, shared()).to_options().command("action");
    let build = build().map(Command::Build);
    let build = construct!(build, shared()).to_options().command("build");
    construct!([action, build])
}

fn main() {
    let opts = parse_command().to_options().run();

    println!("{:?}", opts);
}
