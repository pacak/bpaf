<details>
<summary>Combinatoric usage</summary>

```no_run
# use std::{path::PathBuf, str::FromStr};
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    coin: Coin,
    file: PathBuf,
    name: Option<String>,
}

/// A custom datatype that doesn't implement [`FromOsStr`] but implements [`FromStr`]
#[derive(Debug, Clone, Copy)]
enum Coin {
    Heads,
    Tails,
}
impl FromStr for Coin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heads" => Ok(Coin::Heads),
            "tails" => Ok(Coin::Tails),
            _ => Err(format!("Expected 'heads' or 'tails', got '{}'", s)),
        }
    }
}

pub fn options() -> OptionParser<Options> {
    let file = positional::<PathBuf>("FILE").help("File to use");
    // sometimes you can get away with not specifying type in positional's turbofish
    let coin = long("coin")
        .help("Coin toss results")
        .argument::<FromUtf8<Coin>>("COIN")
        .fallback(Coin::Heads);
    let name = positional::<String>("NAME")
        .help("Name to look for")
        .optional();
    construct!(Options { coin, file, name }).to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use std::{path::PathBuf, str::FromStr};
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// Coin toss results
    #[bpaf(argument::<FromUtf8<Coin>>("COIN"), fallback(Coin::Heads))]
    coin: Coin,

    /// File to use
    #[bpaf(positional::<PathBuf>("FILE"))]
    file: PathBuf,
    /// Name to look for
    #[bpaf(positional("NAME"))]
    name: Option<String>,
}

/// A custom datatype that doesn't implement [`FromOsStr`] but implements [`FromStr`]
#[derive(Debug, Clone, Copy)]
enum Coin {
    Heads,
    Tails,
}
impl FromStr for Coin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heads" => Ok(Coin::Heads),
            "tails" => Ok(Coin::Tails),
            _ => Err(format!("Expected 'heads' or 'tails', got '{}'", s)),
        }
    }
}
```

</details>
<details>
<summary>Examples</summary>


Positionals are consumed left to right, one at a time, no skipping unless the value is optional
```console
% app main.rs
Options { coin: Heads, file: "main.rs", name: None }
```

Both positionals are present
```console
% app main.rs hello
Options { coin: Heads, file: "main.rs", name: Some("hello") }
```

To parse items without having to write `FromOsStr` instance you can use `FromUtf8` helper
type?
```console
% app main.rs --coin tails
Options { coin: Tails, file: "main.rs", name: None }
```

Only `name` is optional in this example, not specifying `file` is a failure
```console
% app 
Expected <FILE>, pass --help for usage information
```

And usage information
```console
% app --help
Usage: [--coin COIN] <FILE> [<NAME>]

Available positional items:
    <FILE>  File to use
    <NAME>  Name to look for

Available options:
        --coin <COIN>  Coin toss results
    -h, --help         Prints help information
```

</details>
