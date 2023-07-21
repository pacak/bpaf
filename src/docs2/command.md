<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Cmd {
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone)]
pub struct Options {
    flag: bool,
    cmd: Cmd,
}

fn cmd() -> impl Parser<Cmd> {
    let flag = long("flag")
        .help("This flag is specific to command")
        .switch();
    let arg = long("arg").argument::<usize>("ARG");
    construct!(Cmd { flag, arg })
        .to_options()
        .descr("Command to do something")
        .command("cmd")
        // you can chain add extra short and long names
        .short('c')
}

pub fn options() -> OptionParser<Options> {
    let flag = long("flag")
        .help("This flag is specific to the outer layer")
        .switch();
    construct!(Options { flag, cmd() }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
// `command` annotation with no name gets the name from the object it is attached to,
// but you can override it using something like #[bpaf(command("my_command"))]
// you can chain more short and long names here to serve as aliases
#[bpaf(command("cmd"), short('c'))]
/// Command to do something
pub struct Cmd {
    /// This flag is specific to command
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// This flag is specific to the outer layer
    flag: bool,
    #[bpaf(external)]
    cmd: Cmd,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Commands show up on both outer level help


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--flag</b></tt>] <tt><i>COMMAND ...</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --flag</b></tt></dt>
<dd>This flag is specific to the outer layer</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p><p><div>
<b>Available commands:</b></div><dl><dt><tt><b>cmd</b></tt>, <tt><b>c</b></tt></dt>
<dd>Command to do something</dd>
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


As well as showing their own help


<div class='bpaf-doc'>
$ app cmd --help<br>
<p>Command to do something</p><p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>cmd</b></tt> [<tt><b>--flag</b></tt>] <tt><b>--arg</b></tt>=<tt><i>ARG</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --flag</b></tt></dt>
<dd>This flag is specific to command</dd>
<dt><tt><b>    --arg</b></tt>=<tt><i>ARG</i></tt></dt>
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


In this example there's only one command and it is required, so is the argument inside of it


<div class='bpaf-doc'>
$ app cmd --arg 42<br>
Options { flag: false, cmd: Cmd { flag: false, arg: 42 } }
</div>


If you don't specify this command - parsing will fail

You can have the same flag names inside and outside of the command, but it might be confusing
for the end user. This example enables the outer flag


<div class='bpaf-doc'>
$ app --flag cmd --arg 42<br>
Options { flag: true, cmd: Cmd { flag: false, arg: 42 } }
</div>



And this one - both inside and outside


<div class='bpaf-doc'>
$ app --flag cmd --arg 42 --flag<br>
Options { flag: true, cmd: Cmd { flag: true, arg: 42 } }
</div>


And that's the confusing part - unless you add context restrictions with
[`adjacent`](crate::ParseCon::adjacent) and parse command first - outer flag wins.
So it's best not to mix names on different levels


<div class='bpaf-doc'>
$ app cmd --arg 42 --flag<br>
Options { flag: true, cmd: Cmd { flag: false, arg: 42 } }
</div>

</details>