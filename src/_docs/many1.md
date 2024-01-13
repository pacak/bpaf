
````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument("ARG"), some("want at least one argument"))]
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), req_flag(true), some("want at least one switch"))]
    switches: Vec<bool>,
}
````

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
        .some("want at least one argument");
    let switches = long("switch")
        .help("some switch")
        .req_flag(true)
        .some("want at least one switch");
    construct!(Options { argument, switches }).to_options()
}
````

In usage lines `some` items are indicated with `...`



```text
$ app --help
Usage: app --argument=ARG... --switch...

Available options:
        --argument=ARG  important argument
        --switch        some switch
    -h, --help          Prints help information
```


Run inner parser as many times as possible collecting all the new results, but unlike
`many` needs to collect at least one element to succeed



```text
$ app --argument 10 --argument 20 --switch
Options { argument: [10, 20], switches: [true] }
```


With not enough parameters to satisfy both parsers at least once - it fails



```text
$ app 
Error: want at least one argument
```


both parsers need to succeed to create a struct



```text
$ app --argument 10
Error: want at least one switch
```


For parsers that can succeed without consuming anything such as `flag` or `switch` - `some`
only collects values as long as they produce something



```text
$ app --switch --argument 10
Options { argument: [10], switches: [true] }
```

