#### Getting started with derive macro

Let's take a look at a simple example

```no_run
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
    let opts = options().run();
    println!("{:?}", opts)
}
```

`bpaf` is trying hard to guess what you are trying to achieve just from the types so it will
pick up types, doc comment, presence or absence of names, but it is possible to customize all
of it, add custom transformations, validations and more.
