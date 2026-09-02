use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    alpha: u32,
    beta: String,
}

fn options() -> OptionParser<Options> {
    let alpha = positional("ALPHA").help("Help for argument alpha");
    let beta = positional("BETA").help("Help for argument beta");
    construct!(Options { alpha, beta }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
