# Compared with clap

Clap:
- Uses stringly typed values: `"v"` in declaration must match `"v"` in usage
- Uses specialized functions: `occurrences_of`
- No generic way of handle unexpected values during parsing


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
- Combination of two generic parsers: [`many`][Parser::many] and [`map`][Parser::map]
- Invalid values are rejected during parse time with [`guard`][Parser::guard]
```ignore
    // definition
    short('v')
        .help("Sets the level of verbosity")
        .req_flag()
        .many()
        .map(|xs| xs.len())
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
| `long`, `short`, `alias`, `aliases` | [`long`], [`short`] | You can specify names multiple times, first specified name (separately for `long` and `short`) becomes visible, remaining are hidden aliases |
| `*_os` | [`arguments_os`][Named::argument_os], [`positional_os`] | With any parsing or validation on top of that |
| `allow_hyphen_values` | N/A | Hypens in parameters are accepted either with `--pattern=--bar` or as a positional argument after double dashes `-- --odd-file-name` |
| `case_insensitive` | [`parse`][Parser::parse] | You can use any parsing logic. |
| `conflicts_with[_all]` | [`or_else`][Parser::or_else] | `foo.or_else(bar)` either `foo` or `bar` will be accepted but not both, unless something else accepts `bar`, can be chained: `foo.or_else(bar).or_else(baz)` |
| `default_value_if[s]`| N/A | Values produced by parsers can't depend on values produced by other parsers. Some functions are achievable with [`or_else`][Parser::or_else] |
| `default_value`| [`fallback`][Parser::fallback], [`fallback_with`][Parser::fallback_with] ||
| `display_order` | N/A | Order is fixed by construction order, you can put more important items first. Logically related commands can be combined into [`subcommands`][params::command]. |
| `env[_os]` | N/A | While using environment variables is not supported directly - it is possible to read configuration values from anywhere using [`fallback_with`][Parser::fallback_with]. It can be env variable, file, windows registry, etc. |
| `fallback_value` | [`fallback`][Parser::fallback] | But it's not limited to strings: `foo.fallback(Megapotato)` |
| `from_usage` | N/A | It's hard to produce anything but strings from that. |
| `from_yaml` | N/A | You can share parsers between multple programs by exporting them. Yaml requires external dependencies and gives stringly typed values. |
| `global` | N/A | Not really needed. Parsing in subcommands can't depend on any other flags but parsed values will be returned in a context that will contain global values. |
| `group[s]` | N/A | Stringly typed groups are not supported. Several parsers can be composed as alternatives with [`or_else`][Parser::or_else] or factored out into a subcommand with [`command`][params::command]. |
| `help`| [`help`][Named::help], [`help`][Parser::help] | `help` is present on several object types. |
| `hidden_*` | N/A | TODO? |
| `index` | N/A | Arguments are not exposed to the user directly, `index` won't be of any use. |
| `last` | N/A | What's the use case? |
| `required` | [`req_flag`][Named::req_flag], [`argument`][Named::argument] | Arguments with no fallback values and not changed to [`optional`][Parser::optional] are required. |
| `require_equals` | N/A | `=` is always accepted but never required. Not sure about the usecase. |
| `require*` | [`or_else`][Parser::or_else] | One and only one in chained `or_else` sequence must succeed. |
| `takes_value` | [`argument`][Named::argument], [`argument_os`][Named::argument_os] | |
| `number_of_values`, `(max,min)_values` | N/A | Consuming multiple separate values with a single flag is not supported but it is possible to implement similar behavior using either custom [`parse`][Parser::parse] or by allowing user to specify an option [`many`][Parser::many] times and using [`guard`][Parser::guard] or [`parse`][Parser::parse] to specify exact limits. |
| `validator[_os]`, `possible_value[s]`, `empty_values` | [`parse`][Parser::parse], [`guard`][Parser::guard] | You can implement any parsing logic not limited to strings. |
| `*_delimiter` | N/A |  Clumped values are not supported directly with [`parse`][Parser::parse]. The alternative is to accept a parameter multiple times [`many`][Parser::many] |
| `value_name[s]`| N/A | You must specify metavar name when creating an [`argument`][Named::argument] or a positional (TODO) option |
| `visible_alias[es]` | [`or_else`][Parser::or_else] ||
| `with_name` | N/A | `bpaf` doesn't use stringly typed values, any parser can have any unique type attached to it using [`parse`][Parser::parse] or [`map`][Parser::map] |
