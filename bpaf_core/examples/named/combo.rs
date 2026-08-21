use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    alpha: u32,
    beta: u32,
    gamma: u32,
}

fn options() -> OptionParser<Options> {
    let alpha = short('a')
        .long("alpha")
        .argument("A")
        .help("Help for argument alpha");
    let beta = long("beta")
        .short('b')
        .argument("B")
        .help("Help for argument beta");
    let gamma = env("MAGIC")
        .env("MORE_MAGIC")
        .long("magic")
        .argument("C")
        .help("Help for flag gamma");
    construct!(Options { alpha, beta, gamma }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
