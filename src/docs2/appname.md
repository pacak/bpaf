<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    user: String,
    appname: String,
}

pub fn options() -> OptionParser<Options> {
    let user = short('u')
        .long("user")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    construct!(Options { user, appname() }).to_options()
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
    /// Specify user name
    #[bpaf(short, long, argument::<String>("NAME"))]
    user: String,

    /// Specify user age
    #[bpaf(external)]
    appname: String,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Parsed application name doesn't show up in the `--help` output


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-u</b></tt>=<tt><i>NAME</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-u</b></tt>, <tt><b>--user</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Specify user name</dd>
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


but simply produces the app name, when it is available


<div class='bpaf-doc'>
$ app --user=Bob<br>
Options { user: "Bob", appname: "app" }
</div>

</details>