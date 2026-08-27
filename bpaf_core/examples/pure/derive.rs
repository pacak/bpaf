use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(argument("NAME"))]
    /// User name
    name: String,
    #[bpaf(pure(330))]
    money: u32,
}

fn main() {
    println!("{:?}", options().run());
}
