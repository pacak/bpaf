//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    premium: bool,
    commands: Vec<Cmd>,
}

#[derive(Debug, Clone)]
enum Cmd {
    Eat(String),
    Drink(bool),
    Sleep(usize),
}

fn cmd() -> impl Parser<Cmd> {
    let eat = positional::<String>("FOOD")
        .to_options()
        .descr("Performs eating action")
        .command("eat")
        .adjacent()
        .map(Cmd::Eat);

    let drink = long("coffee")
        .help("Are you going to drink coffee?")
        .switch()
        .to_options()
        .descr("Performs drinking action")
        .command("drink")
        .adjacent()
        .map(Cmd::Drink);

    let sleep = long("time")
        .argument::<usize>("HOURS")
        .to_options()
        .descr("Performs taking a nap action")
        .command("sleep")
        .adjacent()
        .map(Cmd::Sleep);

    construct!([eat, drink, sleep])
}

pub fn options() -> OptionParser<Options> {
    let premium = short('p')
        .long("premium")
        .help("Opt in for premium serivces")
        .switch();
    let commands = cmd().many();
    construct!(Options { premium, commands }).to_options()
}
