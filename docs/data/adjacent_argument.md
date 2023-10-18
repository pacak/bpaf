```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("SPEC"), adjacent)]
    /// Package to use
    package: String,
}
```

```rust,id:2
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    package: String,
}

fn package() -> impl Parser<String> {
    long("package")
        .short('p')
        .help("Package to use")
        .argument("SPEC")
        .adjacent()
}

pub fn options() -> OptionParser<Options> {
    construct!(Options { package() }).to_options()
}
```

```run,id:1,id:2
--help
```

As with regular [`argument`](SimpleParser::argument) its `adjacent` variant is required by default

```run,id:1,id:2

```

But unlike regular variant `adjacent` requires name and value to be separated by `=` only

```run,id:1,id:2
-p=htb
```

```run,id:1,id:2
--package=bpaf
```

Separating them by space results in parse failure

```run,id:1,id:2
--package htb
```

```run,id:1,id:2
-p htb
```

```run,id:1,id:2
--package
```
