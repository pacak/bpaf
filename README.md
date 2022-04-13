# bpaf

Parse command line arguments by composing a parser from the components optimized for
flexibility and compilation time

# Usage

Add `bpaf` under `[dependencies]` in your `Cargo.toml`

```toml
[dependencies]
bpaf = "0.3"
```

Start with [`short`], [`long`], [`command`] or [`positional`] to define fields used in parser, use
some of the member functions from [`Parser`] to apply further processing, combine parsers using
[`construct!`] or [`or_else`][Parser::or_else], create a parser [`Info`], attach it to the parser
with [`for_parser`][Info::for_parser] and execute with [`run`][OptionParser::run] to get the
results out. As far as the rest of the application is concerned there's only one parameter. See
[params] for starting points explanations.

```no_run
use bpaf::*;

#[derive(Clone, Debug)]
struct Opts {
    speed: f64,
    distance: f64,
}

fn opts() -> Opts {
    let speed = short('k')
        .long("speed")           // give it a name
        .help("speed in KPH")    // and help message
        .argument("SPEED")       // it's an argument with metavar
        .from_str()              // that is parsed from string as f64
        .map(|s: f64| s / 0.62); // and converted to mph

    let distance = short('d')
        .long("distance")
        .help("distance in miles")
        .argument("DISTANCE")
        .from_str();

    // combine parsers `speed` and `distance` parsers into a parser for Opts
    let parser = construct!(Opts { speed, distance });

    // define help message, attach it to parser, and run the results
    Info::default().descr("Accept speed and distance, print them").for_parser(parser).run()
}

fn main() {
    let opts = opts();
    println!("Options: {opts:?}");
}
```

# Design goals

## Flexibility

Library allows to express accepted options by combining primitive parsers using mostly regular
Rust code. For example it is possible to take a parser that requires a single floating point
number and create a parser that takes a pair of them with names and help generated.

The main restriction library sets is that parsed values (but not the fact that parser succeeded
or failed) can't be used to decide how to parse subsequent values. In other words parsers don't
have the monadic strength, only the applicative one.


To give an example, this description is allowed:
"Program takes one of `--stdout` or `--file` flag to specify the output, when it's `--file`
program also requires `-f` attribute with the file name".

```txt
(--stdout | (--file -f ARG))
```

But this one isn't:
"Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it's `'file'`
program also requires `-f` attribute with the file name".


```txt
-o ARG ???????
```

A better approach would be to have two separate parsers that both transform into a single
`enum Output { StdOut, File(File) }` datatype combined with [`or_else`][Parser::or_else].


```rust
use bpaf::*;

#[derive(Clone)]
enum Output { StdOut, File(String) };

let stdout = long("stdout").req_flag(Output::StdOut);
let file = long("file").argument("FILE").map(Output::File);
let output: Parser<Output> = stdout.or_else(file);
```

Library can handle alternatives and perform parsing and validation:

```rust
use bpaf::*;

/// As far as the end user is concerned `speed` is a single argument that is always valid
fn speed() -> Parser<f64> {

    // define a simple string argument
    let kph = short('k').help("speed in KPH").argument("SPEED")
            .from_str::<f64>()                             // parse it from string to f64
            .guard(|&s| s > 0.0, "Speed must be positive"); // and add some restrictions

    let mph = short('m').help("speed in MPH").argument("SPEED")
            .from_str::<f64>()
            .map(|s|s * 1.6)  // can also apply transformations
            .guard(|&s| s > 0.0, "Speed must be positive");

    // compose parsers and apply one more validation for composed parser
    kph.or_else(mph).guard(|&s| s <= 99.9, "That's way too fast")
}
```

## Reusability

`speed` defined in a previous example is a regular Rust function that can be exported and
reused in many places. It can also be composed with other parsers to produce more parsers.

## Help generation

A typical set of options would generate a help message similar to this one:
```txt
Usage: [-a|--AAAAA] -b (-m|--mph ARG) | (-k|--kph ARG) COMMAND

This is a sample program

Available options:
    -a, AAAAA        maps to a boolean, is optional
    -b               also maps to a boolean but mandatory
    -m, mph <SPEED>  speed in MPH
    -k, kph <SPEED>  speed in KPH
    -h, help         Prints help information
    -v, version      Prints version information

Available commands:
    accel  command for acceleration
```


## Fast compilation time, slower runtime performance

Library aims to provide a small number of components that can be composed in a multiple ways on
user side. Runtime performance is not an end goal since it's usually fast enough to take a tiny
fraction of whole program runtime while compiling so library uses dynamic dispatch to generate
less code and might perform additional clones if this allows to unify the code better. But
any noticable performance issues should be fixed.


## Implementing cargo commands

When implementing a cargo subcommand parser needs to be able to consume the first argument which
is always the same as the executable name minus `cargo-` prefix. For example executable named `cargo-super`
will be receiving `"super"` as its first argument. There's two ways to do thins:

- wrap eveything into a [`command`] with this name. Pros: minimal chances of it misfiring, cons:
  when using from a repository directly with `cargo run` users will have to specify the command
  name as well

- use [`cargo_helper`]. Pros: supports both `cargo super ...` and `cargo run ...` variants, cons:
  if first parameter accepted happens to be a file named `"super"` `cargo_helper` might silently
  consume it when used in `cargo run ...` scenario.


## Derive macros

Derive macros are reexported with `derive` feature, disabled by default
