<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Rectangle {
    width: u32,
    height: u32,
}

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
    let rectangle = construct!(Rectangle { width, height }).group_help("Takes a rectangle");

    construct!(Options {
        argument,
        rectangle
    })
    .to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
pub struct Rectangle {
    /// Width of the rectangle
    #[bpaf(argument("W"), fallback(10))]
    width: u32,
    /// Height of the rectangle
    #[bpaf(argument("H"), fallback(10))]
    height: u32,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// secret switch
    #[bpaf(external, group_help("Takes a rectangle"))]
    rectangle: Rectangle,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`group_help` adds extra decoration for the inner group in `--help` message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>] [<tt><b>--width</b></tt>=<tt><i>W</i></tt>] [<tt><b>--height</b></tt>=<tt><i>H</i></tt>]</p><p><div>
<b>Takes a rectangle</b></div><dl><dt><tt><b>    --width</b></tt>=<tt><i>W</i></tt></dt>
<dd>Width of the rectangle</dd>
<dt><tt><b>    --height</b></tt>=<tt><i>H</i></tt></dt>
<dd>Height of the rectangle</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>important argument</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


And doesn't change the parsing behavior in any way


<div class='bpaf-doc'>
$ app --argument 32 --width 20 --height 13<br>
Options { argument: 32, rectangle: Rectangle { width: 20, height: 13 } }
</div>

</details>