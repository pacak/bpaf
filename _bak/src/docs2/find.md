<details><summary><tt>examples/find.rs</tt></summary>

```no_run
//! This is not a typical bpaf usage,
//! but you should be able to replicate command line used by find

use bpaf::*;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Perms {
    read: bool,
    write: bool,
    exec: bool,
}

#[derive(Debug, Clone)]
pub enum Perm {
    All(Perms),
    Any(Perms),
    Exact(Perms),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    paths: Vec<PathBuf>,
    exec: Option<Vec<OsString>>,
    user: Option<String>,
    perm: Option<Perm>,
}

// Parses -user xxx
fn user() -> impl Parser<Option<String>> {
    // match only literal "-user"
    let tag = literal("-user").anywhere();
    let value = positional("USER").help("User name");
    construct!(tag, value)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

// parsers -exec xxx yyy zzz ;
fn exec() -> impl Parser<Option<Vec<OsString>>> {
    let tag = literal("-exec")
        .help("for every file find finds execute a separate shell command")
        .anywhere();

    let item = any::<OsString, _, _>("ITEM", |s| (s != ";").then_some(s))
        .help("command with its arguments, find will replace {} with a file name")
        .many();

    let endtag = any::<String, _, _>(";", |s| (s == ";").then_some(()))
        .help("anything after literal \";\" will be considered a regular option again");

    construct!(tag, item, endtag)
        .adjacent()
        .map(|triple| triple.1)
        .optional()
}

/// parses symbolic permissions `-perm -mode`, `-perm /mode` and `-perm mode`
fn perm() -> impl Parser<Option<Perm>> {
    fn parse_mode(input: &str) -> Result<Perms, String> {
        let mut perms = Perms::default();
        for c in input.chars() {
            match c {
                'r' => perms.read = true,
                'w' => perms.write = true,
                'x' => perms.exec = true,
                _ => return Err(format!("{} is not a valid permission string", input)),
            }
        }
        Ok(perms)
    }

    let tag = literal("-mode").anywhere();

    // `any` here is used to parse an arbitrary string that can also start with dash (-)
    // regular positional parser won't work here
    let mode = any("MODE", Some)
        .help("(perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw")
        .parse::<_, _, String>(|s: String| {
            if let Some(m) = s.strip_prefix('-') {
                Ok(Perm::All(parse_mode(m)?))
            } else if let Some(m) = s.strip_prefix('/') {
                Ok(Perm::Any(parse_mode(m)?))
            } else {
                Ok(Perm::Exact(parse_mode(&s)?))
            }
        });

    construct!(tag, mode)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

pub fn options() -> OptionParser<Options> {
    let paths = positional::<PathBuf>("PATH").many();

    construct!(Options {
        exec(),
        user(),
        perm(),
        paths,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}

```

</details>

<details><summary>Output</summary>

Usually `find` takes a path where to look, the rest is optional


<div class='bpaf-doc'>
$ app src tests<br>
Options { paths: ["src", "tests"], exec: None, user: None, perm: None }
</div>


In addition to paths `find` can take some more options, typically unusual: username, note a
single dash with a long name:


<div class='bpaf-doc'>
$ app -user bob<br>
Options { paths: [], exec: None, user: Some("bob"), perm: None }
</div>



Permissions, in an unusual format:


<div class='bpaf-doc'>
$ app -mode /x<br>
Options { paths: [], exec: None, user: None, perm: Some(Any(Perms { read: false, write: false, exec: true })) }
</div>


And the most interesting one is `-exec` which takes multiple arbitrary parameters terminated
by `;` (in shell you have to escape it as `\\;`)


<div class='bpaf-doc'>
$ app -exec cat -A '{}' \;<br>
Options { paths: [], exec: Some(["cat", "-A", "{}"]), user: None, perm: None }
</div>


As usuall you can mix them and order doesn't matter


<div class='bpaf-doc'>
$ app src -mode -r -user bob -exec rustc '{}' \;<br>
Options { paths: ["src"], exec: Some(["rustc", "{}"]), user: Some("bob"), perm: Some(All(Perms { read: true, write: false, exec: false })) }
</div>


While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-exec</b></tt> [<tt><i>ITEM</i></tt>]... <tt><i>;</i></tt>] [<tt><b>-user</b></tt> <tt><i>USER</i></tt>] [<tt><b>-mode</b></tt> <tt><i>MODE</i></tt>] [<tt><i>PATH</i></tt>]...</p><p><div>
<b>Available options:</b></div><dl><div style='padding-left: 0.5em'><tt><b>-exec</b></tt> [<tt><i>ITEM</i></tt>]... <tt><i>;</i></tt></div><dt><tt><b>-exec</b></tt></dt>
<dd>for every file find finds execute a separate shell command</dd>
<dt><tt><i>ITEM</i></tt></dt>
<dd>command with its arguments, find will replace {} with a file name</dd>
<dt><tt><i>;</i></tt></dt>
<dd>anything after literal ";" will be considered a regular option again</dd>
<p></p><div style='padding-left: 0.5em'><tt><b>-user</b></tt> <tt><i>USER</i></tt></div><dt><tt><i>USER</i></tt></dt>
<dd>User name</dd>
<p></p><div style='padding-left: 0.5em'><tt><b>-mode</b></tt> <tt><i>MODE</i></tt></div><dt><tt><i>MODE</i></tt></dt>
<dd>(perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw</dd>
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