//! # Parsing API overview
//!
//! ## Design and write down data type you want to consume
//!
//! Parsing usually starts from deciding what kind of data your APP wants to get. You want to take
//! advantage of Rust typesystem, try to represent the result such that more validation
//! can be done during parsing.
//!
//! <details>
//! <summary>A few examples</summary>
//! Use enums instead of huge structs for mutually exclusive options:
//!
//! ```no_check
//! enum OutputFormat {
//!     Intel,
//!     Att,
//!     Llvm
//! }
//!
//! fn main() {
//!     ...
//!     // `rustc` ensures you handle each case, parser won't try to consume
//!     // combinations of flags it can't represent. For example it won't accept
//!     // both `--intel` and `--att` at once
//!     // (unless it can collect multiple of them in a vector)
//!     match format {
//!         OutputFormat::Intel => ...,
//!         OutputFormat::Att=> ...,
//!         OutputFormat::Llvm=> ...,
//!     }
//! }
//! ```
//!
//! While it's easy to see how flags like `--intel` and `--att` maps to each of those bools,
//! consuming inside your app is more fragile
//!
//! ```no_check
//! struct OutputFormat {
//!     intel: bool,
//!     att: bool,
//!     llvm: bool,
//! }
//!
//! fn main() {
//!     ...
//!     // what happens when none matches? Or all of them?
//!     // What happens when you add a new output format?
//!     if format.intel {
//!         ...
//!     } else if format.att {
//!         ...
//!     } else if format.llvm {
//!         ...
//!     } else {
//!         // can this branch be reached?
//!     }
//! }
//! ```
//!
//! Mutually exclusive things are not limited to just flags. For example if your program can take
//! input from several different sources such as file, database or interactive input it's a good
//! idea to use enum as well:
//!
//! ```no_check
//! enum Input {
//!     File {
//!         filepath: PathBuf,
//!     }
//!     Database {
//!         user: String,
//!         password: String.
//!     }
//!     Interactive,
//! }
//! ```
//!
//! If your codebase uses newtype pattern - it's a good idea to use it starting from the command
//! options:
//!
//! ```no_check
//! struct Options {
//!     // better than taking a String and parsing internally
//!     date: NaiveDate,
//!     // f64 might work too, but you can start from some basic sanity checks
//!     speed: Speed
//!     ...
//! }
//! ```
//! </details>
//!
//! ## Primitive items on the command line
//!
//! If we are not talking about exotic cases most of the command line arguments can be narrowed
//! down to a few items:
//! <details>
//! <summary>An overview of primitive parser shapes</summary>
//! - an option with a short or a long name: `-v` or `--verbose`, short options can sometimes be
//!   squashed together: `-vvv` can be parsed the same as `-v -v -v` passed separately.
//!   If such option is parsed into a `bool` `bpaf` documentation calls them *switches*, if it
//!   parses into some fixed value - it's a *flag*.
//!
//!   <details>
//!   <summary>Examples of flags and switches</summary>
//!   <div class="code-wrap">
//!   <pre>
//!   cargo build <span style="font-weight: bold">--release</span>
//!   cargo test <span style="font-weight: bold">-q</span>
//!   cargo asm <span style="font-weight: bold">--intel</span>
//!   </pre>
//!   </div>
//!   </details>
//!
//! - an option with a short or a long name with extra value attached: `-p <PACKAGE>` or
//!   `--package <PACKAGE>`. Value can also be separated by `=` sign from the name or, in case
//!   of a short name, be adjacent to it: `--package=bpaf` and `-pbpaf`.
//!   `bpaf` documentation calls them *arguments*.
//!
//!
//!   <details>
//!   <summary>Examples of arguments</summary>
//!   <div class="code-wrap">
//!   <pre>
//!   cargo build <span style="font-weight: bold">--package bpaf</span>
//!   cargo test <span style="font-weight: bold">-j2</span>
//!   cargo check <span style="font-weight: bold">--bin=megapotato</span>
//!   </pre>
//!   </div>
//!   </details>
//!
//! - value taken from a command line just by being in the correct position and not being a flag.
//!   `bpaf` documentation calls them *positionals*.
//!
//!   <details>
//!   <summary>Examples of positionals</summary>
//!   <div class="code-wrap">
//!   <pre>
//!   cat <span style="font-weight: bold">/etc/passwd</span>
//!   rm -rf test <span style="font-weight: bold">target</span>
//!   man <span style="font-weight: bold">gcc</span>
//!   </pre>
//!   </div>
//!   </details>
//!
//! - a positional item that starts a whole new set of options with a separate help message.
//!   `bpaf` documentation calls them *commands* or *subcommands*.
//!
//!   <details>
//!   <summary>Examples of subcommands</summary>
//!   <div class="code-wrap">
//!   <pre>
//!   cargo <span style="font-weight: bold">build --release</span>
//!   cargo <span style="font-weight: bold">clippy</span>
//!   cargo <span style="font-weight: bold">asm --intel --everything</span>
//!   </pre>
//!   </div>
//!   </details>
//!
//! - value can be taken from an environment variable.
//!
//!   <details>
//!   <summary>Examples of environment variable</summary>
//!   <div class="code-wrap">
//!   <pre>
//!   <span style="font-weight: bold">CARGO_TARGET_DIR=~/shared</span> cargo build --release
//!   <span style="font-weight: bold">PASSWORD=secret</span> encrypt file
//!   </pre>
//!   </div>
//!   </details>
//!
//!   </details>
//!
//! `bpaf` allows you to describe the parsers using a mix of two APIs: combinatoric and derive.
//! Both APIs can achieve the same results, you can use one that better suits your needs. You can
//! find documentation with more examples following those links.
//!
//! - [`short`] and/or [`long`] + [`switch`](NamedArg::switch) - simple bool switch
//! - [`short`] and/or [`long`] + [`flag`](NamedArg::flag) - switch that returns a fixed value
//! - [`short`] and/or [`long`] + [`req_flag`](NamedArg::req_flag) - switch only succeeds when
//!   it's name is present on a command line
//! - [`short`] and/or [`long`] + [`argument`](NamedArg::argument) - argument containing a value
//! - [`env`](crate::env) can be combined with any of parsers listed above
//! - [`positional`] - positional argument, you can further customize it with
//!   [`strict`](ParsePositional::strict)
//! - [`command`](OptionParser::command) - command parser, you need to define [`OptionParser`]
//!   for the nested parser first.
//! - [`literal`] and [`any`] - escape hatches that can parse anything not fitting into usual
//!   classification
//!
//! ## Transforming and changing parsers
//!
//! By default primitive parsers gives you back a single `bool`, a single `PathBuf` or a single
//! value produced by [`FromStr`] trait, etc. You can further transform it by chaining methods from
//! [`Parser`] trait, some of those methods are applied automagically if you are using derive API:
//!
//! - [`optional`](Parser::optional) - return `None` if value is missing instead of failing, see
//!   also [`catch`](ParseOption::catch) .
//! - [`many`](Parser::many) and [`some`](Parser::some) - collect multiple values into a vector,
//!   see also [`catch`](ParseMany::catch) and [`catch`](ParseSome).
//! - [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with) - return a
//!   different value if parser fails to find what it is looking for.
//! - [`map`](Parser::map), [`parse`](Parser::parse) and [`guard`](Parser::guard) - transform
//!   and/or validate value produced by a parser
//! - [`to_options`](Parser::to_options) - finalize the parser and prepare to run it
//!
//! ## Combining multiple parsers together
//!
//! Once you have parsers for all the primitive fields figured out you can start combining them
//! together to produce a parser for a final result - data type you designed in the step one.
//! For derive API you apply annotations to data types with `#[derive(Bpaf)`] and `#[bpaf(..)]`,
//! with combinatoric API you use [`construct!`](crate::construct!) macro.
//!
//! ## Improving user experience
//!
//! `bpaf` would use doc comments on fields and values passed in various
//! [`help`](NamedArg)/[`help`](Positional::help)/[`help`](ParseAny::help) methods to generate
//! `--help` documentation, you can further improve it with those methods:
//!
//! - [`display_fallback`](ParseFallback::display_fallback) and
//!   [`debug_fallback`](ParseFallback::debug_fallback) will include `fallback` value into
//!   the generated help
//! - [`hide_usage`](Parser::hide_usage) and [`hide`](Parser::hide) - hide the parser from
//!   generated *Usage* line or whole generated help
//! - [`group_help`] and [`with_group_help`] - for help purposes only group one or more parsers
//!   into a block within generated help with a separate header
//! - [`usage`](Parser::usage) - customize usage for a primitive or composite parser
//! - [`usage`](OptionParser::usage) and [`with_usage`](OptionParser::with_usage) lets you to
//!   customize whole usage line as a whole either by completely overriding it or by building around it.
//!
//! By default with completion enabled `bpaf` would complete names for flags, arguments and
//! commands. You can also generate completion for argument values, possible positionals, etc.
//! This requires enabling **autocomplete** cargo feature.
//!
//! - [`complete`](Parser::complete) and [`complete_shell`](Parser::complete_shell)
//!
//! And finally you can generate documentation for command line in markdown/help and manpage
//! formats
//! - [`to_markdown`](OptionParser::to_markdown)
//! - [`to_manpage`](TODO)
//!
//! ## Testing your parsers and running them
//! - You can [`run`](OptionParser::run) the parser on the arguments passed on the command line
//! - [`check_invariants`](OptionParser::check_invariants) checks for a few invariants in the
//!   parser `bpaf` relies on
//! - [`run_inner`](OptionParser::run_inner) runs the parser with custom [`Args`] you can create
//!   either explicitly or implicitly using one of the [`From`] implementations, `Args` can be
//!   customized with [`set_comp`](Args::set_comp) and [`set_name`](Args::set_name).
//! - [`ParseFailure`] contains the parse outcome, you can consume it either by hands or using one
//!   of [`exit_code`](ParseFailure::exit_code), [`unwrap_stdout`](ParseFailure::unwrap_stdout) and
//!   [`unwrap_stderr`](ParseFailure::unwrap_stderr)

#[cfg(doc)]
use crate::*;
