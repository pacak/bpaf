//! When implementing a cargo subcommand parser needs to be able to skip the first argument which
//! is always the same as the executable name. For this example executable name is `cargo-cmd` so
//! first argument would be `cmd`. A way to support both cases - when it's present and it's absent
//! would be to use a combination of `literal`, `optional` and `hide`.
//! `bpaf` also provides a helper `cargo_helper` that does exactly this.

use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Opts {
    width: usize,
    height: usize,
}

fn main() {
    // defining a parser in a usual way
    let width = short('w').argument::<usize>("WIDTH").fallback(10);
    let height = short('h').argument::<usize>("HEIGHT").fallback(10);
    let parser = construct!(Opts { width, height });

    let cmd = literal("cmd").optional().hide();
    let combined_parser = construct!(cmd, parser).map(|x| x.1);

    let opts = combined_parser.to_options().run();

    println!("{:?}", opts);
}
