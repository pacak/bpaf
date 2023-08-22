#### What gets generated

Usually calling derive macro on a type generates code to derive a trait implementation for this
type. With bpaf it's slightly different. It instead generates a function with a name that
depends on the name of the type and gives either a composable parser (`Parser`) or option parser
(`OptionParser`) back.

You can customize the function name with `generate` annotation at the top level:

```rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, generate(my_options))]
pub struct Options {
    /// A simple switch
    switch: bool
}


fn main() {
    let opts = my_options().run();
    println!("{:?}", opts);
}
```
