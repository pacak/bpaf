#### Structured API reference

# Primitive items on the command line

If we are not talking about exotic cases most of the command line arguments can be narrowed
down to a few items:
<details>
<summary>An overview of primitive parser shapes</summary>

- an option with a short or a long name: `-v` or `--verbose`, short options can sometimes be
  squashed together: `-vvv` can be parsed the same as `-v -v -v` passed separately.
  If such option is parsed into a `bool` `bpaf` documentation calls them *switches*, if it
  parses into some fixed value - it's a *flag*.

  <details>
  <summary>Examples of flags and switches</summary>
  <div class="code-wrap">
  <pre>
  cargo build <span style="font-weight: bold">--release</span>
  cargo test <span style="font-weight: bold">-q</span>
  cargo asm <span style="font-weight: bold">--intel</span>
  </pre>
  </div>
  </details>

- an option with a short or a long name with extra value attached: `-p PACKAGE` or
  `--package PACKAGE`. Value can also be separated by `=` sign from the name or, in case
  of a short name, be adjacent to it: `--package=bpaf` and `-pbpaf`.
  `bpaf` documentation calls them *arguments*.


  <details>
  <summary>Examples of arguments</summary>
  <div class="code-wrap">
  <pre>
  cargo build <span style="font-weight: bold">--package bpaf</span>
  cargo test <span style="font-weight: bold">-j2</span>
  cargo check <span style="font-weight: bold">--bin=megapotato</span>
  </pre>
  </div>
  </details>

- value taken from a command line just by being in the correct position and not being a flag.
  `bpaf` documentation calls them *positionals*.

  <details>
  <summary>Examples of positionals</summary>
  <div class="code-wrap">
  <pre>
  cat <span style="font-weight: bold">/etc/passwd</span>
  rm -rf <span style="font-weight: bold">target</span>
  man <span style="font-weight: bold">gcc</span>
  </pre>
  </div>
  </details>

- a positional item that starts a whole new set of options with a separate help message.
  `bpaf` documentation calls them *commands* or *subcommands*.

  <details>
  <summary>Examples of subcommands</summary>
  <div class="code-wrap">
  <pre>
  cargo <span style="font-weight: bold">build --release</span>
  cargo <span style="font-weight: bold">clippy</span>
  cargo <span style="font-weight: bold">asm --intel --everything</span>
  </pre>
  </div>
  </details>

- value can be taken from an environment variable.

  <details>
  <summary>Examples of environment variable</summary>
  <div class="code-wrap">
  <pre>
  <span style="font-weight: bold">CARGO_TARGET_DIR=~/shared</span> cargo build --release
  <span style="font-weight: bold">PASSWORD=secret</span> encrypt file
  </pre>
  </div>
  </details>

  </details>

`bpaf` allows you to describe the parsers using a mix of two APIs: combinatoric and derive.
Both APIs can achieve the same results, you can use one that better suits your needs. You can
find documentation with more examples following those links.

- For an argument with a name you define [`NamedArg`] using a combination of [`short`],
  [`long`] and [`env`](crate::env). At the same time you can attach
  [`help`](NamedArg::help).
- [`NamedArg::switch`] - simple switch that returns `true` if it's present on a command
  line and `false` otherwise.
- [`NamedArg::flag`] - a variant of `switch` that lets you return one of two custom
  values, for example `Color::On` and `Color::Off`.
- [`NamedArg::req_flag`] - a variant of `switch` that only only succeeds when it's name
  is present on a command line
- [`NamedArg::argument`] - named argument containing a value, you can further
  customize it with [`adjacent`](crate::parsers::ParseArgument::adjacent)
- [`positional`] - positional argument, you can further customize it with
  [`strict`](ParsePositional::strict)
- [`OptionParser::command`] - subcommand parser.
- [`any`] and its specialized version [`literal`] are escape hatches that can parse anything
  not fitting into usual classification.
- [`pure`] and [`pure_with`] - a way to generate a value that can be composed without parsing
  it from the command line.

