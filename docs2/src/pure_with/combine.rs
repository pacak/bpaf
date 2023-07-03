//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    money: u32,
}

fn starting_money() -> Result<u32, &'static str> {
    Ok(330)
}

pub fn options() -> OptionParser<Options> {
    // User can customise a name
    let name = long("name").help("Use a custom user name").argument("NAME");
    // but not starting amount of money
    let money = pure_with(starting_money);
    construct!(Options { name, money }).to_options()
}
