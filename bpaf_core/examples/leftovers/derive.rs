use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    /// This is a flag
    #[bpaf(short, long)]
    flag: bool,
    /// This is a positional item
    #[bpaf(positional("ITEM"), optional)]
    pos: Option<String>,
    #[bpaf(external(leftovers))]
    rest: Vec<String>,
}

fn main() {
    println!("{:?}", options().run());
}
