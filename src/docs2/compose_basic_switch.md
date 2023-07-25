
```no_run
use bpaf::*;

pub fn options() -> OptionParser<bool> {
    let simple = short('s').long("simple").switch();
    simple.to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

If you run the app with no parameters - switch will parse as `false`


<div class='bpaf-doc'>
$ app <br>
false
</div>


Both short and long names produce true


<div class='bpaf-doc'>
$ app -s<br>
true
</div>


<div class='bpaf-doc'>
$ app --simple<br>
true
</div>


In addition to the switch you defined `bpaf` generates switch for user help


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-s</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt>, <tt><b>--simple</b></tt></dt>
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