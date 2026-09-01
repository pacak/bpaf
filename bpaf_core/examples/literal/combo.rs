use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    item: u32,
    flag: bool,
}

fn options() -> OptionParser<Options> {
    let flag = literal("flag")
        .short('f')
        .switch()
        .help("literal optional boolean flag");
    let item = literal("item").req_flag(32).help("literal required flag");
    construct!(Options { item, flag }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
