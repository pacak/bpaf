<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    commands: Vec<Cmd>,
}

# #[allow(dead_code)]
#[derive(Debug, Clone)]
enum Cmd {
    Eat(String),
    Drink(bool),
    Sleep(usize),
}

fn cmd() -> impl Parser<Cmd> {
    let eat = positional::<String>("FOOD")
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
        .argument::<usize>("HOURS")
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
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


You can chain one or more commands, commands can be arbitrarily nested too
```console
% app eat fastfood drink --coffee sleep --time=5
Options { switch: false, commands: [Eat("fastfood"), Drink(true), Sleep(5)] }
```

You can pass other flags after all the commands but not in between them
since commands are treated as positionals. It should be possible to consume
items before and between commands as well if they are consumed before the commands
like this: `construct!(Options { switch, commands })` but in that case you need
to be careful about not consuming anything from the command themselves.
```console
% app sleep --time 10 eat "Bak Kut Teh" drink -s
Options { switch: true, commands: [Sleep(10), Eat("Bak Kut Teh"), Drink(false)] }
```

</details>
