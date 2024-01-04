#### Parsing subcommands

The easiest way to define a group of subcommands is to have them inside the same enum with variant
constructors annotated with `#[bpaf(command("name"))]` with or without the name


```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command("run"))]
    /// Run a binary
    Run {
        /// Name of a binary crate
        name: String,
    },
    /// Run a self test
    #[bpaf(command)]
    Test,
}
```

Help message lists subcommand

```run,id:1
--help
```

Commands have their own arguments and their own help
```run,id:1
run --help
```

```run,id:1
run --name Bob
```

```run,id:1
test
```

And even if `--name` is valid in scope of `run` command - it's not valid for `test`

```run,id:1
test --name bob
```
