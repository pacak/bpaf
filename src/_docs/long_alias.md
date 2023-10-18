## Derive example

````rust
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
````

## Combinatoric example

````rust
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
````

For each argument first long name stays visible, the rest become hidden aliases. Here `--alpha` and
`--Beta` are visible names



```text
$ app --help
Usage: app --alpha=ARG -b=ARG

Available options:
        --alpha=ARG  Parameter Alpha, '--Alpha' is a hidden alias
    -b, --Beta=ARG   Parameter Beta, '--Beta' is a visible name
    -h, --help       Prints help information
```



```text
$ app --alpha 10 --beta 330
Error: no such flag: `--beta`, did you mean `--Beta`?
```


and `--Alpha` is a hidden alias since it was defined on a named structure that already had a long
name.



```text
$ app --Alpha 42 --Beta 15
Options { alpha: 42, beta: 15 }
```

