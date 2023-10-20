## Derive example

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // you can specify exact type argument should produce
    // for as long as it implements `FromStr`
    #[bpaf(short, long, argument::<String>("NAME"))]
    /// Specify user name
    name: String,
    // but often rust can figure it out from the context,
    // here age is going to be `usize`
    #[bpaf(argument("AGE"), fallback(18), display_fallback)]
    /// Specify user age
    age: usize,
}
```

## Combinatoric example

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    age: usize,
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    let age = long("age")
        .help("Specify user age")
        // but often rust can figure it out from the context,
        // here age is going to be `usize`
        .argument("AGE")
        .fallback(18)
        .display_fallback();

    construct!(Options { name, age }).to_options()
}
```

```run,id:1,id:2
--help
```

`--help` shows arguments as a short name with attached metavariable

Value can be separated from flag by space, `=` sign

```run,id:1,id:2
--name Bob --age 12
```

```run,id:1,id:2
--name "Bob" --age=12
```

```run,id:1,id:2
--name=Bob
```

```run,id:1,id:2
--name="Bob"
```

Or in case of short name - be directly adjacent to it

```run,id:1,id:2
-nBob
```

For long names - this doesn't work since parser can't tell where name
stops and argument begins:

```run,id:1,id:2
--age12
```

Either way - value is required, passing just the argument name results in parse failure

```run,id:1,id:2
--name
```
