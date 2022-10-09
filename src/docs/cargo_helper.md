<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
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

    cargo_helper("pretty", options).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options("pretty"))]
pub struct Options {
    /// An argument
    argument: usize,
    /// A switch
    #[bpaf(short)]
    switch: bool,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Let's say the goal is to parse an argument and a switch:
```console
% app --argument 15
Options { argument: 15, switch: false }
```

But when used as a `cargo` subcommand, cargo will also pass the command name, this example
uses _wrong_ subcommand name to bypass the helper and show how it would look without it
```console
% app wrong --argument 15
No such command: `wrong`, did you mean `-s`?
```

When used with the right command - helper simply consumes it
```console
% app pretty --argument 42 -s
Options { argument: 42, switch: true }
```

And it doesn't show up in `--help` so not to confuse users
```console
% app --help
Usage: --argument ARG [-s]

Available options:
        --argument <ARG>  An argument
    -s                    A switch
    -h, --help            Prints help information
```

</details>
