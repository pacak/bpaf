<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
use bpaf::*;
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
        .many();
    let switches = long("switch").help("some switch").switch().many();
    construct!(Options { argument, switches }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), switch)]
    switches: Vec<bool>,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Run inner parser as many times as possible collecting all the new results
```console
% app --argument 10 --argument 20
Options { argument: [10, 20], switches: [] }
```

If there's no matching parsers - it would produce an empty vector
```console
% app 
Options { argument: [], switches: [] }
```

For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they produce something
```console
% app --switch --switch
Options { argument: [], switches: [true, true] }
```

In usage lines `many` items are indicated with `...`
```console
% app --help
Usage: --argument ARG... [--switch]...

Available options:
        --argument <ARG>  important argument
        --switch          some switch
    -h, --help            Prints help information
```

</details>
