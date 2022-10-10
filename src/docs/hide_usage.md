<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
    switch: bool,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .fallback(30);
    let switch = long("switch")
        .help("not that important switch")
        .switch()
        .hide_usage();
    construct!(Options { argument, switch }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// not that important switch
    #[bpaf(hide_usage)]
    switch: bool,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


`hide_usage` doesn't change the parsing behavior in any way
```console
% app --argument 32
Options { argument: 32, switch: false }
```

It hides the inner parser from usage line, but not from the rest of the help or completion
```console
% app --help
Usage: [--argument ARG]

Available options:
        --argument <ARG>  important argument
        --switch          not that important switch
    -h, --help            Prints help information
```

</details>
