
```no_run
use bpaf::*;

pub fn options() -> OptionParser<f64> {
    let miles = long("miles").help("Distance in miles").argument("MI");
    let km = long("kilo").help("Distance in kilometers").argument("KM");
    construct!([miles, km]).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help message describes all the parser combined

<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> (<tt><b>--miles</b></tt>=<tt><i>MI</i></tt> | <tt><b>--kilo</b></tt>=<tt><i>KM</i></tt>)</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --miles</b></tt>=<tt><i>MI</i></tt></dt>
<dd>Distance in miles</dd>
<dt><tt><b>    --kilo</b></tt>=<tt><i>KM</i></tt></dt>
<dd>Distance in kilometers</dd>
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


Users can pass value that satisfy either parser


<div class='bpaf-doc'>
$ app --miles 42<br>
42.0
</div>


<div class='bpaf-doc'>
$ app --kilo 15<br>
15.0
</div>


But not both at once or not at all:


<div class='bpaf-doc'>
$ app --miles 53 --kilo 10<br>
<b>Error:</b> <tt><b>--kilo</b></tt> cannot be used at the same time as <tt><b>--miles</b></tt>
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


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>--miles</b></tt>=<tt><i>MI</i></tt> or <tt><b>--kilo</b></tt>=<tt><i>KM</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


If those cases are valid you can handle them with `optional` and `many`
</details>