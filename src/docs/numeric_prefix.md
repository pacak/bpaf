```no_run
use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    prefix: Option<usize>,
    command: String,
}

pub fn options() -> OptionParser<Options> {
    let prefix = positional::<usize>("PREFIX")
        .help("Optional numeric command prefix")
        .optional()
        .catch();
    let command = positional::<String>("COMMAND").help("Required command name");

    construct!(Options { prefix, command }).to_options()
}

fn main() {
    println!("{:#?}", options().run());
}

```
<details>
<summary style="display: list-item;">Examples</summary>


If `bpaf` can parse first positional argument as number - it becomes a numeric prefix
```console
% app 10 eat
Options { prefix: Some(10), command: "eat" }
```

Otherwise it gets ignored
```console
% app "just eat"
Options { prefix: None, command: "just eat" }
```

If validation passes but second argument is missing - there's no fallback
```console
% app 10
Expected <COMMAND>, pass --help for usage information
```

Help should reflect the fact that the prefix is optional
```console
% app --help
Usage: [<PREFIX>] <COMMAND>

Available positional items:
    <PREFIX>   Optional numeric command prefix
    <COMMAND>  Required command name

Available options:
    -h, --help  Prints help information
```

</details>
