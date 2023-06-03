<details><summary>Combinatoric example</summary>

```no_run
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
```

</details>
<details><summary>Output</summary>

Example implements a parser that supports one of three possible commands:


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-p</b></tt>] [<tt><i>COMMAND ...</i></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-p</b></tt>, <tt><b>--premium</b></tt></dt>
<dd>Opt in for premium serivces</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p><p><div>
<b>Available commands:</b></div><dl><dt><tt><b>eat</b></tt></dt>
<dd>Performs eating action</dd>
<dt><tt><b>drink</b></tt></dt>
<dd>Performs drinking action</dd>
<dt><tt><b>sleep</b></tt></dt>
<dd>Performs taking a nap action</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


As usual every command comes with its own help


<div class='bpaf-doc'>
$ app drink --help<br>
<p>Performs drinking action</p><p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>drink</b></tt> [<tt><b>--coffee</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --coffee</b></tt></dt>
<dd>Are you going to drink coffee?</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


Normally you can use one command at a time, but making commands `adjacent` lets
parser to succeed after consuming an adjacent block only and leaving leftovers for the rest of
the parser, consuming them as a `Vec<Cmd>` with [`many`](Parser::many) allows to chain multiple
items sequentially


<div class='bpaf-doc'>
$ app eat Fastfood drink --coffee sleep --time=5<br>
Options { premium: false, commands: [Eat("Fastfood"), Drink(true), Sleep(5)] }
</div>


The way this works is by running parsers for each command. In the first iteration `eat` succeeds,
it consumes `eat fastfood` portion and appends its value to the resulting vector. Then second
iteration runs on leftovers, in this case it will be `drink --coffee sleep --time=5`.
Here `drink` succeeds and consumes `drink --coffee` portion, then `sleep` parser runs, etc.

You can mix chained commands with regular arguments that belong to the top level parser


<div class='bpaf-doc'>
$ app sleep --time 10 --premium eat 'Bak Kut Teh' drink<br>
Options { premium: true, commands: [Sleep(10)] }
</div>


But not inside the command itself


<div class='bpaf-doc'>
$ app sleep --time 10 eat --premium 'Bak Kut Teh' drink<br>
Expected <tt><i>FOOD</i></tt>, pass <tt><b>--help</b></tt> for usage information
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>

</details>