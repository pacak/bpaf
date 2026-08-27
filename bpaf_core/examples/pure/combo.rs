use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    name: String,
    money: u32,
}

fn options() -> OptionParser<Options> {
    let name = long("name").help("User name").argument("NAME");
    let money = pure(330);
    construct!(Options { name, money }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
