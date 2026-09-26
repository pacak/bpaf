use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    name: String,
    money: u32,
}

fn starting_money() -> Result<u32, &'static str> {
    Ok(330)
}

fn options() -> OptionParser<Options> {
    let name = long("name").help("Use a custom user name").argument("NAME");
    let money = pure_with(starting_money);
    construct!(Options { name, money }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
