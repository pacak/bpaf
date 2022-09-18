<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cmd {
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    flag: bool,
    cmd: Cmd,
}

fn cmd() -> impl Parser<Cmd> {
    let flag = long("flag")
        .help("This flag is specific to command")
        .switch();
    let arg = long("arg").argument("ARG");
    construct!(Cmd { flag, arg })
        .to_options()
        .descr("Command to do something")
        .command("cmd")
        .help("Command to do something")
}

pub fn options() -> OptionParser<Options> {
    let flag = long("flag")
        .help("This flag is specific to the outer layer")
        .switch();
    construct!(Options { flag, cmd() }).to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(command)]
/// Command to do something
pub struct Cmd {
    /// This flag is specific to command
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// This flag is specific to the outer layer
    flag: bool,
    #[bpaf(external)]
    cmd: Cmd,
}
```

</details>
<details>
<summary>Examples</summary>


In this example there's only one command and it is required, so is the argument inside of it
```console
% app cmd --arg 42
Options { flag: false, cmd: Cmd { flag: false, arg: 42 } }
```

If you don't specify this command - parsing will fail
```console
% app 
Expected COMMAND ..., pass --help for usage information
```

You can have the same flag names inside and outside of the command, but it might be confusing
for the end user. This example enables the outer flag
```console
% app --flag cmd --arg 42
Options { flag: true, cmd: Cmd { flag: false, arg: 42 } }
```

And this one - both inside and outside
```console
% app --flag cmd --arg 42 --flag
Options { flag: true, cmd: Cmd { flag: true, arg: 42 } }
```

And that's the confusing part - unless you add context restrictions with
[`adjacent`](Parser::adjacent) and parse command first - outer flag wins.
So it's best not to mix names on different levels
```console
% app cmd --arg 42 --flag
Options { flag: true, cmd: Cmd { flag: false, arg: 42 } }
```

Commands show up on both outer level help
```console
% app --help
Usage: [--flag] COMMAND ...

Available options:
        --flag  This flag is specific to the outer layer
    -h, --help  Prints help information

Available commands:
    cmd  Command to do something
```

As well as showing their own help
```console
% app cmd --help
Command to do something

Usage: [--flag] --arg ARG

Available options:
        --flag       This flag is specific to command
        --arg <ARG>
    -h, --help       Prints help information
```

</details>
