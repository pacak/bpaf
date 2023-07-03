<details><summary><tt>examples/derive_show_asm.rs</tt></summary>

```no_run
//! Parsing snippet from cargo-show-asm
//! Derive + typed fallback + external both with and without name

use bpaf::{construct, long, Bpaf, Parser, ShellComp};
use std::{convert::Infallible, path::PathBuf};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options("asm"))] // derives cargo helper for cargo-asm
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    #[bpaf(external(parse_manifest_path))]
    pub manifest_path: PathBuf,
    /// Custom target directory for generated artifacts
    #[bpaf(argument("DIR"))]
    pub target_dir: Option<PathBuf>,
    /// Package to use if ambigous
    #[bpaf(long, short, argument("SPEC"))]
    pub package: Option<String>,
    #[bpaf(external, optional)]
    pub focus: Option<Focus>,
    /// Produce a build plan instead of actually building
    pub dry: bool,
    /// Requires Cargo.lock and cache are up to date
    pub frozen: bool,
    /// Requires Cargo.lock is up to date
    pub locked: bool,
    /// Run without accessing the network
    pub offline: bool,
    #[bpaf(external)]
    pub format: Format,
    #[bpaf(external, fallback(Syntax::Intel))]
    pub syntax: Syntax,
    #[bpaf(external)]
    pub selected_function: SelectedFunction,
}

#[derive(Debug, Clone, Bpaf)]
/// Item to pick from the output
pub struct SelectedFunction {
    /// Complete or partial function name to filter
    #[bpaf(positional("FUNCTION"))]
    pub function: Option<String>,
    /// Select nth item from a filtered list
    #[bpaf(positional("INDEX"), fallback(0))]
    pub nth: usize,
}

fn parse_manifest_path() -> impl Parser<PathBuf> {
    long("manifest-path")
        .help("Path to Cargo.toml")
        .argument::<PathBuf>("PATH")
        .complete_shell(ShellComp::File {
            mask: Some("*.toml"),
        })
        .parse(|p| {
            // cargo-metadata wants to see
            if p.is_absolute() {
                Ok(p)
            } else {
                std::env::current_dir()
                    .map(|d| d.join(p))
                    .and_then(|full_path| full_path.canonicalize())
            }
        })
        .fallback_with(|| std::env::current_dir().map(|x| x.join("Cargo.toml")))
}

#[derive(Debug, Clone, Bpaf)]
/// How to render output
pub struct Format {
    /// Print interleaved Rust code
    pub rust: bool,

    #[bpaf(external(color_detection))]
    pub color: bool,

    /// include full demangled name instead of just prefix
    pub full_name: bool,
}

#[derive(Debug, Clone, Bpaf)]
/// Pick output type
///
/// included help
///
///
/// Extended help
pub enum Syntax {
    /// Generate assembly using Intel style
    Intel,
    /// Generate assembly using AT&T style
    Att,
}

fn color_detection() -> impl Parser<bool> {
    let yes = long("color")
        .help("Enable color highlighting")
        .req_flag(true);
    let no = long("no-color")
        .help("Disable color highlighting")
        .req_flag(false);
    construct!([yes, no]).fallback_with::<_, Infallible>(|| {
        // we can call for supports-color crate here
        Ok(true)
    })
}

fn comp_examples(prefix: &String) -> Vec<(String, Option<String>)> {
    // in the actual app we can ask cargo-metadata for this info
    let examples = ["derive_show_asm", "coreutils", "comonad"];
    examples
        .iter()
        .filter_map(|e| {
            if e.starts_with(prefix) {
                Some((e.to_string(), None))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone, Bpaf)]
/// Select artifact to use for analysis
///
/// Only one is valid
pub enum Focus {
    /// Show results from library code
    Lib,

    Test(
        /// Show results from a test
        #[bpaf(long("test"), argument("TEST"))]
        String,
    ),

    Bench(
        /// Show results from a benchmark
        #[bpaf(long("bench"), argument("BENCH"))]
        String,
    ),

    Example(
        /// Show results from an example
        #[bpaf(long("example"), argument("EXAMPLE"), complete(comp_examples))]
        String,
    ),

    Bin(
        /// Show results from a binary
        #[bpaf(long("bin"), argument("BIN"))]
        String,
    ),
}

fn main() {
    println!("{:#?}", options().run());
}

```

</details>

<details><summary>Output</summary>

