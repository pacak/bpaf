<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    switch: bool,
    arg: usize,
    username: String,
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s') // first `short` creates a builder
        .short('S') // second switch is a hidden alias
        .long("switch") // visible long name
        .long("also-switch") // hidden alias
        .help("Switch with many names")
        .switch(); // `switch` finalizes the builder

    let arg = long("argument") // long is also a builder
        .short('a')
        .short('A')
        .long("also-arg")
        .help("Argument with names")
        .argument::<usize>("ARG");

    let username = long("user")
        .short('u')
        .env("USER1")
        .help("Custom user name")
        .argument("USER");

    construct!(Options {
        switch,
        arg,
        username
    })
    .to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, short('S'), long("also-switch"))]
    /// Switch with many names
    switch: bool,
    #[bpaf(short, long("argument"), short('A'), long("also-arg"))]
    /// Argument with names
    arg: usize,
    #[bpaf(short, long("user"), env("USER1"), argument("USER"))]
    /// Custom user name
    username: String,
}
```

</details>
<details>
<summary>Examples</summary>


As usual switch is optional, arguments are required
```console
% app -a 42 -u Bobert
Options { switch: false, arg: 42, username: "Bobert" }
```

Help displays only visible aliases (and a current value for env arguments)
```console
% app --help
Usage: [-s] -a ARG -u USER

Available options:
    -s, --switch          Switch with many names
    -a, --argument <ARG>  Argument with names
    -u, --user <USER>     [env:USER1 = "pacak"]
                          Custom user name
    -h, --help            Prints help information
```

But you can still use hidden aliases, both short and long
```console
% app --also-switch --also-arg 330 --user Bobert
Options { switch: true, arg: 330, username: "Bobert" }
```

And unless there's `many` or similar modifiers having multiple aliases doesn't mean
you can specify them multiple times:
```console
% app -A 42 -a 330 -u Bobert
-a is not expected in this context
```

Also hidden aliases are really hidden and only meant to do backward compatibility stuff, they
won't show up anywhere else in completions or error messages
```console
% app -a 42 -A 330 -u Bobert
No such flag: `-A`, did you mean `-u`?
```

</details>
