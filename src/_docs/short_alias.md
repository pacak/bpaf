
````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, short('A'))]
    /// Parameter Alpha, '-A' is a hidden alias
    alpha: u32,

    #[bpaf(long, short('B'))]
    /// Parameter Beta, '-B' is a visible name
    beta: u32,
}
````

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    /// Parameter Alpha, '-A' is a hidden alias
    alpha: u32,

    /// Parameter Beta, '-B' is a visible name
    beta: u32,
}

pub fn options() -> OptionParser<Options> {
    let alpha = short('a')
        .short('A')
        .help("Parameter Alpha, '-A' is a hidden alias")
        .argument("ARG");
    let beta = long("beta")
        .short('B')
        .help("Parameter Beta, '-B' is a visible name")
        .argument("ARG");
    construct!(Options { alpha, beta }).to_options()
}
````

For each argument first short name stays visible, the rest become hidden aliases. Here `-a` and
`-B` are visible names



```text
$ app --help
Usage: app -a=ARG -B=ARG

Available options:
    -a=ARG          Parameter Alpha, '-A' is a hidden alias
    -B, --beta=ARG  Parameter Beta, '-B' is a visible name
    -h, --help      Prints help information
```



```text
$ app -a 10 -B 330
Options { alpha: 10, beta: 330 }
```


and `-A` is a hidden alias since it was defined on a named structure that already had a short
name.



```text
$ app -A 42 --beta 15
Options { alpha: 42, beta: 15 }
```

