## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), switch)]
    switches: Vec<bool>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
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
````

In the generated usage lines `many` items are indicated with `...`



```text
$ app --help
Usage: app [--argument=ARG]... [--switch]...

Available options:
        --argument=ARG  important argument
        --switch        some switch
    -h, --help          Prints help information
```


Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything



```text
$ app --argument 10 --argument 20
Options { argument: [10, 20], switches: [false] }
```


If there's no matching parameters - it would produce an empty vector. Note, in case of
[`switch`](SimpleParser::switch) parser or other parsers that can succeed without consuming anything
it would capture that value so `many` captures the first one of those.
You can use [`req_flag`](SimpleParser::req_flag) to avoid that.



```text
$ app 
Options { argument: [], switches: [false] }
```


For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they consume something or at least one



```text
$ app --switch --switch
Options { argument: [], switches: [true, true] }
```

