//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    commands: Vec<Cmd>,
}

//
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Cmd {
    Eat(String),
    Drink(bool),
    Sleep(usize),
}

fn cmd() -> impl Parser<Cmd> {
    let eat = positional("FOOD")
        .to_options()
        .command("eat")
        .adjacent()
        .map(Cmd::Eat);

    let drink = long("coffee")
        .switch()
        .to_options()
        .command("drink")
        .adjacent()
        .map(Cmd::Drink);

    let sleep = long("time")
        .argument("HOURS")
        .from_str::<usize>()
        .to_options()
        .command("sleep")
        .adjacent()
        .map(Cmd::Sleep);

    construct!([eat, drink, sleep])
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let commands = cmd().many();
    construct!(Options { commands, switch }).to_options()
}
