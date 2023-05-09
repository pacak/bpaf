```no_run
use bpaf::*;
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
    extensions: Vec<(String, bool)>,
}

// matches literal name prefixed with - or +
fn toggle_options(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<bool> {
    any(meta, move |s: String| {
        if let Some(suf) = s.strip_prefix('+') {
            (suf == name).then_some(true)
        } else if let Some(suf) = s.strip_prefix('-') {
            (suf == name).then_some(false)
        } else {
            None
        }
    })
    .help(help)
    .anywhere()
}

// matches literal +ext and -ext followed by extension name
fn extension() -> impl Parser<(String, bool)> {
    let state = any("(+|-)ext", |s: String| match s.as_str() {
        "-ext" => Some(false),
        "+ext" => Some(true),
        _ => None,
    })
    .anywhere();

    let name = positional::<String>("EXT")
        .help("Extension to enable or disable, see documentation for the full list");
    construct!(state, name).adjacent().map(|(a, b)| (b, a))
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_options("(+|-)backing", "backing", "Set backing status").fallback(false);
    let xinerama =
        toggle_options("(+|-)xinerama", "xinerama", "Set Xinerama status").fallback(true);
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let extensions = extension().many();
    construct!(Options {
        turbo,
        backing,
        xinerama,
        extensions,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}

```
<details>
<summary style="display: list-item;">Examples</summary>


`xorg` takes parameters in a few different ways, notably as a long name starting with plus or
minus with different defaults
```console
% app -xinerama +backing
Options { turbo: false, backing: true, xinerama: false, extensions: [] }
```

But also as `+ext name` and `-ext name` to enable or disable an extensions
```console
% app --turbo +ext banana -ext apple
Options { turbo: true, backing: false, xinerama: true, extensions: [("banana", true), ("apple", false)] }
```

While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.
```console
% app --help
Usage: [-t] [(+|-)backing] [(+|-)xinerama] [(+|-)ext <EXT>]...

Available options:
    -t, --turbo    Engage the turbo mode
    (+|-)backing   Set backing status
    (+|-)xinerama  Set Xinerama status
  (+|-)ext <EXT>
    <EXT>          Extension to enable or disable, see documentation for the full list

    -h, --help     Prints help information
```

</details>
