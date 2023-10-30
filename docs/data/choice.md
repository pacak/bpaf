## Combinatoric example

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    desert: Option<&'static str>,
}

pub fn options() -> OptionParser<Options> {
    let desert = ["apple", "banana", "orange", "grape", "strawberry"]
        .iter()
        .map(|name| {
            long(name)
                .help("Pick one of the options")
                .req_flag(*name)
                .boxed()
        });
    let desert = choice(desert).optional();
    construct!(Options { desert }).to_options()
}
```


Here [`choice`] function is used to create an option for each possible desert item

```run,id:1
--help
```

User can pick any item

```run,id:1
--apple
```

Since parser consumes only one value you can't specify multiple flags of the same type

```run,id:1
--orange --grape
```

And [`Parser::optional`] makes it so when value is not specified - `None` is produced instead

```run,id:1

```
