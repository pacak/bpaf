## Derive example

```rust,id:1
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
```

## Combinatoric example

```rust,id:2
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
```

`pure` does not show up in `--help` message

```run,id:1,id:2
--help
```

And there's no way to alter the value from the command line

```run,id:1,id:2
--name Bob
```

Any attempts to do so would result in an error :)

```run,id:1,id:2
--money 100000 --name Hackerman
```
