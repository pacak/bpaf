
```no_run
const DB: &str = "DATABASE_VAR";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Use verbose output
    // No name annotation and name is not a single character:
    // `bpaf` uses it as a long name - `--verbose`
    pub verbose: bool,

    /// Compile in a release mode
    #[bpaf(short)]
    // Name is long, but explicit annotation for a short name
    // `bpaf` makes a short name from the first symbol: `-r`
    pub release: bool,

    /// Number of parallel jobs, defaults to # of CPUs
    // Explicit annotation with a short name: `-j`
    #[bpaf(short('j'))]
    pub threads: Option<usize>,

    /// Upload artifacts to the storage
    // Explicit annotation for a single suppresses the oher one,
    // but you can specify both of them. `-u` and `--upload`
    #[bpaf(short, long)]
    pub upload: bool,

    /// List of features to activate
    // you can mix explicit annotations with and without names
    // when convenient, here it's `-F` and `--features`
    #[bpaf(short('F'), long)]
    pub features: Vec<String>,

    /// Read information from the database
    #[bpaf(env(DB))]
    // Annotation for `env` does not affect annotation for names
    // so `bpaf` makes `--database` flag too
    pub database: String,

    /// Only print essential information
    #[bpaf(short, long, long("essential"))]
    // `--essential` is a hidden ailias, `-q` and `--quiet` are visible
    pub quiet: bool,

    /// implicit long + env variable "USER"
    #[bpaf(env("USER"))]
    pub user: String,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

`--help` output will contain first short and first long names that are present and won't have
anything about hidden aliases.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--verbose</b></tt>] [<tt><b>-r</b></tt>] [<tt><b>-j</b></tt>=<tt><i>ARG</i></tt>] [<tt><b>-u</b></tt>] [<tt><b>-F</b></tt>=<tt><i>ARG</i></tt>]... <tt><b>--database</b></tt>=<tt><i>ARG</i></tt> [<tt><b>-q</b></tt>] <tt><b>--user</b></tt>=<tt><i>ARG</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --verbose</b></tt></dt>
<dd>Use verbose output</dd>
<dt><tt><b>-r</b></tt></dt>
<dd>Compile in a release mode</dd>
<dt><tt><b>-j</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Number of parallel jobs, defaults to # of CPUs</dd>
<dt><tt><b>-u</b></tt>, <tt><b>--upload</b></tt></dt>
<dd>Upload artifacts to the storage</dd>
<dt><tt><b>-F</b></tt>, <tt><b>--features</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>List of features to activate</dd>
<dt><tt><b>    --database</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Read information from the database</dd>
<dt></dt>
<dd>[env:DATABASE_VAR: N/A]</dd>
<dt><tt><b>-q</b></tt>, <tt><b>--quiet</b></tt></dt>
<dd>Only print essential information</dd>
<dt><tt><b>    --user</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>implicit long + env variable "USER"</dd>
<dt></dt>
<dd>[env:USER = "pacak"]</dd>
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


`--essential` is a hidden alias and still works despite not being present in `--help` output
above


<div class='bpaf-doc'>
$ app --database default --essential<br>
Options { verbose: false, release: false, threads: None, upload: false, features: [], database: "default", quiet: true, user: "pacak" }
</div>


And hidden means actually hidden. While error message can suggest to fix a typo to make it a
valid _visible_ argument


<div class='bpaf-doc'>
$ app --database default --quie<br>
<b>Error:</b> no such flag: <b>--quie</b>, did you mean <tt><b>--quiet</b></tt>?
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


It will not do so for hidden aliases


<div class='bpaf-doc'>
$ app --database default --essentia<br>
<b>Error:</b> <b>--essentia</b> is not expected in this context
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