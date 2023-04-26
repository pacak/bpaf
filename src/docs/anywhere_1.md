<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct MultiArg {
    set: (),
    name: String,
    value: String,
}

pub fn options() -> OptionParser<Options> {
    let set = long("set").req_flag(());
    let name = positional("ARG");
    let value = positional("ARG");
    let multi_arg = construct!(MultiArg { set, name, value })
        .anywhere()
        .optional();

    let turbo = long("turbo").switch();
    construct!(Options { multi_arg, turbo }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, optional)]
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(anywhere)]
# #[allow(dead_code)]
pub struct MultiArg {
    #[bpaf(long)]
    set: (),
    #[bpaf(positional)]
    name: String,
    #[bpaf(positional)]
    value: String,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


It's possible to implement multi argument options by using required flag followed by one or
more positional items
```console
% app --turbo --set name Bob
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }
```

Other flags can go on either side of items
```console
% app --set name Bob --turbo
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }
```

But not in between
```console
% app --set name --turbo Bob
Expected <ARG>, pass --help for usage information
```

</details>
