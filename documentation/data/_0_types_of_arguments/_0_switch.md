#### Options, switches or flags

Options or flags usually starts with a dash, a single dash for short options and a double dash for
long one. Several short options can usually be squashed together with a single dash in front of
them to save on typing: `-vvv` can be parsed the same as `-v -v -v`. Options don't have any
other information apart from being there or not. Relative position usually does not matter and
`--alpha --beta` should parse the same as `--beta --alpha`.

<div class="code-wrap">
<pre>
$ cargo <span style="font-weight: bold">--help</span>
$ ls <span style="font-weight: bold">-la</span>
$ ls <span style="font-weight: bold">--time --reverse</span>
</pre>
</div>


```rust,id:1
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    release: bool,
    default_features: bool,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce verbose output")
        .switch();
    let release = long("release")
        .help("Build artifacts in release mode")
        .flag(true, false);
    let default_features = long("no-default-features")
        .help("Do not activate default features")
        // default_features uses opposite values,
        // producing `true` when value is absent
        .flag(false, true);

    construct!(Options {
        verbose,
        release,
        default_features,
    })
    .to_options()
}
```

```run,id:1
--help
```
For more detailed info see [`NamedArg::switch`] and
[`NamedArg::flag`]
