## Important restriction

To parse positional arguments from a command line you should place parsers for all your
named values before parsers for positional items and commands. In derive API fields parsed as
positional items or commands should be at the end of your `struct`/`enum`. The same rule applies
to parsers with positional fields or commands inside: such parsers should go to the end as well.

Use [`check_invariants`](OptionParser::check_invariants) in your test to ensure correctness.

For example for non-positional `non_pos` and positional `pos` parsers

````rust
# use bpaf::*;
# fn foo() {
# let non_pos = || short('n').switch();
# let pos = ||positional::<String>("POS");
let valid = construct!(non_pos(), pos());
let invalid = construct!(pos(), non_pos());
# let _ = (valid, invalid);
# }
````

**`bpaf` panics during help generation unless this restriction holds**

## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Display detailed information
    #[bpaf(short, long)]
    verbose: bool,

    // You must place positional items and commands after all named parsers
    #[bpaf(positional("CRATE"))]
    /// Crate name to use
    crate_name: String,

    #[bpaf(positional("FEATURE"))]
    /// Display information about this feature
    feature_name: Option<String>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    crate_name: String,
    feature_name: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Display detailed information")
        .switch();

    let crate_name = positional("CRATE").help("Crate name to use");

    let feature_name = positional("FEATURE")
        .help("Display information about this feature")
        .optional();

    construct!(Options {
        verbose,
        // You must place positional items and commands after all named parsers
        crate_name,
        feature_name
    })
    .to_options()
}
````

Positional items show up in a separate group of arguments if they contain a help message,
otherwise they will show up only in **Usage** part.



```text
$ app --help
Usage: app [-v] CRATE [FEATURE]

Available positional items:
    CRATE          Crate name to use
    FEATURE        Display information about this feature

Available options:
    -v, --verbose  Display detailed information
    -h, --help     Prints help information
```


You can mix positional items with regular items



```text
$ app --verbose bpaf
Options { verbose: true, crate_name: "bpaf", feature_name: None }
```


And since `bpaf` API expects to have non positional items consumed before positional ones - you
can use them in a different order. In this example `bpaf` corresponds to a `crate_name` field and
`--verbose` -- to `verbose`.



```text
$ app bpaf --verbose
Options { verbose: true, crate_name: "bpaf", feature_name: None }
```


In previous examples optional field `feature` was missing, this one contains it.



```text
$ app bpaf autocomplete
Options { verbose: false, crate_name: "bpaf", feature_name: Some("autocomplete") }
```


Users can use `--` to tell `bpaf` to treat remaining items as positionals - this might be
required to handle unusual items.



```text
$ app bpaf -- --verbose
Options { verbose: false, crate_name: "bpaf", feature_name: Some("--verbose") }
```



```text
$ app -- bpaf --verbose
Options { verbose: false, crate_name: "bpaf", feature_name: Some("--verbose") }
```


Without using `--` `bpaf` would only accept items that don't start with `-` as positional.



```text
$ app --detailed
Error: expected `CRATE`, got `--detailed`. Pass `--help` for usage information
```



```text
$ app --verbose
Error: expected `CRATE`, pass `--help` for usage information
```


You can use [`any`](any) to work around this restriction.

Alternatively you can use [`strict`](SimpleParser::strict) to make a parser that requires user
to separate positional items from named items with `--`.