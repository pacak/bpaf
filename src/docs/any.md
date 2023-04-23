<details>
<summary style="display: list-item;">Combinatoric usage</summary>

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
    let rest = any::<OsString>("REST")
        .help("app will pass anything unused to a child process")
        .guard(|x| x != "--help", "keep help")
        .many();
    construct!(Options { turbo, rest }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

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
    #[bpaf(any("REST"), guard(not_help, "keep help"), many)]
    /// app will pass anything unused to a child process
    rest: Vec<OsString>,
}

fn not_help(s: &OsString) -> bool {
    s != "--help"
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Capture `--turbo` flag for internal use and return everything else as is so it can be passed
to some other program. Anything except for `--turbo` here and in following examples is
consumed by `any`
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

You can keep `--help` working, but you need to add extra `guard` for that
```console
% app --turbo --help
Usage: [-t] [<REST>]...

Available positional items:
    <REST>  app will pass anything unused to a child process

Available options:
    -t, --turbo  Engage the turbo mode
    -h, --help   Prints help information
```

</details>
