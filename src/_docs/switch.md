## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Produce verbose output
    // bpaf uses `switch` for `bool` fields in named
    // structs unless consumer attribute is present.
    // But it is also possible to give it explicit
    // consumer annotation to serve as a reminder:
    // #[bpaf(short, long, switch)]
    #[bpaf(short, long)]
    verbose: bool,

    #[bpaf(flag(true, false))]
    /// Build artifacts in release mode
    release: bool,

    /// Do not activate default features
    // default_features uses opposite values,
    // producing `true` when value is absent
    #[bpaf(long("no-default-features"), flag(false, true))]
    default_features: bool,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    release: bool,
    default_features: bool,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce verbose output")
        .switch();
    let release = long("release")
        .help("Build artifacts in release mode")
        .flag(true, false);
    let default_features = long("no-default-features")
        .help("Do not activate default features")
        // default_features uses opposite values,
        // producing `true` when value is absent
        .flag(false, true);

    construct!(Options {
        verbose,
        release,
        default_features,
    })
    .to_options()
}
````

In `--help` output `bpaf` shows switches as usual flags with no meta variable attached



```text
$ app --help
Usage: app [-v] [--release] [--no-default-features]

Available options:
    -v, --verbose  Produce verbose output
        --release  Build artifacts in release mode
        --no-default-features  Do not activate default features
    -h, --help     Prints help information
```


Both `switch` and `flag` succeed if value is not present, `switch` returns true, `flag` returns
second value.



```text
$ app 
Options { verbose: false, release: false, default_features: true }
```


When value is present - `switch` returns `true`, `flag` returns first value.



```text
$ app --verbose --no-default-features --detailed
Error: `--detailed` is not expected in this context
```


Like with most parsrs unless specified `switch` and `flag` consume at most one item from the
command line:



```text
$ app --no-default-features --no-default-features
Error: argument `--no-default-features` cannot be used multiple times in this context
```

