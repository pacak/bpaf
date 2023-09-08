#### What gets generated

Usually calling derive macro on a type generates a trait implementation for this type. With
bpaf it's slightly different. It instead generates a function with a name that depends on the
name of the type and gives either a composable parser (`Parser`) or option parser
(`OptionParser`) back.

You can customize the function name with `generate` annotation at the top level:

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, generate(my_options))]
pub struct Options {
    /// A simple switch
    switch: bool
}

fn main() {
    println!("{:?}", my_options().run());
}
# pub fn options() -> OptionParser<Options> { my_options() }
```

```run,id:1
--help
```


By default function shares the same visibility as the structure, but you can make it module
private with `private` annotation:


```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, generate(my_options), private)]
pub struct Options {
    /// A simple switch
    switch: bool
}

fn main() {
    println!("{:?}", my_options().run());
}
# pub fn options() -> OptionParser<Options> { my_options() }
```

```run,id:1
--help
```
