
```no_run
use bpaf::doc::*;
use bpaf::*;
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

fn generate_rectangle_help(meta: MetaInfo) -> Doc {
    let mut buf = Doc::default();
    buf.text("The app takes a rectangle defined by width and height\n\nYou can customize the screen size using ");
    buf.meta(meta, true);
    buf.text(" parameters");
    buf
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
    let rectangle =
        construct!(Rectangle { width, height }).with_group_help(generate_rectangle_help);

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

<details><summary>Output</summary>

`with_group_help` lets you write longer description for group of options that can also refer to
those options. Similar to [`group_help`](Parser::group_help) encased optios are separated from
the rest by a blank line.

Invoking help with a single `--help` flag renders shot(er) version of the help message
that contanis only the first paragraph for each block:


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>] [<tt><b>--width</b></tt>=<tt><i>W</i></tt>] [<tt><b>--height</b></tt>=<tt><i>H</i></tt>]</p><p><div>
<b>The app takes a rectangle defined by width and height</b><div style='padding-left: 0.5em'> You can customize the screen size using [<tt><b>--width</b></tt>=<tt><i>W</i></tt>] [<tt><b>--height</b></tt>=<tt><i>H</i></tt>] parameters</div></div><dl><dt><tt><b>    --width</b></tt>=<tt><i>W</i></tt></dt>
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


Invoking help with double `--help --help` flag renders the full help message with all the
descriptions added


<div class='bpaf-doc'>
$ app --help --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>] [<tt><b>--width</b></tt>=<tt><i>W</i></tt>] [<tt><b>--height</b></tt>=<tt><i>H</i></tt>]</p><p><div>
<b>The app takes a rectangle defined by width and height</b><div style='padding-left: 0.5em'> You can customize the screen size using [<tt><b>--width</b></tt>=<tt><i>W</i></tt>] [<tt><b>--height</b></tt>=<tt><i>H</i></tt>] parameters</div></div><dl><dt><tt><b>    --width</b></tt>=<tt><i>W</i></tt></dt>
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


Other than rendering the help message that there's no interactions with other parsers


<div class='bpaf-doc'>
$ app --width 120 --height 11<br>
Options { argument: 30, rectangle: Rectangle { width: 120, height: 11 } }
</div>



<div class='bpaf-doc'>
$ app --argument 12<br>
Options { argument: 12, rectangle: Rectangle { width: 10, height: 10 } }
</div>

</details>