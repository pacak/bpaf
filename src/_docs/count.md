## Derive example

You can pass `count` annotation to the right of the parser you want to count:

````rust
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Increase the verbosity level
    #[bpaf(short('v'), long("verbose"), req_flag(()), count)]
    verbosity: usize,
}
````

## Combinatoric example

````rust
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbosity: usize,
}

pub fn options() -> OptionParser<Options> {
    let verbosity = short('v')
        .long("verbose")
        .help("Increase the verbosity level")
        .req_flag(())
        .count();

    construct!(Options { verbosity }).to_options()
}
````

This example uses [`req_flag`](SimpleParser::req_flag), in help message it look similarly to
[`switch`](SimpleParser::switch) or [`flag`](SimpleParser::flag)



```text
$ app --help
Usage: app [-v]...

Available options:
    -v, --verbose  Increase the verbosity level
    -h, --help     Prints help information
```


`req_flag` succeeds and produces its value only when a flag is present on a command line.
`count` tracks those successes and discards values produced by `req_flag`:



```text
$ app -vvv --verbose
Options { verbosity: 4 }
```


No flags present on a command line - no successes - count produces 0.



```text
$ app 
Options { verbosity: 0 }
```

