use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    /// Use global file
    global: bool,
    #[bpaf(external, fallback(Action::List))]
    action: Action,
}

#[derive(Debug, Clone, Bpaf)]
enum Action {
    /// Add a new TODO item
    #[bpaf(command)]
    Add(String),

    /// Mark nth item as done
    ///
    ///
    /// ddd
    #[bpaf(command)]
    Mark(usize),

    /// Read nth item
    #[bpaf(command)]
    Read(usize),

    /// Lists everything
    // name argument for command is optional
    #[bpaf(command("list"))]
    List,

    /// Similar to List but converted as a flag instead of a command
    AlsoList,
}

fn main() {
    println!("{:?}", options().run());
}
