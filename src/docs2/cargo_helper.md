<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    argument: usize,
    switch: bool,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("An argument")
        .argument::<usize>("ARG");
    let switch = short('s').help("A switch").switch();
    let options = construct!(Options { argument, switch });

    // Given the cargo command is `cargo pretty`.
    cargo_helper("pretty", options).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options("pretty"))] // Given the cargo command is `cargo pretty`.
pub struct Options {
    /// An argument
    argument: usize,
    /// A switch
    #[bpaf(short)]
    switch: bool,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Let's say the goal is to parse an argument and a switch:


<div class='bpaf-doc'>
$ app --argument 15<br>
Options { argument: 15, switch: false }
</div>


But when used as a `cargo` subcommand, cargo will also pass the command name. For example
you can invoke an app with binary name `cargo-asm`

```console
$ cargo asm --lib --everything
...
```

`cargo` will then spawn the executable and pass it following parameters:

```console
$ cargo-asm asm --lib --everything
...
```

If you are not using `cargo_helper` - parser won't know what to do with `asm` part.
`cargo-helper` allows the parser to strip it from the front and everything works as expected.

And it doesn't show up in `--help` so not to confuse users


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt> [<tt><b>-s</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>An argument</dd>
<dt><tt><b>-s</b></tt></dt>
<dd>A switch</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>

</details>