<details>
<summary>Combinatoric usage</summary>

```no_run
# use std::ffi::OsString;
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    arg: OsString,
    pos: Option<OsString>,
}

pub fn options() -> OptionParser<Options> {
    let arg = long("arg").help("consume a String").argument("ARG").os();
    let pos = positional("POS")
        .help("consume an OsString")
        .os()
        .optional();

    construct!(Options { arg, pos }).to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use std::ffi::OsString;
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// consume a String
    arg: String,
    /// consume an OsString
    #[bpaf(positional)]
    pos: Option<OsString>,
}
```

</details>
<details>
<summary>Examples</summary>


adding .os() at the end allows to consume `OsString` encoded items
```console
% app --arg arg pos
Options { arg: "arg", pos: Some("pos") }
```

other modifiers still work:
```console
% app --arg string
Options { arg: "string", pos: None }
```

</details>
