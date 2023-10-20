## Derive example

```rust,id:1
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
```

## Combinatoric example

```rust,id:2
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
```


In `--help` output `bpaf` shows switches as usual flags with no meta variable attached


```run,id:1,id:2
--help
```

Both `switch` and `flag` succeed if value is not present, `switch` returns true, `flag` returns
second value.

```run,id:1,id:2

```

When value is present - `switch` returns `true`, `flag` returns first value.

```run,id:1,id:2
--verbose --no-default-features --detailed
```

Like with most parsrs unless specified `switch` and `flag` consume at most one item from the
command line:

```run,id:1,id:2
--no-default-features --no-default-features
```
