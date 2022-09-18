<details>
<summary>Combinatoric usage</summary>

```no_run
# use std::path::PathBuf;
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    file: PathBuf,
    name: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let file = positional::<PathBuf>("FILE").help("File to use");
    // sometimes you can get away with not specifying type in positional's turbofish
    let name = positional("NAME").help("Name to look for").optional();
    construct!(Options { file, name }).to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use std::path::PathBuf;
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// File to use
    #[bpaf(positional::<PathBuf>("FILE"))]
    file: PathBuf,
    /// Name to look for
    #[bpaf(positional("NAME"))]
    name: Option<String>,
}
```

</details>
<details>
<summary>Examples</summary>


Positionals are consumed left to right, one at a time, no skipping unless the value is optional
```console
% app main.rs
Options { file: "main.rs", name: None }
```

Both positionals are present
```console
% app main.rs hello
Options { file: "main.rs", name: Some("hello") }
```

Only `name` is optional in this example, not specifying `file` is a failure
```console
% app 
Expected <FILE>, pass --help for usage information
```

And usage information
```console
% app --help
Usage: <FILE> [<NAME>]

Available positional items:
    <FILE>  File to use
    <NAME>  Name to look for

Available options:
    -h, --help  Prints help information
```

</details>
