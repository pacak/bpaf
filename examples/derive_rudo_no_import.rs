#[derive(Debug, Clone, bpaf::Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    /// Use global file
    global: bool,
    #[bpaf(external, fallback(Action::List))]
    action: Action,
}

#[derive(Debug, Clone, bpaf::Bpaf)]
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
    println!("{:?}", bpaf::OptionParser::run(options()));
}
