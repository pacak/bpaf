use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(short, long)]
    /// Help for argument alpha
    alpha: u32,
    #[bpaf(long("beta"), short('a'))]
    /// Help for argument beta
    beta: u32,

    #[bpaf(env("MAGIC"), long("magic"), env("MORE_MAGIC"))]
    /// Help for flag gamma
    gamma: u32,
}

fn main() {
    println!("{:?}", options().run());
}
