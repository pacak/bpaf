use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(positional("ALPHA"))]
    /// Help for argument alpha
    alpha: u32,
    #[bpaf(positional("BETA"), strict)]
    /// Help for argument beta
    beta: String,
}

fn main() {
    println!("{:?}", options().run());
}