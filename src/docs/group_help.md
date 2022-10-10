<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Rectangle {
    width: u32,
    height: u32,
}

# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
    rectangle: Rectangle,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .fallback(30);

    let width = long("width")
        .help("Width of the rectangle")
        .argument("W")
        .fallback(10);
    let height = long("height")
        .help("Height of the rectangle")
        .argument("H")
        .fallback(10);
    let rectangle = construct!(Rectangle { width, height }).group_help("takes a rectangle");

    construct!(Options {
        argument,
        rectangle
    })
    .to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
pub struct Rectangle {
    /// Width of the rectangle
    #[bpaf(argument("W"), fallback(10))]
    width: u32,
    /// Height of the rectangle
    #[bpaf(argument("H"), fallback(10))]
    height: u32,
}
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// secret switch
    #[bpaf(external, group_help("takes a rectangle"))]
    rectangle: Rectangle,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


`group_help` doesn't change the parsing behavior in any way
```console
% app --argument 32 --width 20 --height 13
Options { argument: 32, rectangle: Rectangle { width: 20, height: 13 } }
```

Instead it adds extra decoration for the inner group in --help message
```console
% app --help
Usage: [--argument ARG] [--width W] [--height H]

Available options:
        --argument <ARG>  important argument
  takes a rectangle
        --width <W>       Width of the rectangle
        --height <H>      Height of the rectangle

    -h, --help            Prints help information
```

</details>
