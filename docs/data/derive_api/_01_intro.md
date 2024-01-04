#### Getting started with derive macro

Let's take a look at a simple example


```rust,id:1
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    switch: bool,

    /// A custom argument
    argument: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

`bpaf` generates a help message

```run,id:1
--help
```

And parsers for two items: numeric argument is required, boolean switch is optional and fall back value
is `false`.

```run,id:1
--switch
```

```run,id:1
--switch --argument 42
```

```run,id:1
--argument 42
```

`bpaf` is trying hard to guess what you are trying to achieve just from the types so it will
pick up types, doc comments, presence or absence of names, but it is possible to customize all
of it, add custom transformations, validations and more.
