#### Implementing `Xorg(1)`: parsing `+xinerama` and `-xinerama` into a `bool`

A full example is available in examples folder at bpaf's github.

This example implements a parser for a named argument that starts with either `+` or `-` and
gets parsed into a `bool`. As with anything unusual parser utilizes [`any`] function to check
if input starts with `+` or `-`, and if so - checks if it matches predefined name, then producing
`true` or `false` depending on the first character. This logic is placed in `toggle` function.

Since custom items that start with a `-` can be interpreted as a set of short flags - it's a
good idea to place parsers created by `toggle` before regular parsers.

```rust,id:1
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
}

// matches literal name prefixed with `-` or `+`.
// If name is not specified - parser fails with "not found" type of error.
fn toggle(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<bool> {
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

pub fn options() -> OptionParser<Options> {
    let backing = toggle("(+|-)backing", "backing", "Set backing status")
        .fallback(false)
        .display_fallback();
    let xinerama = toggle("(+|-)xinerama", "xinerama", "Set Xinerama status")
        .fallback(true)
        .display_fallback();
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    construct!(Options {
        backing,
        xinerama,
        turbo,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
```

Help message lists all the custom items

```run,id:1
--help
```

You can use custom parsers alongside with regular parsers

```run,id:1
-xinerama --turbo +backing
```

And default values for toggle parsers are set according to fallback values.

```run,id:1

```
