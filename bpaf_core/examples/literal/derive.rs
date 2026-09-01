use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(external(lit_req))]
    item: u32,
    #[bpaf(external(lit_flag))]
    flag: bool,
}

fn lit_flag() -> impl Parser<Output = bool> {
    literal("flag").short('f').switch()
}

fn lit_req() -> impl Parser<Output = u32> {
    literal("item").req_flag(32)
}

fn main() {
    println!("{:?}", options().run());
}
