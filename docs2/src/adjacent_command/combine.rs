//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    premium: bool,
    commands: Vec<Cmd>,
}

#[derive(Debug, Clone)]
// shape of the variants doesn't really matter, let's use all of them :)
enum Cmd {
    Eat(String),
    Drink { coffee: bool },
    Sleep { time: usize },
}

fn cmd() -> impl Parser<Cmd> {
    let eat = positional::<String>("FOOD")
        .to_options()
        .descr("Performs eating action")
        .command("eat")
        .adjacent()
        .map(Cmd::Eat);

    let coffee = long("coffee")
        .help("Are you going to drink coffee?")
        .switch();
    let drink = construct!(Cmd::Drink { coffee })
        .to_options()
        .descr("Performs drinking action")
        .command("drink")
        .adjacent();

    let time = long("time").argument::<usize>("HOURS");
    let sleep = construct!(Cmd::Sleep { time })
        .to_options()
        .descr("Performs taking a nap action")
        .command("sleep")
        .adjacent();

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
