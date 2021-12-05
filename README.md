# bpaf

Parse command line arguments by composing a parser from the components optimized for
flexibility and compilation time

[Documentation is available on docs.rs](href=https://docs.rs/bpaf).

# Design goals

- Flexibility
- Reusability
- Compilation time

# Design non-goals
- Runtime performance - more on that later

# Usage

Add `bpaf` under `[dependencies]` in your `Cargo.toml`

```toml
[dependencies]
bpaf = "0.1"
```

Start with [`short`], [`long`], [`command`] or [`positional`] to define fields used in parser,
combine them using [`or_else`][Parser::or_else], [`construct!`], [`apply!`] or [`tuple!`],
create parser [`Info`], attach it to the parser with [`for_parser`][Info::for_parser] and
execute with [`run`] to get the results out. As far as the rest of the application is concerned
 there's only one parameter. See [params] for starting points explanations.

```no_run
use bpaf::*;

fn speed() -> Parser<f64> {
    // Define how to parse speed given in KPH
    let speed_in_kph
        = short('k').long("speed_kph")   // give it a name
          .help("speed in KPH")          // and help message
          .argument("SPEED")             // it's an argument with metavar
          .build()                       // parameter definition
          .from_str::<f64>()             // that is parsed from string as f64
          .map(|s| s / 0.62);            // and converted to mph

    // Same for MPH
    let speed_in_mph
        = short('m').long("speed_mph")
          .help("speed in KPH")
          .argument("SPEED")
          .build()
          .from_str();

    // Resulting parser accepts either of those but not both at once
    speed_in_kph.or_else(speed_in_mph)
}

fn main() {
    let info = Info::default().descr("Accept speed in KPH or MPH, print it as MPH");
    let parser = speed();
    let decorated = info.for_parser(parser);
    let res = run(decorated);
    println!("Speed in MPH: {}", res);
}
```

# Design goals expanded

## Flexibility

Library allows to express accepted options by combining primitive parsers using mostly regular
Rust code. For example it is possible to take a parser that requires a single floating point
number and create a parser that takes a pair of them with names and help generated.

The main restriction library sets is that parsed values (but not the fact that parser succeeded
or failed) can't be used to decide how to parse subsequent values. In other words parsers don't
have the monadic strength, only the applicative one.


To give an example, this description is allowed:
"Program takes one of `--stdout` or `--file` flag to specify the output, when it's `--flag`
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

Library can handle alternatives and perform parsing and validation:

```no_run
use bpaf::*;

/// As far as the end user is concerned `speed` is a single argument that is always valid
fn speed() -> Parser<f64> {

    // define a simple string argument
    let kph = short('k').help("speed in KPH").argument("SPEED").build()
            .from_str::<f64>()                             // parse it from string to f64
            .guard(|&s| s > 0.0, "Speed must be positive"); // and add some restrictions

    let mph = short('m').help("speed in MPH").argument("SPEED").build()
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

Library aims provides a small number of components that can be composed in a multiple ways on
user side. Runtime performance is not an end goal since it's usually fast enough to take a tiny
fraction of whole program runtime while compiling so library uses dynamic dispatch to generate
less code and might perform additional clones if this allows to unify the code better. But
any noticable performance issues should be fixed.


# Compared with clap

Clap:
- Uses stringly typed values: `"v"` in declaration must match `"v"` in usage
- Uses specialized functions: `occurrences_of`
- No generic way of handle unexpected values diring parsing


```ignore
    // definition
    ...
      .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))

    // usage

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

```

bpaf:
- Value is parsed into a typed variable, usize in this case but could be an `enum`
- Combination of two generic parsers: [`many`][Parser::many] and [`parse`][Parser::parse]
- Invalid values are rejected during parse time with [`guard`][Parser::guard]
```ignore
    // definition

    short('v')
        .help("Sets the level of verbosity")
        .req_flag()
        .many().parse(|xs|xs.len())
        .guard(|x| x < 3)

    // usage
    match v {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => unreachable!(),
    }
