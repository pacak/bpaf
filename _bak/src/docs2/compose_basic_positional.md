
```no_run
use bpaf::*;

pub fn options() -> OptionParser<String> {
    let simple = positional("URL").help("Url to open");
    simple.to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Same as with argument by default there's no fallback so with no arguments parser fails


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><i>URL</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


Other than that any name that does not start with a dash or explicitly converted to positional
parameter gets parsed:


<div class='bpaf-doc'>
$ app https://lemmyrs.org<br>
"https://lemmyrs.org"
</div>


<div class='bpaf-doc'>
$ app "strange url"<br>
"strange url"
</div>


<div class='bpaf-doc'>
$ app -- --can-start-with-dash-too<br>
"--can-start-with-dash-too"
</div>


And as usual there's help message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><i>URL</i></tt></p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>URL</i></tt></dt>
<dd>Url to open</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
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