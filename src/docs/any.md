<details>
<summary>Combinatoric usage</summary>

```no_run
# use std::ffi::OsString;
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    turbo: bool,
    rest: Vec<OsString>,
}

pub fn options() -> OptionParser<Options> {
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let rest = any("REST").many();
    construct!(Options { turbo, rest }).to_options()
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
    #[bpaf(short, long)]
    /// Engage the turbo mode
    turbo: bool,
    #[bpaf(any("REST"))]
    rest: Vec<OsString>,
}
```

</details>
<details>
<summary>Examples</summary>


Capture `--turbo` flag for internal use and return everything else as is so it can be passed
to some other program
```console
% app --turbo git commit -m "hello world"
Options { turbo: true, rest: ["git", "commit", "-m", "hello world"] }
```

Or just capture and return everything
```console
% app git commit -m "hello world"
Options { turbo: false, rest: ["git", "commit", "-m", "hello world"] }
```

Doesn't have to be in order either
```console
% app git commit -m="hello world" --turbo
Options { turbo: true, rest: ["git", "commit", "-m=hello world"] }
```

</details>
