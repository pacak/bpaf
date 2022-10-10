<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
use bpaf::*;
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
    let switch = long("switch").help("secret switch").switch().hide();
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
    /// secret switch
    #[bpaf(hide)]
    switch: bool,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


`hide` doesn't change the parsing behavior in any way
```console
% app --argument 32
Options { argument: 32, switch: false }
```

It hides the inner parser from any help or autocompletion logic
```console
% app --help
Usage: [--argument ARG]

Available options:
        --argument <ARG>  important argument
    -h, --help            Prints help information
```

</details>
