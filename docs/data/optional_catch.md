Parsers in those examples show how to parse argument using two different parsers. If your
application expects two different types of input with the same name (numeric OR arbitrary
string literals) you should try to combine them into a single enum. This example parses them
separately for simplicity.

## Derive example

```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, argument("PX"), optional, catch)]
    /// Height of a rectangle
    height: Option<usize>,

    #[bpaf(long("height"), argument("PX"), optional, hide)]
    height_str: Option<String>,

    #[bpaf(long, argument("PX"), optional)]
    /// Width of a rectangle
    width: Option<usize>,

    #[bpaf(long("width"), argument("PX"), optional, hide)]
    width_str: Option<String>,
}
```

## Combinatoric example

```rust,id:2
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    height: Option<usize>,
    height_str: Option<String>,
    width: Option<usize>,
    width_str: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .optional()
        .catch();

    let height_str = long("height").argument::<String>("PX").optional().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .optional();

    let width_str = long("width").argument::<String>("PX").optional().hide();

    construct!(Options {
        height,
        height_str,
        width,
        width_str
    })
    .to_options()
}
```


Despite parser producing a funky value - help looks like you would expect from a parser that
takes two values

```run,id:1,id:2
--help
````

When executed with no parameters it produces four `None` values - all parsers succeed by the
nature of them being [`optional`](Parser::optional)

```run,id:1,id:2

```

When executed with expected parameters fields with `usize` get their values

```run,id:1,id:2
--height 100 --width 100
```

With incorrect value for `--height` parameter inner part of `height` parser fails, `optional`
combined with `catch` handles this failure and produces `None` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string

```run,id:1,id:2
--height ten
```

In case of wrong `--width` - parser `width` fails, parser for `optional` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser

```run,id:1,id:2
--width ten
```
