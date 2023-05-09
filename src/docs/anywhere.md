<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
}

fn toggle_option(name: &'static str, help: &'static str) -> impl Parser<bool> {
    any::<String, _, _>(name, move |s: String| {
        if let Some(rest) = s.strip_prefix('+') {
            (rest == name).then_some(true)
        } else if let Some(rest) = s.strip_prefix('-') {
            (rest == name).then_some(false)
        } else {
            None
        }
    })
    .help(help)
    .anywhere()
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_option("backing", "Backing status").fallback(false);
    let xinerama = toggle_option("xinerama", "Xinerama status").fallback(true);
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    construct!(Options {
        turbo,
        backing,
        xinerama,
    })
    .to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


This example shows how to parse some very unusual options, same style as used by Xorg
`-backing` disables backing `+backing` enables it, usual restrictions and combinations apply:
fails if present more than once, can be further transformed with combinators.
By default `xinerama` is enabled, anything else is disabled
```console
% app 
Options { turbo: false, backing: false, xinerama: true }
```

Strange things we added can be mixed with the regular options
```console
% app --turbo +backing -xinerama
Options { turbo: true, backing: true, xinerama: false }
```

As expected - order doesn't matter
```console
% app +backing --turbo
Options { turbo: true, backing: true, xinerama: true }
```

--help will try to render it but you can always `.hide` it and add your own lines
with `.header` or `.footer` methods on `OptionParser`.
```console
% app --help
Usage: [-t] [<backing>] [<xinerama>]

Available options:
    -t, --turbo  Engage the turbo mode
    <backing>    Backing status
    <xinerama>   Xinerama status
    -h, --help   Prints help information
```

</details>
