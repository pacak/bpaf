//! #### Introduction and design goals
//!
//! `bpaf` is a command line parser that is both lightweight and flexible. It supports two different
//! API styles, combinatoric and derive. You can choose to use either style, or mix and match them in
//! a single parser: picking one API style does not lock you out from using the other style.
//!
//! ##### Combinatoric API
//!
//! This API allows you to explicitly specify the parsers that you want to use and how they should
//! be combined. This gives you the most control over the parsing process, but it can also be more
//! verbose. Your IDE will be able to help you more - it can complete methods from traits and types.
//!
//! ````rust
//! use bpaf::*;
//!
//! #[derive(Debug, Clone)]
//! pub struct Options {
//!     message: String,
//! }
//!
//! pub fn options() -> OptionParser<Options> {
//!     let message = positional("MESSAGE")
//!         .help("Message to print in a big friendly letters");
//!     construct!(Options { message }).to_options()
//! }
//! ````
//!
//! ##### Derive API
//!
//! This API uses proc macros to automatically generate the parsers for your structs. This is more
//! concise and easier to use, but IDE won't be able to help as much - annotations are not valid
//! rust.
//!
//! ````rust
//! use bpaf::*;
//!
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)]
//! pub struct Options {
//!     /// Message to print in a big friendly letters
//!     #[bpaf(positional("MESSAGE"))]
//!     message: String,
//! }
//! ````
//!
//! ##### Sample output
//!
//! Either example above produces the same result: program takes a single argument positionally
//!
//!
//!
//! ```text
//! $ app "Hello world"
//! Options { message: "Hello world" }
//! ```
//!
//!
//! As well as the `--help` switch to output the help message generated by the library
//!
//!
//!
//! ```text
//! $ app --help
//! Usage: app MESSAGE
//!
//! Available positional items:
//!     MESSAGE     Message to print in a big friendly letters
//!
//! Available options:
//!     -h, --help  Prints help information
//! ```
//!
//!
//! #### Design goals
//!
//! ##### Parse, don't validate
//!
//! The bpaf library tries to make it easy to represent the invariants of your user input in Rust
//! types. For example, if you have two mutually exclusive options coming from several parsers, you
//! can represent them as an enum with two variants. You can also collect the intermediate results
//! of parsing into a type like [`BTreeSet`](std::collections::BTreeSet) or any other custom type.
//!
//! In addition to representing invariants in types, bpaf also supports validation. This means that
//! you can check the validity of your input after it has been parsed. For example, you could check
//! that the sum of every numeric field is divisible by both 3 and 5, but only when it's Thursday.
//! Ideas for
//! [making invalid states unrepresentable](https://geeklaunch.io/blog/make-invalid-states-unrepresentable/)
//! and [using parsing over validation](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
//! are not new.
//!
//! ##### Flexibility
//!
//! While aiming to be a general-purpose command line parser `bpaf` offers a few backdoors that
//! allow you to parse pretty much anything you want: chained commands, custom blocks of options,
//! DOS-style options (`/ofile.pas`), `dd` style options (`if=file of=out`), etc. A similar idea applies
//! to what the parser can produce - if your app operates with boxed string slices internally - `bpaf`
//! will give you `Box<str>` instead of `String` if you ask it to.
//!
//! The only restriction is that you cannot use information from items parsed earlier (but not
//! the fact that something was parsed successfully or not) to decide to how to parse further
//! options, and even then you can side step this restriction by passing some shared state as a
//! parameter to the parsers.
//!
//! ##### Reusability
//!
//! Parsers in `bpaf` are not monolithic and you can share their parts across multiple binaries,
//! workspace members or even independent projects. Say you have multiple binaries in a workspace
//! that perform different operations on some input. You can declare a parser for the input
//! specifically, along with all the validations, help messages or shell dynamic completion
//! functions you need and use it across all the binaries alongside the arguments specific to
//! those binaries.
//!
//! ##### Composition, transformation
//!
//! Parsers in `bpaf` are not finalized either, say you have a parser that describes a single input
//! for your program, it can take multiple arguments or perform extra validations, etc. You can
//! always compose this parser with any other parser to produce tuples of both results for example.
//! Or to make it so parser runs multiple times and collects results into a `Vec`.
//!
//! ##### Performance
//!
//! While performance is an explicit non-goal - `bpaf` does nothing that would pessimize it either,
//! so performance is on par or better compared to other fully featured parsers.
//!
//! ##### Correctness
//!
//! `bpaf` would parse only items it can represent and will reject anything it cannot represent
//! in the output. Say your parser accepts both `--intel` and `--att` flags, but encodes the result
//! into `enum Style { Intel, Att }`, `bpaf` will accept those flags separately, but not if they
//! are used both at once. If the parser later collects multiple styles into a `Vec<Style>` then it
//! will accept any combinationof those flags.
//!
//! ##### User friendly
//!
//! `bpaf` tries to provide user-friendly error messages, and suggestions for typos but also scripts
//! for shell completion, `man` pages and markdown documentation for the web.
#[allow(unused_imports)] use crate::{*, parsers::*};