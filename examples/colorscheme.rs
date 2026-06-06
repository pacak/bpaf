//! By default, bpaf uses `Colorscheme::BRIGHT` for colored help output.
//! You can switch it to `Colorscheme::DULL` for a more muted look or even define your own style

use bpaf::*;

#[derive(Debug, Clone)]
struct Opts {
    verbose: bool,
    count: u32,
}

fn opts() -> OptionParser<Opts> {
    let verbose = short('v')
        .long("verbose")
        .help("Enable verbose mode")
        .switch();
    let count = short('n')
        .long("count")
        .help("Number of items")
        .argument::<u32>("N")
        .fallback(1)
        .display_fallback();

    construct!(Opts { verbose, count })
        .to_options()
        .descr("Colorscheme override example")
        .colorscheme(&Colorscheme::DULL)
}

fn main() {
    let parsed = opts().run();
    println!("{:#?}", parsed);
}
