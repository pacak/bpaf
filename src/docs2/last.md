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
    style: Style,
    report: Report,
}

pub fn options() -> OptionParser<Options> {
    let intel = long("intel")
        .help("Show assembly using Intel style")
        .req_flag(Style::Intel);
    let att = long("att")
        .help("Show assembly using AT&T style")
        .req_flag(Style::Att);
    let llvm = long("llvm").help("Show llvm-ir").req_flag(Style::Llvm);
    let style = construct!([intel, att, llvm]).last();

    let detailed = long("detailed")
        .help("Include detailed report")
        .req_flag(Report::Detailed);
    let minimal = long("minimal")
        .help("Include minimal report")
        .req_flag(Report::Minimal);
    let report = construct!([detailed, minimal])
        .last()
        .fallback(Report::Undecided);

    construct!(Options { style, report }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(last)]
pub enum Style {
    /// Show assembly using Intel style
    Intel,
    /// Show assembly using AT&T style
    Att,
    /// Show llvm-ir
    Llvm,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(last, fallback(Report::Undecided))]
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

In `--help` message `last` shows that inner parser can run multiple times


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> (<tt><b>--intel</b></tt> | <tt><b>--att</b></tt> | <tt><b>--llvm</b></tt>)... [<tt><b>--detailed</b></tt> | <tt><b>--minimal</b></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --intel</b></tt></dt>
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



`style` takes one of several possible values and `last` lets user to pass it several times


<div class='bpaf-doc'>
$ app --intel<br>
Options { style: Intel, report: Undecided }
</div>


<div class='bpaf-doc'>
$ app --intel --att<br>
Options { style: Att, report: Undecided }
</div>


<div class='bpaf-doc'>
$ app --intel --att --intel<br>
Options { style: Intel, report: Undecided }
</div>


same goes with `report`


<div class='bpaf-doc'>
$ app --intel --detailed<br>
Options { style: Intel, report: Detailed }
</div>


<div class='bpaf-doc'>
$ app --att --detailed --minimal<br>
Options { style: Att, report: Minimal }
</div>

</details>