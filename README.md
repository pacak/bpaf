# bpaf

Parse command line arguments by composing a parser from the components

# Usage

Add `bpaf` under `[dependencies]` in your `Cargo.toml`

```toml
[dependencies]
bpaf = "0.1"
```

Define fields used in parser

```i
use bpaf::*;
use std::str::FromStr;

fn speed() -> Parser<f64> {

    let speed_in_kph
        = short('k').long("speed_kph")         // give it a name
          .argument()                          // it's an argument
          .metavar("SPEED")                    // with metavar
          .help(("speed in KPH").build()       // and help message
          .parse(|s| f64::from_str(&s).map(|s| s / 0.62)); // that is parsed from string

    let speed_in_mph
        = short('m').long("speed_mph")
          .argument()
          .metavar("SPEED")
          .help(("speed in KPH").build()
          .parse(|s| f64::from_str(&s));

    speed_in_kph.or_else(speed_in_mph)
}
```

Arguments can be composed in multiple ways, for example
if application wants speed - it can accept it in either of two formats

```rust
```

As far as the rest of the application is concerned - there's only one parameter

# Design goals

## Flexibility

The main restriction library sets is that parsed values (but not the fact that parser succeeded
or failed) can't be used to decide how to parse subsequent values. In other words parsers don't
have the monadic strength, only the applicative one.


To give an example, this description is allowed:
"Program takes one of --stdout or --file flag to specify the output, when it's --flag
program also requires -f attribute with the file name".

```txt
(--stdout | (--file -f ARG))
```

But this one isn't.
"Program takes an -o attribute with possible values of 'stdout' and 'file', when it's 'file'
program also requires -f attribute with the file name".


```

```

# main features


## full help is generated including usage line and a list of commands
```
Usage: [-a|--AAAAA] (-b) (-m|--mph) | (-k|--kph) COMMAND

this is a test

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

## alternatives and parse failures are handled at parsing level:

```rust

let kph = short('k').argument().metavar("SPEED").help("speed in KPH").build()
        .parse(|s| f64::from_string(s)?).guard(|s| s > 0);
let mph = short('m').argument().metavar("SPEED").help("speed in MPH").build()
        .parse(|s| f64::from_string(s)?  * 1.6).guard(|s| s > 0);
let speed = kph.or(mph);
```

## composable and reusable
```
fn get_something() -> Parser<Foo> {
   // define flag/argument/positional here with help and everything related
   // it's a usual function which you can export and reuse across multiple
   // applications
}
```


# fast compilation time

library provides a small number of components that can be composed
in a multiple ways on user side


# compare vs

## vs clap

Clap:
- stringly typed value: `"v"` in declaration must match `"v"` in usage
- magical single purpose function: `occurrences_of`
- compile time panic if number of occurances is too much


```ignore
    ...
      .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))


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
- value is parsed into a typed variable
- combination of two generic parsers: [`many`] and [`parse`]
- invalid values are rejected during parse time with [`guard`]
```ignore

    short('v').req_flag()
        .help("Sets the level of verbosity")
        .many().parse(|xs|xs.len())
        .guard(|x| x < 3)
    ...

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
| `*_os` | [`argument_os`] | Plus any parsing you want on top of that |
| `alias` | [`short`], [`long`] | First name becomes visible, remaining are hidden aliases |
| `aliases` | [`short`], [`long`]  | You pay a single push per alias |
| `allow_hyphen_values` | N/A | values like `--pattern=--bar` are accepted as is |
| `case_insensitive` | N/A | You can use a custom parsing with [`parse`] |
| `case_insensitive` | [`parse`] | You can use any parsing logic. |
| `conflicts_with` | [`or_else`] | `foo.or_else(bar)` either `foo` or `bar` will be accepted but not both, unless something else accepts `bar` |
| `conflicts_with_all`| [`or_else`] | can be chained: `foo.or_else(bar).or_else(baz)` |
| `default_value_if[s]`| N/A | Values produced by parsers can't depend on values produced by other parsers. Some functions are achievable with [`or_else`] |
| `default_value`| [`fallback_value`] ||
| `display_order` | N/A | Order is fixed by construction order, you can put more important items first. |
| `empty_values` | [`guard`] | Same as any validation |
| `env[_os]` | [`fallback_with`] | |
| `fallback_value` | [`fallback`] | But it's not limited to strings: `foo.fallback(Megapotato)` |
| `from_usage` | N/A | In the best case scenario you'll get some stringly typed values. Just no. |
| `from_yaml` | N/A | You can share parsers between multple programs by exporting them. Yaml requires external dependencies and gives stringly typed values. Also no. |
| `global` | N/A | Not really needed. Parsing in subcommands can't depend on any other flags but parsed values will be returned in a context that will contain global values. |
| `group[s]` | [`or_else`], [`command`] | Instead of making a stringly typed group - you can make a real one. |
| `help`| [`help`] ||
| `hidden_*` | N/A | TODO? |
| `index` | N/A | Arguments are not exposed to the user directly, `index` won't be of any use. |
| `last` | N/A | What's the use case? |
| `long` | [`long`] ||
| `required` | [`req_flag`], [`req_switch`], [`argument`] | arguments with no fallback values and not optional ones are required by default. |
| `require_equals` | N/A | What's the use case? |
| `require*` | [`or_else`] | One and only one in chained `or_else` sequence must succeed. |
| `takes_value` | [`argument`], [`argument_os`] | |
| `number_of_values` | [`many`] + [`guard`] | This will require user to specify `-f` multiple times but that's how most of the applications out there do it. |
| `(max,min)_values` | [`many`] + [`guard`]| Same. |
| `possible_value[s]` | [`parse`], [`guard`] | You can use any parsing logic.|
| `require_delimiter` | [`parse`], [`many`]||
| `short` | [`short`] ||
| `use_delimiter` | [`parse`], [`many`] ||
| `validator[_os]`| [`parse`], [`guard`]| You can use any parsing logic. It's not limited to strings. |
| `value_delimiter` |[`parse`], [`many`]||
| `value_name[s]`| [`metavar`] ||
| `visible_alias[es]` | [`or_else`] ||
| `with_name` | N/A ||

## vs pico-args

- generates help message
- handles alternatives and subcommands


## vs macro based one

vs `structopt`, `gumdrop` and `argh`

- no proc macros:
- no syn/quote/proc-macro2 dependencies => faster compilation time
- pure rust, no cryptic macro commands, full support from tools
