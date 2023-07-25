
```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command("run"))]
    /// Run a binary
    Run {
        /// Name of a binary crate
        name: String,
    },

    /// Run a self test
    #[bpaf(command)]
    Test,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help message lists subcommand

<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><i>COMMAND ...</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p><p><div>
<b>Available commands:</b></div><dl><dt><tt><b>run</b></tt></dt>
<dd>Run a binary</dd>
<dt><tt><b>test</b></tt></dt>
<dd>Run a self test</dd>
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


Commands have their own arguments


<div class='bpaf-doc'>
$ app run --name Bob<br>
Run { name: "Bob" }
</div>



<div class='bpaf-doc'>
$ app test<br>
Test
</div>



<div class='bpaf-doc'>
$ app test --name bob<br>
<b>Error:</b> <b>--name</b> is not expected in this context
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