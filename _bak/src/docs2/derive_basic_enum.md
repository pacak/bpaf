
```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    File {
        /// Read input from a file
        name: String,
    },

    Url {
        /// Read input from URL
        url: String,
        /// Authentication method to use for the URL
        auth_method: String,
    },
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help message reflects mutually exclusive parts


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> (<tt><b>--name</b></tt>=<tt><i>ARG</i></tt> | <tt><b>--url</b></tt>=<tt><i>ARG</i></tt> <tt><b>--auth-method</b></tt>=<tt><i>ARG</i></tt>)</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --name</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Read input from a file</dd>
<dt><tt><b>    --url</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Read input from URL</dd>
<dt><tt><b>    --auth-method</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Authentication method to use for the URL</dd>
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


At least one branch needs to succeed


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>--name</b></tt>=<tt><i>ARG</i></tt> or <tt><b>--url</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


And in this example only one branch can succeed


<div class='bpaf-doc'>
$ app --name Cargo.toml<br>
File { name: "Cargo.toml" }
</div>



<div class='bpaf-doc'>
$ app --url https://crates.io --auth-method digest<br>
Url { url: "https://crates.io", auth_method: "digest" }
</div>


While both branches can succeed at once - only one will actually succeed and afetr that
parsing fails since there are unconsumed items


<div class='bpaf-doc'>
$ app --url https://crates.io --auth-method digest --name Cargo.toml<br>
<b>Error:</b> <tt><b>--name</b></tt> cannot be used at the same time as <tt><b>--url</b></tt>
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