## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(positional("OUTPUT"))]

    /// Brief option description
    ///
    ///  Detailed help description
    ///  that can span multiple lines
    //  ^ note extra spaces before when that preserve the linebreaks
    output: Option<String>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    output: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let output = positional("OUTPUT")
        .help(
"Brief option description

 Detailed help description
 that can span multiple lines")
    //  ^ note extra spaces before when that preserve the linebreaks
        .optional();

    construct!(Options {
        output
    })
    .to_options()
}
````

When `--help` used once it renders shorter version of the help information



```text
$ app --help
Usage: app [OUTPUT]

Available positional items:
    OUTPUT      Brief option description

Available options:
    -h, --help  Prints help information
```


When used twice - it renders full version. Documentation generator uses full
version as well



```text
$ app --help --help
Usage: app [OUTPUT]

Available positional items:
    OUTPUT      Brief option description
                 Detailed help description
                that can span multiple lines

Available options:
    -h, --help  Prints help information
```


Presence or absense of a help message should not affect the parser's output



```text
$ app output.txt
Options { output: Some("output.txt") }
```

