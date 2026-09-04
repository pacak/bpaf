use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    alpha: u32,
    beta: u32,
}

fn options() -> OptionParser<Options> {
    let alpha = short('a')
        .long("alpha")
        .argument("A")
        .help("Help for argument alpha\n\nAlpha counts things. Very carefully.");
    let beta = long("beta")
        .short('b')
        .argument("B")
        .help("Help for argument beta\n\nBeta counts other things.\n\nVery carelessly.");
    construct!(Options { alpha, beta })
        .to_options()
        .help_parser(help::short_long)
}

fn main() {
    println!("{:?}", options().run());
}
