Parsers in those examples show how to parse argument using two different parsers. If your
application expects two different types of input with the same name (numeric OR arbitrary
string literals) you should try to combine them into a single enum. This example parses them
separately for simplicity.

## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, argument("PX"), some("You must specify some heights"), catch)]
    /// Height of a rectangle
    height: Vec<usize>,

    #[bpaf(long("height"), argument("PX"), many, hide)]
    height_str: Vec<String>,

    #[bpaf(long, argument("PX"), some("You must specify some widths"))]
    /// Width of a rectangle
    width: Vec<usize>,

    #[bpaf(long("width"), argument("PX"), many, hide)]
    width_str: Vec<String>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    height: Vec<usize>,
    height_str: Vec<String>,
    width: Vec<usize>,
    width_str: Vec<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .some("You must specify some heights")
        .catch();

    let height_str = long("height").argument::<String>("PX").many().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .some("You must specify some widths");

    let width_str = long("width").argument::<String>("PX").many().hide();

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


When executed with no parameters parser fails because `some` requires you to specify at least
one matching parameter



```text
$ app 
Error: You must specify some heights
```


When executed with expected parameters fields with `usize` get their values



```text
$ app --height 100 --width 100 --height 12 --width 44
Options { height: [100, 12], height_str: [], width: [100, 44], width_str: [] }
```


With incorrect value for `--height` parameter inner part of `height` parser fails, `some`
combined with `catch` handles this failure and produces `[]` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string



```text
$ app --height 10 --height twenty --width 33
Options { height: [10], height_str: ["twenty"], width: [33], width_str: [] }
```


In case of wrong `--width` - parser `width` fails, parser for `some` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser



```text
$ app --height 10 --width 33 --width ten
Error: couldn't parse `ten`: invalid digit found in string
```

