#### Implementing `Xorg(1)`: parsing `+ext name` and `-ext name` into a `(String, bool)`

A full example is available in examples folder at bpaf's github.

This example parses a literal `+ext` or `-ext` followed by an arbitrary extension name a pair
containing extension name and status. As with anything unusal parser utilizes [`any`] with
[`SimpleParser::anywhere`] to match initial `+ext` and `-ext`, alternative approach is going to
be using a combination of two [`literal`] functions. Once the tag is parsed - string that
follows it is parsed with [`adjacent`](crate::SimpleParser::adjacent) restriction.


```rust,id:1
use bpaf::*;
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    extensions: Vec<(String, bool)>,
}

// matches literal +ext and -ext followed by an extension name
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
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let extensions = extension().many();
    construct!(Options {
        extensions,
        turbo,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
```


```run,id:1
--help
```

```run,id:1
+ext banana -t -ext apple
```
