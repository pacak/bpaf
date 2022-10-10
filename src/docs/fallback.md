<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    version: usize,
}
pub fn options() -> OptionParser<Options> {
    let version = long("version").argument("VERS").fallback(42);
    construct!(Options { version }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# #[allow(dead_code)]
pub struct Options {
    #[bpaf(argument("VERS"), fallback(42))]
    version: usize,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Allows you to specify a default value used when argument is not specified
```console
% app 
Options { version: 42 }
```

If value is present - fallback value is ignored
```console
% app --version 10
Options { version: 10 }
```

Parsing errors are preserved and preserved to user
```console
% app --version ten
Couldn't parse "ten": invalid digit found in string
```

`bpaf` encases parsers with fallback value in usage with `[]`
```console
% app --help
Usage: [--version VERS]

Available options:
        --version <VERS>
    -h, --help            Prints help information
```

</details>
