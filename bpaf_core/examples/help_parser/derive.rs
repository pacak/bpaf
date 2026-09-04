use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options, help_parser(help::short_long))]
struct Options {
    /// Help for argument alpha
    ///
    /// Alpha counts things.
    ///
    /// Very carefully.
    #[bpaf(short, long)]
    alpha: u32,

    /// Help for argument beta
    ///
    /// Beta counts other things.
    ///
    /// Very carelessly.
    #[bpaf(short, long)]
    beta: u32,
}

fn main() {
    println!("{:?}", options().run());
}
