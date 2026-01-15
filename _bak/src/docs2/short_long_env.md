<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    arg: usize,
    username: String,
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s') // first `short` creates a builder
        .short('S') // second switch is a hidden alias
        .long("switch") // visible long name
        .long("also-switch") // hidden alias
        .help("Switch with many names")
        .switch(); // `switch` finalizes the builder

    let arg = long("argument") // long is also a builder
        .short('a')
        .short('A')
        .long("also-arg")
        .help("Argument with names")
        .argument::<usize>("ARG");

    let username = long("user")
        .short('u')
        .env("USER1")
        .help("Custom user name")
        .argument::<String>("USER");

    construct!(Options {
        switch,
        arg,
        username
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
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, short('S'), long("also-switch"))]
    /// Switch with many names
    switch: bool,
    #[bpaf(short, long("argument"), short('A'), long("also-arg"))]
    /// Argument with names
    arg: usize,
    #[bpaf(short, long("user"), env("USER1"), argument("USER"))]
    /// Custom user name
    username: String,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

As usual switch is optional, arguments are required


<div class='bpaf-doc'>
$ app -a 42 -u Bobert<br>
Options { switch: false, arg: 42, username: "Bobert" }
</div>



Help displays only visible aliases (and a current value for env arguments)


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-s</b></tt>] <tt><b>-a</b></tt>=<tt><i>ARG</i></tt> <tt><b>-u</b></tt>=<tt><i>USER</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt>, <tt><b>--switch</b></tt></dt>
<dd>Switch with many names</dd>
<dt><tt><b>-a</b></tt>, <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Argument with names</dd>
<dt><tt><b>-u</b></tt>, <tt><b>--user</b></tt>=<tt><i>USER</i></tt></dt>
<dd>Custom user name</dd>
<dt></dt>
<dd>[env:USER1: N/A]</dd>
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


But you can still use hidden aliases, both short and long


<div class='bpaf-doc'>
$ app --also-switch --also-arg 330 --user Bobert<br>
Options { switch: true, arg: 330, username: "Bobert" }
</div>


And unless there's `many` or similar modifiers having multiple aliases doesn't mean
you can specify them multiple times:


<div class='bpaf-doc'>
$ app -A 42 -a 330 -u Bobert<br>
<b>Error:</b> <b>-a</b> is not expected in this context
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


Also hidden aliases are really hidden and only meant to do backward compatibility stuff, they
won't show up anywhere else in completions or error messages


<div class='bpaf-doc'>
$ app -a 42 -A 330 -u Bobert<br>
<b>Error:</b> <b>-A</b> is not expected in this context
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