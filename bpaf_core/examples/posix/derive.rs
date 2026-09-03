use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    /// Emit diagnostic messages
    #[bpaf(short)]
    verbose: bool,
    /// File to process
    #[bpaf(positional("FILE"), posix, many)]
    file: Vec<String>,
}

fn main() {
    println!("{:?}", options().run());
}
