use bpaf::{help::Help, *};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    alpha: u32,
    beta: u32,
}

fn options() -> OptionParser<Options> {
    let h = short('h').long("help").req_flag(Help::Brief);
    let hh = long("full-help").req_flag(Help::Full);
    let custom_help_parser = h
        .or_else(hh)
        .help_literal("    \u{1B}[2m-h\u{1B}[0m, \u{1B}[2m--help\u{1B}[0m, \u{1B}[2m--full-help\u{1B}[0m\tPrints help information")
        .hide_usage();

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
        .help_parser(custom_help_parser)
}

fn main() {
    println!("{:?}", options().run());
}
