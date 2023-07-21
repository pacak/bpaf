<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    exec: Option<Vec<OsString>>,
    switch: bool,
}

fn exec() -> impl Parser<Option<Vec<OsString>>> {
    // this defines starting token - "--exec"
    let start = long("exec")
        .help("Spawn a process for each file found")
        .req_flag(());
    // this consumes everything that is not ";"
    let body = any("COMMAND", |s| (s != ";").then_some(s))
        .help("Command and arguments, {} will be replaced with a file name")
        .some("You need to pass some arguments to exec");
    // this defines endint goken - ";"
    let end = literal(";");
    // this consumes everything between starting token and ending token
    construct!(start, body, end)
        // this makes it so everything between those tokens is consumed
        .adjacent()
        // drop the surrounding tokens leaving just the arguments
        .map(|x| x.1)
        // and make it optional so that instead of an empty Vec
        // it is `None` when no `--exec` flags was passed.
        .optional()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s')
        .long("switch")
        .help("Regular top level switch")
        .switch();
    construct!(Options { exec(), switch }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external(execs))]
    exec: Option<Vec<OsString>>,
    #[bpaf(long, short)]
    /// Regular top level switch
    switch: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Exec {
    /// Spawn a process for each file found
    exec: (),

    #[bpaf(
        any("COMMAND", not_semi),
        some("Command and arguments, {} will be replaced with a file name")
    )]
    /// Command and arguments, {} will be replaced with a file name
    body: Vec<OsString>,

    #[bpaf(external(is_semi))]
    end: (),
}

fn not_semi(s: OsString) -> Option<OsString> {
    (s != ";").then_some(s)
}

fn is_semi() -> impl Parser<()> {
    // TODO - support literal in bpaf_derive
    literal(";")
}

// a different alternative would be to put a singular Exec
fn execs() -> impl Parser<Option<Vec<OsString>>> {
    exec().map(|e| e.body).optional()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Generated `--help` message is somewhat descriptive of the purpose


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--exec</b></tt> <tt><i>COMMAND</i></tt>... <tt><b>;</b></tt>] [<tt><b>-s</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><div style='padding-left: 0.5em'><tt><b>--exec</b></tt> <tt><i>COMMAND</i></tt>... <tt><b>;</b></tt></div><dt><tt><b>    --exec</b></tt></dt>
<dd>Spawn a process for each file found</dd>
<dt><tt><i>COMMAND</i></tt></dt>
<dd>Command and arguments, {} will be replaced with a file name</dd>
<p></p><dt><tt><b>-s</b></tt>, <tt><b>--switch</b></tt></dt>
<dd>Regular top level switch</dd>
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


You can have as many items between `--exec` and `;` as you want, they all will be captured
inside the exec vector. Extra options can go either before or after the block.


<div class='bpaf-doc'>
$ app --exec foo --bar ; -s<br>
Options { exec: Some(["foo", "--bar"]), switch: true }
</div>


This example uses [`some`](Parser::some) to make sure there are some parameters, but that's
optional.


<div class='bpaf-doc'>
$ app --exec ;<br>
<b>Error:</b> <b>--exec</b> is not expected in this context
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