
```no_run
use bpaf::*;

pub fn options() -> OptionParser<usize> {
    short('s')
        .long("size")
        .help("Defines size of an object")
        .argument::<usize>("SIZE")
        .to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

By default all arguments are required so running with no arguments produces an error


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>--size</b></tt>=<tt><i>SIZE</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


Bpaf accepts various combinations of names and adjacencies:


<div class='bpaf-doc'>
$ app -s100<br>
100
</div>


<div class='bpaf-doc'>
$ app --size 300<br>
300
</div>


<div class='bpaf-doc'>
$ app -s=42<br>
42
</div>


<div class='bpaf-doc'>
$ app --size=14<br>
14
</div>


Since not every string is a valid number - bpaf would report any parsing failures to the user
directly


<div class='bpaf-doc'>
$ app --size fifty<br>
<b>Error:</b> couldn't parse <b>fifty</b>: invalid digit found in string
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


In addition to the switch you defined `bpaf` generates switch for user help which will include
the description from the `help` method


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-s</b></tt>=<tt><i>SIZE</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt>, <tt><b>--size</b></tt>=<tt><i>SIZE</i></tt></dt>
<dd>Defines size of an object</dd>
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

</details>