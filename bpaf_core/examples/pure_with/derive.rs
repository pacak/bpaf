use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(argument("NAME"))]
    /// Use a custom user name
    name: String,
    #[bpaf(pure_with(starting_money))]
    money: u32,
}

fn starting_money() -> Result<u32, &'static str> {
    Ok(330)
}

fn main() {
    println!("{:?}", options().run());
}