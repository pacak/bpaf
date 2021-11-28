# bpaf

Parse command line arguments by composing a parser from the components

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

Define fields used in parser, attach meta information and execute to get the results out.
As far as the rest of the application is concerned - there's only one parameter.

```no_run
use bpaf::*;

fn speed() -> Parser<f64> {
    // Define how to parse speed given in KPH
    let speed_in_kph
        = short('k').long("speed_kph")   // give it a name
          .argument("SPEED")             // it's an argument with metavar
          .help("speed in KPH").build()  // and help message
          .from_str::<f64>()             // that is parsed from string as f64
          .map(|s| s / 0.62);            // and converted to mph

    // Same for MPH
    let speed_in_mph
        = short('m').long("speed_mph")
          .argument("SPEED")
          .help("speed in KPH").build()
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

The main restriction library sets is that parsed values (but not the fact that parser succeeded
or failed) can't be used to decide how to parse subsequent values. In other words parsers don't
have the monadic strength, only the applicative one.


To give an example, this description is allowed:
"Program takes one of `--stdout` or `--file` flag to specify the output, when it's `--flag`
program also requires `-f` attribute with the file name".

```txt
(--stdout | (--file -f ARG))
```

But this one isn't.
"Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it's `'file'`
program also requires `-f` attribute with the file name".


```txt
-o ARG ???????
```

A better approach would be to have two separate parsers that both transform into a single
`enum Output { StdOut, File(File) }` datatype combined with [`or_else`].

Library can handle alternatives and perform parsing and validation:

```no_run
use bpaf::*;

/// As far as the end user is concerned `speed` is a single argument that is always valid
fn speed() -> Parser<f64> {

    // define a simple string argument
    let kph = short('k').argument("SPEED").help("speed in KPH").build()
            .from_str::<f64>()                             // parse it from string to f64
            .guard(|&s| s > 0.0, "Speed must be positive"); // and add some restrictions

    let mph = short('m').argument("SPEED").help("speed in MPH").build()
            .from_str::<f64>()
            .map(|s|s * 1.6)  // can also apply transformations
            .guard(|&s| s > 0.0, "Speed must be positive");

    // compose parsers and apply one more validation for composed parser
    kph.or_else(mph).guard(|&s| s <= 99.9, "That's way too fast")
}
```

## Composable and reusable

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


## Fast compilation time

Library aims provides a small number of components that can be composed
in a multiple ways on user side.


# Compared with clap

Clap:
- stringly typed value: `"v"` in declaration must match `"v"` in usage
- magical single purpose function: `occurrences_of`
- compile time panic if number of occurances is too much


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
- value is parsed into a typed variable, usize in this case but could be an `enum`
- combination of two generic parsers: [`many`] and [`parse`]
- invalid values are rejected during parse time with [`guard`]
```ignore
    // definition

    short('v').req_flag()
        .help("Sets the level of verbosity")
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
| `*_os` | [`argument_os`] | Plus any parsing you want on top of that |
| `alias` | [`short`], [`long`] | First name becomes visible, remaining are hidden aliases |
| `aliases` | [`short`], [`long`]  | You pay a single push per alias |
| `allow_hyphen_values` | N/A | values like `--pattern=--bar` are accepted as is |
| `case_insensitive` | [`parse`][Parser::parse] | You can use any parsing logic. |
| `conflicts_with` | [`or_else`][Parser::or_else] | `foo.or_else(bar)` either `foo` or `bar` will be accepted but not both, unless something else accepts `bar` |
| `conflicts_with_all`| [`or_else`][Parser::or_else] | can be chained: `foo.or_else(bar).or_else(baz)` |
| `default_value_if[s]`| N/A | Values produced by parsers can't depend on values produced by other parsers. Some functions are achievable with [`or_else`][Parser::or_else] |
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
| `value_name[s]`| N/A | You need to specify it when creating an [`argument`][Named::argument] or a positional (TODO) option |
| `visible_alias[es]` | [`or_else`][Parser::or_else] ||
| `with_name` | N/A ||

# Compared with pico-args

- generates help message
- handles alternatives and subcommands


## vs macro based one

vs `structopt`, `gumdrop` and `argh`

- no proc macros:
- no syn/quote/proc-macro2 dependencies => faster compilation time
- pure rust, no cryptic macro commands, full support from tools