Example defines this parser


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--manifest-path</b></tt>=<tt><i>PATH</i></tt>] [<tt><b>--target-dir</b></tt>=<tt><i>DIR</i></tt>] [<tt><b>-p</b></tt>=<tt><i>SPEC</i></tt>] [<tt><b>--lib</b></tt> | <tt><b>--test</b></tt>=<tt><i>TEST</i></tt> | <tt><b>--bench</b></tt>=<tt><i>BENCH</i></tt> | <tt><b>--example</b></tt>=<tt><i>EXAMPLE</i></tt> | <tt><b>--bin</b></tt>=<tt><i>BIN</i></tt>] [<tt><b>--dry</b></tt>] [<tt><b>--frozen</b></tt>] [<tt><b>--locked</b></tt>] [<tt><b>--offline</b></tt>] [<tt><b>--rust</b></tt>] [<tt><b>--color</b></tt> | <tt><b>--no-color</b></tt>] [<tt><b>--full-name</b></tt>] [<tt><b>--intel</b></tt> | <tt><b>--att</b></tt>] [<tt><i>FUNCTION</i></tt>] [<tt><i>INDEX</i></tt>]</p><p><div>
<b>Select artifact to use for analysis</b><div style='padding-left: 0.5em'> Only one is valid</div></div><dl><dt><tt><b>    --lib</b></tt></dt>
<dd>Show results from library code</dd>
<dt><tt><b>    --test</b></tt>=<tt><i>TEST</i></tt></dt>
<dd>Show results from a test</dd>
<dt><tt><b>    --bench</b></tt>=<tt><i>BENCH</i></tt></dt>
<dd>Show results from a benchmark</dd>
<dt><tt><b>    --example</b></tt>=<tt><i>EXAMPLE</i></tt></dt>
<dd>Show results from an example</dd>
<dt><tt><b>    --bin</b></tt>=<tt><i>BIN</i></tt></dt>
<dd>Show results from a binary</dd>
</dl>
</p><p><div>
<b>How to render output</b></div><dl><dt><tt><b>    --rust</b></tt></dt>
<dd>Print interleaved Rust code</dd>
<dt><tt><b>    --color</b></tt></dt>
<dd>Enable color highlighting</dd>
<dt><tt><b>    --no-color</b></tt></dt>
<dd>Disable color highlighting</dd>
<dt><tt><b>    --full-name</b></tt></dt>
<dd>include full demangled name instead of just prefix</dd>
</dl>
</p><p><div>
<b>Pick output type</b><div style='padding-left: 0.5em'> included help</div></div><dl><dt><tt><b>    --intel</b></tt></dt>
<dd>Generate assembly using Intel style</dd>
<dt><tt><b>    --att</b></tt></dt>
<dd>Generate assembly using AT&T style</dd>
</dl>
</p><p><div>
<b>Item to pick from the output</b></div><dl><dt><tt><i>FUNCTION</i></tt></dt>
<dd>Complete or partial function name to filter</dd>
<dt><tt><i>INDEX</i></tt></dt>
<dd>Select nth item from a filtered list</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --manifest-path</b></tt>=<tt><i>PATH</i></tt></dt>
<dd>Path to Cargo.toml</dd>
<dt><tt><b>    --target-dir</b></tt>=<tt><i>DIR</i></tt></dt>
<dd>Custom target directory for generated artifacts</dd>
<dt><tt><b>-p</b></tt>, <tt><b>--package</b></tt>=<tt><i>SPEC</i></tt></dt>
<dd>Package to use if ambigous</dd>
<dt><tt><b>    --dry</b></tt></dt>
<dd>Produce a build plan instead of actually building</dd>
<dt><tt><b>    --frozen</b></tt></dt>
<dd>Requires Cargo.lock and cache are up to date</dd>
<dt><tt><b>    --locked</b></tt></dt>
<dd>Requires Cargo.lock is up to date</dd>
<dt><tt><b>    --offline</b></tt></dt>
<dd>Run without accessing the network</dd>
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


By default completion system lists all possible cases


<pre>
% derive_show_asm \t
% derive_show_asm
--manifest-path=PATH     -- Path to Cargo.toml
--target-dir=DIR         -- Custom target directory for generated artifacts
--package=SPEC           -- Package to use if ambigous
--dry                    -- Produce a build plan instead of actually building
--frozen                 -- Requires Cargo.lock and cache are up to date
--locked                 -- Requires Cargo.lock is up to date
--offline                -- Run without accessing the network
Select artifact to use for analysis
--lib                    -- Show results from library code
--test=TEST              -- Show results from a test
--bench=BENCH            -- Show results from a benchmark
--example=EXAMPLE        -- Show results from an example
--bin=BIN                -- Show results from a binary
How to render output
--rust                   -- Print interleaved Rust code
--color                  -- Enable color highlighting
--no-color               -- Disable color highlighting
--full-name              -- include full demangled name instead of just prefix
Pick output type
--intel                  -- Generate assembly using Intel style
--att                    -- Generate assembly using AT&T style
Item to pick from the output
FUNCTION: Complete or partial function name to filter
</pre>


But when user tries to complete example name - it only lists examples produced by
`comp_examples` function


<pre>
% derive_show_asm --example \t
% derive_show_asm --example
Select artifact to use for analysis
EXAMPLE: Show results from an example
derive_show_asm
coreutils
comonad
</pre>


And completes the full name when user gives enough information


<pre>
% derive_show_asm --example cor\t
% derive_show_asm --example coreutils
</pre>

</details>