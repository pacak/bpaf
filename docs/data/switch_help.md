## Derive example

```rust,id:1
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
```

## Combinatoric example

```rust,id:2
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
```

When `--help` used once it renders shorter version of the help information

```run,id:1,id:2
--help
```

When used twice - it renders full version. Documentation generator uses full
version as well

```run,id:1,id:2
--help --help
```

Presence or absense of a help message should not affect the parser's output

```run,id:1,id:2
output.txt
```
