Parsers in those examples show how to parse argument using two different parsers. If your
application expects two different types of input with the same name (numeric OR arbitrary
string literals) you should try to combine them into a single enum. This example parses them
separately for simplicity.

## Derive example

````rust
# use bpaf::*;
# use std::collections::BTreeSet;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, argument::<usize>("PX"), collect, catch)]
    /// Height of a rectangle
    height: BTreeSet<usize>,

    #[bpaf(long("height"), argument::<String>("PX"), collect, hide)]
    height_str: BTreeSet<String>,

    #[bpaf(long, argument::<usize>("PX"), collect)]
    /// Width of a rectangle
    width: BTreeSet<usize>,

    #[bpaf(long("width"), argument::<String>("PX"), collect, hide)]
    width_str: BTreeSet<String>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
# use std::collections::BTreeSet;
#[derive(Debug, Clone)]
pub struct Options {
    height: BTreeSet<usize>,
    height_str: BTreeSet<String>,
    width: BTreeSet<usize>,
    width_str: BTreeSet<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .collect()
        .catch();

    let height_str = long("height").argument::<String>("PX").collect().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .collect();

    let width_str = long("width").argument::<String>("PX").collect().hide();

    construct!(Options {
        height,
        height_str,
        width,
        width_str
    })
    .to_options()
}
````

Despite parser producing a funky value - help looks like you would expect from a parser that
takes two values



```text
$ app --help
Usage: app --height=PX... --width=PX...

Available options:
        --height=PX  Height of a rectangle
        --width=PX   Width of a rectangle
    -h, --help       Prints help information
```


When executed with no parameters it produces four empty values - all parsers succeed by the
nature of them being [`collect`](Parser::collect)



```text
$ app 
Options { height: {}, height_str: {}, width: {}, width_str: {} }
```


When executed with expected parameters fields with `usize` get their values



```text
$ app --height 100 --width 100 --height 12 --width 44
Options { height: {12, 100}, height_str: {}, width: {44, 100}, width_str: {} }
```


With incorrect value for `--height` parameter inner part of `height` parser fails, `many`
combined with `catch` handles this failure and produces an empty set without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string



```text
$ app --height ten --height twenty
Options { height: {}, height_str: {"ten", "twenty"}, width: {}, width_str: {} }
```


In case of wrong `--width` - parser `width` fails, parser for `many` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser



```text
$ app --width ten
Error: couldn't parse `ten`: invalid digit found in string
```

