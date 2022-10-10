<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: Vec<u32>,
    switches: Vec<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .some("want at least one argument");
    let switches = long("switch")
        .help("some switch")
        .switch()
        .some("want at least one switch");
    construct!(Options { argument, switches }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument("ARG"), some("want at least one argument"))]
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), switch, some("want at least one switch"))]
    switches: Vec<bool>,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Run inner parser as many times as possible collecting all the new results, but unlike
`many` needs to collect at least one element to succeed
```console
% app --argument 10 --argument 20 --switch
Options { argument: [10, 20], switches: [true] }
```

With not enough parameters to satisfy both parsers at least once - it fails
```console
% app 
want at least one argument
```

both parsers need to succeed to create a struct
```console
% app --argument 10
want at least one switch
```

For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they produce something
```console
% app --switch --argument 10
Options { argument: [10], switches: [true] }
```

In usage lines `some` items are indicated with `...`
```console
% app --help
Usage: --argument ARG... [--switch]...

Available options:
        --argument <ARG>  important argument
        --switch          some switch
    -h, --help            Prints help information
```

</details>
