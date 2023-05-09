```no_run
//! This is not a typical bpaf usage, but you should be able to replicate command line used by find

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
    let value = positional("USER");
    construct!(tag, value)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

// parsers -exec xxx yyy zzz ;
fn exec() -> impl Parser<Option<Vec<OsString>>> {
    let tag = literal("-exec")
        .anywhere()
        .help("-exec /path/to/command flags and options ;");

    let item = any::<OsString, _, _>("ITEM", |s| (s != ";").then_some(s))
        .many()
        .hide();
    let endtag = any::<String, _, _>("END", |s| (s == ";").then_some(())).hide();
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

    let tag = any::<String, _, _>("-mode", |s| (s == "-mode").then_some(())).anywhere();

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
<details>
<summary style="display: list-item;">Examples</summary>


Usually `find` takes a path where to look, the rest is optional
```console
% app src tests
Options { paths: ["src", "tests"], exec: None, user: None, perm: None }
```

In addition to paths `find` can take some more options, typically unusual: username, note a
single dash with a long name:
```console
% app -user bob
Options { paths: [], exec: None, user: Some("bob"), perm: None }
```

Permissions, in an unusual format:
```console
% app -mode /x
Options { paths: [], exec: None, user: None, perm: Some(Any(Perms { read: false, write: false, exec: true })) }
```

And the most interesting one is `-exec` which takes multiple arbitrary parameters terminated
by `;` (in shell you have to escape it as `\\;`)
```console
% app -exec cat -A '{}' \;
Options { paths: [], exec: Some(["cat", "-A", "{}"]), user: None, perm: None }
```

As usuall you can mix them and order doesn't matter
```console
% app src -mode -r -user bob -exec rustc '{}' \;
Options { paths: ["src"], exec: Some(["rustc", "{}"]), user: Some("bob"), perm: Some(All(Perms { read: true, write: false, exec: false })) }
```

While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.
```console
% app --help
Usage: [-exec] [-user <USER>] [-mode <MODE>] [<PATH>]...

Available options:
  -exec
    -exec       -exec /path/to/command flags and options ;

  -user <USER>

  -mode <MODE>
    <MODE>      (perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw

    -h, --help  Prints help information
```

</details>
