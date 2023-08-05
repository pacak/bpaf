<details><summary><tt>examples/xorg.rs</tt></summary>

```no_run
/// A way to represent xorg like flags, not a typical usage
use bpaf::*;
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
    extensions: Vec<(String, bool)>,
}

// matches literal name prefixed with - or +
fn toggle_options(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<bool> {
    any(meta, move |s: String| {
        if let Some(suf) = s.strip_prefix('+') {
            (suf == name).then_some(true)
        } else if let Some(suf) = s.strip_prefix('-') {
            (suf == name).then_some(false)
        } else {
            None
        }
    })
    .help(help)
    .anywhere()
}

// matches literal +ext and -ext followed by extension name
fn extension() -> impl Parser<(String, bool)> {
    let state = any("(+|-)ext", |s: String| match s.as_str() {
        "-ext" => Some(false),
        "+ext" => Some(true),
        _ => None,
    })
    .anywhere();

    let name = positional::<String>("EXT")
        .help("Extension to enable or disable, see documentation for the full list");
    construct!(state, name).adjacent().map(|(a, b)| (b, a))
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_options("(+|-)backing", "backing", "Set backing status").fallback(false);
    let xinerama =
        toggle_options("(+|-)xinerama", "xinerama", "Set Xinerama status").fallback(true);
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let extensions = extension().many();
    construct!(Options {
        turbo,
        backing,
        xinerama,
        extensions,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}

```

</details>

<details><summary>Output</summary>

`xorg` takes parameters in a few different ways, notably as a long name starting with plus or
minus with different defaults


<div class='bpaf-doc'>
$ app -xinerama +backing<br>
Options { turbo: false, backing: true, xinerama: false, extensions: [] }
</div>


But also as `+ext name` and `-ext name` to enable or disable an extensions


<div class='bpaf-doc'>
$ app --turbo +ext banana -ext apple<br>
Options { turbo: true, backing: false, xinerama: true, extensions: [("banana", true), ("apple", false)] }
</div>


While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-t</b></tt>] [<tt><i>(+|-)backing</i></tt>] [<tt><i>(+|-)xinerama</i></tt>] [<tt><i>(+|-)ext</i></tt> <tt><i>EXT</i></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-t</b></tt>, <tt><b>--turbo</b></tt></dt>
<dd>Engage the turbo mode</dd>
<dt><tt><i>(+|-)backing</i></tt></dt>
<dd>Set backing status</dd>
<dt><tt><i>(+|-)xinerama</i></tt></dt>
<dd>Set Xinerama status</dd>
<div style='padding-left: 0.5em'><tt><i>(+|-)ext</i></tt> <tt><i>EXT</i></tt></div><dt><tt><i>EXT</i></tt></dt>
<dd>Extension to enable or disable, see documentation for the full list</dd>
<p></p><dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
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