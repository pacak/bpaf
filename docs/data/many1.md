```rust,id:1
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
```

```rust,id:2
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
```


In usage lines `some` items are indicated with `...`

```run,id:1,id:2
--help
```

Run inner parser as many times as possible collecting all the new results, but unlike
`many` needs to collect at least one element to succeed

```run,id:1,id:2
--argument 10 --argument 20 --switch
```

With not enough parameters to satisfy both parsers at least once - it fails

```run,id:1,id:2

```

both parsers need to succeed to create a struct

```run,id:1,id:2
--argument 10
```

 For parsers that can succeed without consuming anything such as `flag` or `switch` - `some`
only collects values as long as they produce something

```run,id:1,id:2
--switch --argument 10
```
