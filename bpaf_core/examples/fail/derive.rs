use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    /// Field to specify
    #[bpaf(short, long, argument("F"), or_else(missing))]
    field: u32,
}

fn missing() -> impl Parser<Output = u32> {
    fail("You need to specify the field value")
}

fn main() {
    println!("{:?}", options().run());
}
