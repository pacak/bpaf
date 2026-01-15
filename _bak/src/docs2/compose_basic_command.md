
```no_run
#[derive(Debug, Clone)]
pub enum Options {
    /// Run a binary
    Run {
        /// Name of a binary to run
        bin: String,

        /// Arguments to pass to a binary
        args: Vec<String>,
    },
    /// Compile a binary
    Build {
        /// Name of a binary to build
        bin: String,

        /// Compile the binary in release mode
        release: bool,
    },
}

// combine mode gives more flexibility to share the same code across multiple parsers
fn run() -> impl Parser<Options> {
    let bin = long("bin").help("Name of a binary to run").argument("BIN");
    let args = positional("ARG")
        .strict()
        .help("Arguments to pass to a binary")
        .many();

    construct!(Options::Run { bin, args })
}

pub fn options() -> OptionParser<Options> {
    let run = run().to_options().descr("Run a binary").command("run");

    let bin = long("bin")
        .help("Name of a binary to build ")
        .argument("BIN");
    let release = long("release")
        .help("Compile the binary in release mode")
        .switch();
    let build = construct!(Options::Build { bin, release })
        .to_options()
        .descr("Compile a binary")
        .command("build");

    construct!([run, build]).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help contains both commands, bpaf takes short command description from the inner command
description


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><i>COMMAND ...</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p><p><div>
<b>Available commands:</b></div><dl><dt><tt><b>run</b></tt></dt>
<dd>Run a binary</dd>
<dt><tt><b>build</b></tt></dt>
<dd>Compile a binary</dd>
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


Same as before each command gets its own help message


<div class='bpaf-doc'>
$ app run --help<br>
<p>Run a binary</p><p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>run</b></tt> <tt><b>--bin</b></tt>=<tt><i>BIN</i></tt> <tt><b>--</b></tt> [<tt><i>ARG</i></tt>]...</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>ARG</i></tt></dt>
<dd>Arguments to pass to a binary</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --bin</b></tt>=<tt><i>BIN</i></tt></dt>
<dd>Name of a binary to run</dd>
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


And can be executed separately


<div class='bpaf-doc'>
$ app run --bin basic<br>
Run { bin: "basic", args: [] }
</div>


<div class='bpaf-doc'>
$ app build --bin demo --release<br>
Build { bin: "demo", release: true }
</div>

</details>