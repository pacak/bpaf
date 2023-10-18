## Derive example

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, env("USER"), argument("USER"))]
    /// Custom user name
    username: String,
}
```

## Combinatoric example

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    username: String,
}

pub fn options() -> OptionParser<Options> {
    let username = long("username")
        .short('u')
        .env("USER")
        .help("Custom user name")
        .argument::<String>("USER");
    construct!(Options { username }).to_options()
}
```


Help message shows env variable name along with its value, if it is set

```run,id:1,id:2
--help
```

When both named argument and environment variable are present - name takes the priority
```run,id:1,id:2
-u bob
```

Otherwise parser falls back to the environment variable or fails with a usual "value not found"
type of error if the environment variable is not set either.

```run,id:1,id:2

```
