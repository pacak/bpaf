## Derive example

````rust
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("NAME"))]
    /// Use a custom user name
    name: String,
    #[bpaf(pure(330))]
    money: u32,
}
````

## Combinatoric example

````rust
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    money: u32,
}

pub fn options() -> OptionParser<Options> {
    // User can customise a name
    let name = long("name").help("Use a custom user name").argument("NAME");
    // but not starting amount of money
    let money = pure(330);
    construct!(Options { name, money }).to_options()
}
````

`pure` does not show up in `--help` message



```text
$ app --help
Usage: app --name=NAME

Available options:
        --name=NAME  Use a custom user name
    -h, --help       Prints help information
```


And there's no way to alter the value from the command line



```text
$ app --name Bob
Options { name: "Bob", money: 330 }
```


Any attempts to do so would result in an error :)



```text
$ app --money 100000 --name Hackerman
Error: `--money` is not expected in this context
```

