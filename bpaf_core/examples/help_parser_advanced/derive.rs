use bpaf::*;

fn custom_help_parser() -> impl Parser<Output = help::Help> {
    let h = short('h').long("help").req_flag(help::Help::Brief);
    let hh = long("full-help").req_flag(help::Help::Full);
    h.or_else(hh)
        .help_literal("    \u{1B}[2m-h\u{1B}[0m, \u{1B}[2m--help\u{1B}[0m, \u{1B}[2m--full-help\u{1B}[0m\tPrints help information")
        .hide_usage()
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options, help_parser(custom_help_parser()))]
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
