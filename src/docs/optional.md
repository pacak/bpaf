<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    version: Option<usize>,
    feature: Option<String>,
}
pub fn options() -> OptionParser<Options> {
    let version = long("version").argument("VERS").optional();
    let feature = long("feature").argument("FEAT").optional();
    construct!(Options { version, feature }).to_options()
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
    #[bpaf(argument("VERS"))]
    version: Option<usize>,
    #[bpaf(argument("FEAT"))]
    feature: Option<String>,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Missing arguments are turned into None
```console
% app 
Options { version: None, feature: None }
```

Present values are Some
[`optional`](Parser::optional)
```console
% app --version 10
Options { version: Some(10), feature: None }
```

You can specify both
```console
% app --version 10 --feature feat
Options { version: Some(10), feature: Some("feat") }
```

bpaf encases optional arguments in usage with []
```console
% app --help
Usage: [--version VERS] [--feature FEAT]

Available options:
        --version <VERS>
        --feature <FEAT>
    -h, --help            Prints help information
```

</details>
