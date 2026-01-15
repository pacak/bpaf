<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
}

fn toggle_option(name: &'static str, help: &'static str) -> impl Parser<bool> {
    // parse +name and -name into a bool
    any::<String, _, _>(name, move |s: String| {
        if let Some(rest) = s.strip_prefix('+') {
            (rest == name).then_some(true)
        } else if let Some(rest) = s.strip_prefix('-') {
            (rest == name).then_some(false)
        } else {
            None
        }
    })
    // set a custom usage and help metavariable
    .metavar(
        &[
            ("+", Style::Literal),
            (name, Style::Literal),
            (" | ", Style::Text),
            ("-", Style::Literal),
            (name, Style::Literal),
        ][..],
    )
    // set a custom help description
    .help(help)
    // apply this parser to all unconsumed items
    .anywhere()
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_option("backing", "Enable or disable backing")
        .fallback(false)
        .debug_fallback();
    let xinerama = toggle_option("xinerama", "enable or disable Xinerama")
        .fallback(true)
        .debug_fallback();
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    construct!(Options {
        turbo,
        backing,
        xinerama,
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
    /// Engage the turbo mode
    #[bpaf(short, long)]
    turbo: bool,
    #[bpaf(external(backing), fallback(false), debug_fallback)]
    backing: bool,
    #[bpaf(external(xinerama), fallback(true), debug_fallback)]
    xinerama: bool,
}

fn toggle_option(name: &'static str, help: &'static str) -> impl Parser<bool> {
    // parse +name and -name into a bool
    any::<String, _, _>(name, move |s: String| {
        if let Some(rest) = s.strip_prefix('+') {
            (rest == name).then_some(true)
        } else if let Some(rest) = s.strip_prefix('-') {
            (rest == name).then_some(false)
        } else {
            None
        }
    })
    // set a custom usage and help metavariable
    .metavar(
        &[
            ("+", Style::Literal),
            (name, Style::Literal),
            (" | ", Style::Text),
            ("-", Style::Literal),
            (name, Style::Literal),
        ][..],
    )
    // set a custom help description
    .help(help)
    // apply this parser to all unconsumed items
    .anywhere()
}

fn backing() -> impl Parser<bool> {
    toggle_option("backing", "Enable or disable backing")
}

fn xinerama() -> impl Parser<bool> {
    toggle_option("xinerama", "enable or disable Xinerama")
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`--help` message describes all the flags as expected


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-t</b></tt>] [<tt><b>+backing</b></tt> | <tt><b>-backing</b></tt>] [<tt><b>+xinerama</b></tt> | <tt><b>-xinerama</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-t</b></tt>, <tt><b>--turbo</b></tt></dt>
<dd>Engage the turbo mode</dd>
<dt><tt><b>+backing</b></tt> | <tt><b>-backing</b></tt></dt>
<dd>Enable or disable backing</dd>
<dt></dt>
<dd>[default: false]</dd>
<dt><tt><b>+xinerama</b></tt> | <tt><b>-xinerama</b></tt></dt>
<dd>enable or disable Xinerama</dd>
<dt></dt>
<dd>[default: true]</dd>
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


Parser obeys the defaults


<div class='bpaf-doc'>
$ app <br>
Options { turbo: false, backing: false, xinerama: true }
</div>


And can handle custom values


<div class='bpaf-doc'>
$ app --turbo -xinerama +backing<br>
Options { turbo: true, backing: true, xinerama: false }
</div>


`bpaf` won't be able to generate good error messages or suggest to fix typos to users since it
doesn't really knows what the function inside `any` is going to consume


<div class='bpaf-doc'>
$ app --turbo -xinerama +backin<br>
<b>Error:</b> <b>+backin</b> is not expected in this context
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