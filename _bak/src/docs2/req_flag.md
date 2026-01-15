<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub enum Style {
    Intel,
    Att,
    Llvm,
}

#[derive(Debug, Clone)]
pub enum Report {
    /// Include defailed report
    Detailed,
    /// Include minimal report
    Minimal,
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone)]
pub struct Options {
    agree: (),
    style: Style,
    report: Report,
}

pub fn options() -> OptionParser<Options> {
    let agree = long("agree")
        .help("You must agree to perform the action")
        .req_flag(());

    let intel = long("intel")
        .help("Show assembly using Intel style")
        .req_flag(Style::Intel);
    let att = long("att")
        .help("Show assembly using AT&T style")
        .req_flag(Style::Att);
    let llvm = long("llvm").help("Show llvm-ir").req_flag(Style::Llvm);
    let style = construct!([intel, att, llvm]);

    let detailed = long("detailed")
        .help("Include detailed report")
        .req_flag(Report::Detailed);
    let minimal = long("minimal")
        .help("Include minimal report")
        .req_flag(Report::Minimal);
    let report = construct!([detailed, minimal]).fallback(Report::Undecided);

    construct!(Options {
        agree,
        style,
        report
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
pub enum Style {
    /// Show assembly using Intel style
    Intel,
    /// Show assembly using AT&T style
    Att,
    /// Show llvm-ir
    Llvm,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(fallback(Report::Undecided))]
pub enum Report {
    /// Include detailed report
    Detailed,
    /// Include minimal report
    Minimal,
    #[bpaf(skip)]
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// You must agree to perform the action
    agree: (),
    // external here uses explicit reference to function `style`
    // generated above
    #[bpaf(external(style))]
    style: Style,
    // here reference is implicit and derived from field name: `report`
    #[bpaf(external)]
    report: Report,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In `--help` message `req_flag` look similarly to [`switch`](NamedArg::switch) and
[`flag`](NamedArg::flag)


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--agree</b></tt> (<tt><b>--intel</b></tt> | <tt><b>--att</b></tt> | <tt><b>--llvm</b></tt>) [<tt><b>--detailed</b></tt> | <tt><b>--minimal</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --agree</b></tt></dt>
<dd>You must agree to perform the action</dd>
<dt><tt><b>    --intel</b></tt></dt>
<dd>Show assembly using Intel style</dd>
<dt><tt><b>    --att</b></tt></dt>
<dd>Show assembly using AT&T style</dd>
<dt><tt><b>    --llvm</b></tt></dt>
<dd>Show llvm-ir</dd>
<dt><tt><b>    --detailed</b></tt></dt>
<dd>Include detailed report</dd>
<dt><tt><b>    --minimal</b></tt></dt>
<dd>Include minimal report</dd>
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


Example contains two parsers that fails without any input: `agree` requires passing `--agree`


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>--agree</b></tt>, pass <tt><b>--help</b></tt> for usage information
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


While `style` takes one of several possible values


<div class='bpaf-doc'>
$ app --agree<br>
<b>Error:</b> expected <tt><b>--intel</b></tt>, <tt><b>--att</b></tt>, or more, pass <tt><b>--help</b></tt> for usage information
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


It is possible to alter the behavior using [`fallback`](Parser::fallback) or
[`hide`](Parser::hide).


<div class='bpaf-doc'>
$ app --agree --intel<br>
Options { agree: (), style: Intel, report: Undecided }
</div>


While parser for `style` takes any posted output - it won't take multiple of them at once
(unless other combinators such as [`many`](Parser::many) permit it) or [`last`](Parser::last).


<div class='bpaf-doc'>
$ app --agree --att --llvm<br>
<b>Error:</b> <tt><b>--llvm</b></tt> cannot be used at the same time as <tt><b>--att</b></tt>
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