#### Parsing subcommands

Easiest way to define a group of subcommands is to have them inside the same enum with variant
constructors annotated with `#[bpaf(command("name"))]` with or without the name

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum Options {
    #[bpaf(command("run"))]
    /// Run a binary
    Run {
        /// Name of a binary crate
        name: String
    },

    /// Run a self test
    #[bpaf(command)]
    Test
}

fn main() {
    let opts = options().run();
    println!("{:?}", opts);
}
```
