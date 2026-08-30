use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    flag: bool,
    pos: Option<String>,
    rest: Vec<String>,
}

fn options() -> OptionParser<Options> {
    let flag = short('f').long("flag").switch().help("This is a flag");
    let pos = positional("ITEM")
        .help("This is a positional item")
        .optional();
    let rest = leftovers();
    construct!(Options { flag, rest, pos }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
