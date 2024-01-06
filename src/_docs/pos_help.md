## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Output detailed help information, you can specify it multiple times
    ///
    ///  when used once it outputs basic diagnostic info,
    ///  when used twice or three times - it includes extra debugging.
    //  ^ note extra spaces before when that preserve the linebreaks
    verbose: bool,

    #[bpaf(argument("NAME"))]
    /// Use this as a task name
    name: String,

    #[bpaf(positional("OUTPUT"))]
    /// Save output to a file
    output: Option<String>,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    name: String,
    output: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help(
            "\
Output detailed help information, you can specify it multiple times

 when used once it outputs basic diagnostic info,
 when used twice or three times - it includes extra debugging.",
            // ^ note extra spaces before "when" that preserve the linebreaks
        )
        .switch();
    let name = long("name")
        .help("Use this as a task name")
        .argument("NAME");

    let output = positional("OUTPUT")
        .help("Save output to a file")
        .optional();

    construct!(Options {
        verbose,
        name,
        output
    })
    .to_options()
}
````

When `--help` used once it renders shorter version of the help information



```text
$ app --help
Usage: app [-v] --name=NAME [OUTPUT]

Available positional items:
    OUTPUT           Save output to a file

Available options:
    -v, --verbose    Output detailed help information, you can specify it multiple times
        --name=NAME  Use this as a task name
    -h, --help       Prints help information
```


When used twice - it renders full version. Documentation generator uses full
version as well



```text
$ app --help --help
Usage: app [-v] --name=NAME [OUTPUT]

Available positional items:
    OUTPUT           Save output to a file

Available options:
    -v, --verbose    Output detailed help information, you can specify it multiple times
                      when used once it outputs basic diagnostic info,
                     when used twice or three times - it includes extra debugging.
        --name=NAME  Use this as a task name
    -h, --help       Prints help information
```


Presence or absense of a help message should not affect the parser's output



```text
$ app --name Bob output.txt
Options { verbose: false, name: "Bob", output: Some("output.txt") }
```

