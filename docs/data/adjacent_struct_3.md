## Derive example

```rust,id:1
# use std::ffi::OsString;
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external(execs))]
    exec: Option<Vec<OsString>>,
    #[bpaf(long, short)]
    /// Regular top level switch
    switch: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Exec {
    /// Spawn a process for each file found
    exec: (),

    #[bpaf(
        any("COMMAND", not_semi),
        some("Command and arguments, {} will be replaced with a file name")
    )]
    /// Command and arguments, {} will be replaced with a file name
    body: Vec<OsString>,

    #[bpaf(external(is_semi))]
    end: (),
}

fn not_semi(s: OsString) -> Option<OsString> {
    (s != ";").then_some(s)
}

fn is_semi() -> impl Parser<()> {
    literal(";", ())
}

// a different alternative would be to put a singular Exec
fn execs() -> impl Parser<Option<Vec<OsString>>> {
    exec().map(|e| e.body).optional()
}
```

## Combinatoric example

```rust,id:2
# use std::ffi::OsString;
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    exec: Option<Vec<OsString>>,
    switch: bool,
}

fn exec() -> impl Parser<Option<Vec<OsString>>> {
    // this defines starting token - "--exec"
    let start = long("exec")
        .help("Spawn a process for each file found")
        .req_flag(());
    // this consumes everything that is not ";"
    let body = any("COMMAND", |s| (s != ";").then_some(s))
        .help("Command and arguments, {} will be replaced with a file name")
        .some("You need to pass some arguments to exec");
    // this defines endint goken - ";"
    let end = literal(";", ());
    // this consumes everything between starting token and ending token
    construct!(start, body, end)
        // this makes it so everything between those tokens is consumed
        .adjacent()
        // drop the surrounding tokens leaving just the arguments
        .map(|x| x.1)
        // and make it optional so that instead of an empty Vec
        // it is `None` when no `--exec` flags was passed.
        .optional()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s')
        .long("switch")
        .help("Regular top level switch")
        .switch();
    construct!(Options { exec(), switch }).to_options()
}
```

Generated `--help` message is somewhat descriptive of the purpose

```run,id:1,id:2
--help
```

You can have as many items between `--exec` and `;` as you want, they all will be captured
inside the exec vector. Extra options can go either before or after the block.

```run,id:1,id:2
--exec foo --bar ; -s
```

This example uses [`some`](Parser::some) to make sure there are some parameters, but that's
optional.

```run,id:1,id:2
--exec ;
```
