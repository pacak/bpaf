//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    meal: Vec<Meal>,
    premium: bool,
}

#[derive(Debug, Clone)]
struct Meal {
    m: (),
    spicy: Option<usize>,
    drink: bool,
    dish: usize,
}

/// You can mix all sorts of things inside the adjacent group
fn meal() -> impl Parser<Meal> {
    let m = short('o')
        .long("meal")
        .help("A meal [o]rder consists of a main dish with an optional drink")
        .req_flag(());
    let spicy = long("spicy")
        .help("On a scale from 1 to a lot, how spicy do you want your meal?")
        .argument::<usize>("SPICY")
        .optional();
    let drink = long("drink")
        .help("Do you want drink with your meal?")
        .switch();
    let dish = positional::<usize>("DISH").help("Main dish number");
    construct!(Meal {
        m,
        spicy,
        drink,
        dish
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let premium = short('p')
        .long("premium")
        .help("Do you want to opt in for premium service?")
        .switch();
    let meal = meal().many();
    construct!(Options { meal, premium }).to_options()
}
