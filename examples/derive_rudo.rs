/// parser inspired by https://github.com/hood/rudo/blob/e448942b752c56dd2be2e2bb5026ced45e215ed6/src/main.rs
///
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    /// help
    #[bpaf(external, fallback(Action::List))]
    action: Action,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
enum Action {
    /// Add a new TODO item
    #[bpaf(command)]
    Add(String),

    /// Mark nth item as done
    #[bpaf(command)]
    Mark(usize),

    /// Read nth item
    #[bpaf(command)]
    Read(usize),

    /// Lists everything
    // name argument for command is optional
    #[bpaf(command("list"))]
    List,
}

fn main() {
    println!("{:?}", options().run());
}
