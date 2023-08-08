#### Subcommand parsers

To make a parser for a subcommand you make an `OptionParser` for that subcommand first as if it
was the only thing your application would parse then turn it into a regular [`Parser`]
you can further compose with [`OptionParser::command`].

This gives [`ParseCommand`] back, you can add aliases or tweak the help message if you want to.


```rust,id:1
# use bpaf::*;
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

pub fn main() {
    println!("{:?}", options().run());
}
```

Help contains both commands, bpaf takes short command description from the inner command
description

```run,id:1
--help
```

Same as before each command gets its own help message

```run,id:1
run --help
```

And can be executed separately

```run,id:1
run --bin basic
```

```run,id:1
build --bin demo --release
```
