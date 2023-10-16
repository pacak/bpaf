```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Opt in for premium serivces
    pub premium: bool,
    #[bpaf(external(cmd), many)]
    pub commands: Vec<Cmd>,
}

#[derive(Debug, Clone, Bpaf)]
// shape of the variants doesn't really matter, let's use all of them :)
pub enum Cmd {
    #[bpaf(command, adjacent)]
    /// Performs eating action
    Eat(#[bpaf(positional("FOOD"))] String),

    #[bpaf(command, adjacent)]
    /// Performs drinking action
    Drink {
        /// Are you going to drink coffee?
        coffee: bool,
    },

    #[bpaf(command, adjacent)]
    /// Performs taking a nap action
    Sleep {
        #[bpaf(argument("HOURS"))]
        time: usize,
    },
}
```

```rust,id:2
# use bpaf::*;
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
```

Example implements a parser that supports one of three possible commands:

```run,id:1,id:2
--help
```

As usual every command comes with its own help

```run,id:1,id:2
drink --help
```

Normally you can use one command at a time, but making commands `adjacent` lets
parser to succeed after consuming an adjacent block only and leaving leftovers for the rest of
the parser, consuming them as a `Vec<Cmd>` with [`many`](Parser::many) allows to chain multiple
items sequentially

```run,id:1,id:2
eat Fastfood drink --coffee sleep --time=5
```

The way this works is by running parsers for each command. In the first iteration `eat` succeeds,
it consumes `eat fastfood` portion and appends its value to the resulting vector. Then second
iteration runs on leftovers, in this case it will be `drink --coffee sleep --time=5`.
Here `drink` succeeds and consumes `drink --coffee` portion, then `sleep` parser runs, etc.

You can mix chained commands with regular arguments that belong to the top level parser

```run,id:1,id:2
sleep --time 10 --premium eat 'Bak Kut Teh' drink
```

But not inside the command itself since values consumed by the command are not going to be
adjacent

```run,id:1,id:2
sleep --time 10 eat --premium 'Bak Kut Teh' drink
```