```


## Clap to bpaf dictionary

| `Clap` |      `bpaf` |  explanation  |
|----------|-------------|------|
| `long`, `short` | [`long`], [`short`] ||
| `alias`, `aliases` | [`long`], [`short`] | You can specify names multiple times, first specified name (separately for `long` and `short`) becomes visible, remaining are hidden aliases |
| `*_os` | [`argument_os`][Argument::build_os] | With any parsing or validation on top of that |
| `allow_hyphen_values` | N/A | Hypens in parameters are accepted either with `--pattern=--bar` or as a positional argument after double dashes `-- --odd-file-name` |
| `case_insensitive` | [`parse`][Parser::parse] | You can use any parsing logic. |
| `conflicts_with` | [`or_else`][Parser::or_else] | `foo.or_else(bar)` either `foo` or `bar` will be accepted but not both, unless something else accepts `bar` |
| `conflicts_with_all`| [`or_else`][Parser::or_else] | can be chained: `foo.or_else(bar).or_else(baz)` |
| `default_value_if[s]`| N/A | Values produced by parsers can't depend on values produced by other parsers. Some functions are achievable with [`or_else`][Parser::or_else] |
| `default_value`| [`fallback`][Parser::fallback], [`fallback_with`][Parser::fallback_with] ||
| `display_order` | N/A | Order is fixed by construction order, you can put more important items first. Logically related commands can be combined into [`subcommands`][params::command]. |
| `env[_os]` | N/A | While using environment variables is not supported directly - it is possible to read configuration values from anywhere using [`fallback_with`][Parser::fallback_with]. It can be env variable, file, windows registry, etc. |
| `fallback_value` | [`fallback`][Parser::fallback] | But it's not limited to strings: `foo.fallback(Megapotato)` |
| `from_usage` | N/A | It's hard to produce anything but strings from that. |
| `from_yaml` | N/A | You can share parsers between multple programs by exporting them. Yaml requires external dependencies and gives stringly typed values. |
| `global` | N/A | Not really needed. Parsing in subcommands can't depend on any other flags but parsed values will be returned in a context that will contain global values. |
| `group[s]` | N/A | Stringly typed groups are not supported. Several parsers can be composed as alternatives with [`or_else`][Parser::or_else] or factored out into a subcommand with [`command`][params::command]. |
| `help`| [`help`][Named::help] | `help` is present on several object types. |
| `hidden_*` | N/A | TODO? |
| `index` | N/A | Arguments are not exposed to the user directly, `index` won't be of any use. |
| `last` | N/A | What's the use case? |
| `required` | [`req_flag`][Named::req_flag], [`argument`][Named::argument] | Arguments with no fallback values and not changed to [`optional`][Parser::optional] are required. |
| `require_equals` | N/A | `=` is always accepted but never required. Not sure about the usecase. |
| `require*` | [`or_else`][Parser::or_else] | One and only one in chained `or_else` sequence must succeed. |
| `takes_value` | [`argument`][Argument::build], [`argument_os`][Argument::build_os] | |
| `number_of_values`, `(max,min)_values` | N/A | Consuming multiple separate values with a single flag is not supported but it is possible to implement similar behavior using either custom [`parse`][Parser::parse] or by allowing user to specify an option [`many`][Parser::many] times and using [`guard`][Parser::guard] or [`parse`][Parser::parse] to specify exact limits. |
| `validator[_os]`, `possible_value[s]`, `empty_values` | [`parse`][Parser::parse], [`guard`][Parser::guard] | You can implement any parsing logic not limited to strings. |
| `*_delimiter` | N/A |  Clumped values are not supported directly with [`parse`][Parser::parse]. The alternative is to accept a parameter multiple times [`many`][Parser::many] |
| `value_name[s]`| N/A | You must specify metavar name when creating an [`argument`][Named::argument] or a positional (TODO) option |
| `visible_alias[es]` | [`or_else`][Parser::or_else] ||
| `with_name` | N/A | `bpaf` doesn't use stringly typed values, any parser can have any unique type attached to it using [`parse`][Parser::parse] or [`map`][Parser::map] |

## Compared with pico-args

- generates help message
- handles alternatives and subcommands


## vs macro based one

vs `structopt`, `gumdrop` and `argh`

- no proc macros:
- no syn/quote/proc-macro2 dependencies => faster compilation time
- pure rust, no cryptic macro commands, full support from tools