## 3. Transforming and changing parsers

By default primitive parsers gives you back a single `bool`, a single `PathBuf` or a single
value produced by [`FromStr`] trait, etc. You can further transform it by chaining methods from
[`Parser`] trait, some of those methods are applied automagically if you are using derive API.

`bpaf` distinguishes two types of parse failures - "value is absent" and
"value is present but invalid", most parsers listed in this section only handle the first
type of falure by default, but you can use their respective `catch` method to handle the later
one.

- [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with) - return a
  different value if parser fails to find what it is looking for. Generated help for former
  can be updated to include default value using
  [`display_fallback`](ParseFallback::display_fallback) and
  [`debug_fallback`](ParseFallback::debug_fallback) .
- [`optional`](Parser::optional) - return `None` if value is missing instead of failing, see
  also [`catch`](ParseOptional::catch) .
- [`many`](Parser::many), [`some`](Parser::some) and [`collect`](Parser::collect) - collect
  multiple values into a collection, usually a vector, see their respective
  [`catch`](ParseMany::catch), [`catch`](ParseSome::catch) and [`catch`](ParseCollect::catch).
- [`map`](Parser::map), [`parse`](Parser::parse) and [`guard`](Parser::guard) - transform
  and/or validate value produced by a parser
- [`to_options`](Parser::to_options) - finalize the parser and prepare to run it

## 4. Combining multiple parsers together

Once you have parsers for all the primitive fields figured out you can start combining them
together to produce a parser for a final result - data type you designed in the step one.
For derive API you apply annotations to data types with `#[derive(Bpaf)`] and `#[bpaf(..)]`,
with combinatoric API you use [`construct!`](crate::construct!) macro.

All fields in a struct needs to be successfully parsed in order for the parser to succeed
and only one variant from enum will consume its values at a time.

You can use [`adjacent`](ParseCon::adjacent) annotation to parse multiple flags as an adjacent
group allowing for more unusual scenarios such as multiple value arguments or chained commands.

## 5. Improving user experience

`bpaf` would use doc comments on fields and structures in derive mode and and values passed
in various `help` methods to generate `--help` documentation, you can further improve it
using those methods:

- [`hide_usage`](Parser::hide_usage) and [`hide`](Parser::hide) - hide the parser from
  generated *Usage* line or whole generated help
- [`group_help`](Parser::group_help) and [`with_group_help`](Parser::with_group_help) -
  add a common description shared by several parsers
- [`custom_usage`](Parser::custom_usage) - customize usage for a primitive or composite parser
- [`usage`](OptionParser::usage) and [`with_usage`](OptionParser::with_usage) lets you to
  customize whole usage line as a whole either by completely overriding it or by building around it.

By default with completion enabled `bpaf` would complete names for flags, arguments and
commands. You can also generate completion for argument values, possible positionals, etc.
This requires enabling **autocomplete** cargo feature.

- [`complete`](Parser::complete) and [`complete_shell`](Parser::complete_shell)

And finally you can generate documentation for command line in markdown, html and manpage
formats using [`render_markdown`](OptionParser::render_markdown),
[`render_html`](OptionParser::render_html) and [`render_manpage`](OptionParser::render_manpage),
for more detailed info see [`doc`] module

## 6. Testing your parsers and running them
- You can [`OptionParser::run`] the parser on the arguments passed on the command line
- [`check_invariants`](OptionParser::check_invariants) checks for a few invariants in the
  parser `bpaf` relies on
- [`run_inner`](OptionParser::run_inner) runs the parser with custom [`Args`] you can create
  either explicitly or implicitly using one of the [`From`] implementations, `Args` can be
  customized with [`set_comp`](Args::set_comp) and [`set_name`](Args::set_name).
- [`ParseFailure`] contains the parse outcome, you can consume it either by hands or using one
  of [`exit_code`](ParseFailure::exit_code), [`unwrap_stdout`](ParseFailure::unwrap_stdout) and
  [`unwrap_stderr`](ParseFailure::unwrap_stderr)
