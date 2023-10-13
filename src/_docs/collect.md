## Derive example

````rust
# use bpaf::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument::<u32>("ARG"), collect)]
    argument: BTreeSet<u32>,
    /// some switch
    #[bpaf(long("switch"), switch, collect)]
    switches: BTreeSet<bool>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Options {
    argument: BTreeSet<u32>,
    switches: BTreeSet<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .collect();
    let switches = long("switch").help("some switch").switch().collect();
    construct!(Options { argument, switches }).to_options()
}
````

In usage lines `collect` items are indicated with `...`



```text
$ app --help
Usage: app --argument=ARG... [--switch]...

Available options:
        --argument=ARG  important argument
        --switch        some switch
    -h, --help          Prints help information
```


Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything



```text
$ app --argument 10 --argument 20 --argument 20
Options { argument: {10, 20}, switches: {false} }
```


If there's no matching parameters - it would produce an empty collection. Note, in case of
[`switch`](SimpleParser::switch) parser or other parsers that can succeed without consuming
anything `collect` would capture that value. You can use [`req_flag`](SimpleParser::req_flag)
to only collect values that are present.



```text
$ app 
Options { argument: {}, switches: {false} }
```


For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they produce something



```text
$ app --switch --switch
Options { argument: {}, switches: {true} }
```

