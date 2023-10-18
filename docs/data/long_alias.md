```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, long("Alpha"))]
    /// Parameter Alpha, '--Alpha' is a hidden alias
    alpha: u32,

    #[bpaf(short, long("Beta"))]
    /// Parameter Beta, '--Beta' is a visible name
    beta: u32,
}
```

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    /// Parameter Alpha, '--Alpha' is a hidden alias
    alpha: u32,

    /// Parameter Beta, '--Beta' is a visible name
    beta: u32,
}

pub fn options() -> OptionParser<Options> {
    let alpha = long("alpha")
        .long("Alpha")
        .help("Parameter Alpha, '--Alpha' is a hidden alias")
        .argument("ARG");
    let beta = short('b')
        .long("Beta")
        .help("Parameter Beta, '--Beta' is a visible name")
        .argument("ARG");
    construct!(Options { alpha, beta }).to_options()
}
```

For each argument first long name stays visible, the rest become hidden aliases. Here `--alpha` and
`--Beta` are visible names

```run,id:1,id:2
--help
```

```run,id:1,id:2
--alpha 10 --beta 330
```

and `--Alpha` is a hidden alias since it was defined on a named structure that already had a long
name.

```run,id:1,id:2
--Alpha 42 --Beta 15
```
