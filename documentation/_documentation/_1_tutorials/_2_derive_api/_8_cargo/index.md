#### Making a cargo command

To make a cargo command you should pass its name as a parameter to `options`. In this example,
`bpaf` will parse extra parameter cargo passes and you will be able to use it either directly
with `cargo run` from the repository, running it by `cargo-asm` name or with `cargo asm` name.

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options("asm"))]
pub struct Options {
    /// A simple switch
    switch: bool
}


fn main() {
    let opts = options().run();
    println!("{:?}", opts);
}
```
