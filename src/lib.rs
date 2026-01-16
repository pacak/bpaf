//!
//! All parsers in *bpaf* consist of primitive parsers enhanced with [`Parser`] trait and glued
//! together with [`construct!`] macro. Primitive parsers start here. Pick how your item
//! looks like
//! - [Library organization](#library-organization)
//! - [Core Philosophy](#core-philosophy)
//! - Primitive parsers
//!   - [named items: flags, switches, arguments](#named-items-or-options-and-option-arguments)
//!   - [positional items](#positional-items-or-operands)
//! - Functor/Applicative
//! - Parser Composition
//! - Repetition
//! - Composite parsers
//!   - [subcommands and multi value parsers](#subcommands-and-composite-parsers)
//!   - [something else that doesn't quite fit into usual categories](#custom-lexers)
//! - Autocomplete
//! - Custom UI/UX
//! - Testing
//! - Cookbook
//!
//! # Library Organization
//!
//! `bpaf` uses [Fluent interface (Wikipedia)](https://en.wikipedia.org/wiki/Fluent_interface)
//! and parser combinators approach for parsing command line options. You can write
//! them by hand, use a derive macro or mix and match both methods.
//!
//! With combinatoric approach you usually start by [creating](#primitive-parsers)
//! parsers for separate items you want to get from the command line then
//! [combining](#parser-composition) them together using [`construct!`] macro.
//!
//! ```rust
//! use bpaf::*;
//! fn main() {
//!     let alpha = long("alpha").switch();
//!     let beta = long("beta").switch();
//!     let parser = construct!(alpha, beta).to_options();
//!     println!("{:?}", parser.run());
//! }
//! ```
//!
//! Most of the types exposed by the library either implement [`Parser`] trait
//! directly or help to create something that does. Methods on `Parser` trait give
//! back something that implements this trait as well.
//!
//! Documentation for **combinatoric interface** also explains how to achieve the
//! same or similar result using the **derive macro**.
//!
//! When you are implementing your own functions that produce parsers
//! a better approach is to avoid naming return types directly and use
//! "Return position impl Trait" syntax: `impl Parser<Out=MyType>`.
//!
//! # Core Philosophy
//!
//! ## type safety
//! ## composition over complexity
//! ## parse, don't validate
//! ## reusability
//! ## applicative parser with no backtracking
//! ## correctness
//!
//!
//! # Consuming Arguments, Making a Simple [`Parser`]
//!
//! Parsers in *bpaf* consume bits of user input on the command line and produce
//! wrapped values. There are predefined building blocks to parse POSIX style
//! options, option-arguments and operands.
//!
//! ## Named Items or Options and Option-Arguments
//!
//! <div class="code-wrap"><pre>
//! $ cargo <span style="font-weight: bold">--help</span>
//! $ ls <span style="font-weight: bold">-la</span>
//! $ cargo build <span style="font-weight: bold">--bin demo</span>
//! </pre></div>
//!
//! Named items start with a dash (a short name) or two (a long name), followed by
//! a name. Can take a value - for short items it can be immediately adjacent:
//! `-ofile.txt`, for both separated from a name by `=` `-o=file.txt` or
//! `--output=file` or be next in a sequence: `-o file.txt`.
//!
//! To parse one you start with [`short`] or [`long`] to get [`Named`], add more
//! names using [`Named::short`], [`Named::long`] then convert this intermediate
//! representation into a final parser using one of [`Named::flag`],
//! [`Named::argument`], [`Named::switch`] or [`Named::req_flag`]. Named items can
//! have several names - chained with `short` or `long`. Help message will display
//! first short and first long names, but parser will match anything on a list
//!
//! You can also attach a help message with one of `help` methods: [`help`](Named::help),
//! [`help`](Flag::help) or [`help`](Argument::help).
//!
//! Overview of related functions. Actual signatures are slightly different, check their
//! documentation for examples
//!
//! - <code class="code-header">fn [`short`](crate::short)(char) -> [`Named`]</code>
//! - <code class="code-header">fn [`long`](crate::long)(&str) -> [`Named`]</code>
//! - <code class="code-header">fn [`Named::short`](Named::short)(self, char) -> [`Named`]</code>
//! - <code class="code-header">fn [`Named::long`](Named::long)(self, &str) -> [`Named`]</code>
//!
//! - <code class="code-header">fn [`Named::switch`](Named::switch)(self) -> impl [`Parser<bool>`](Parser)</code>,
//!   parser produces `true` if a named item is present, `false` - otherwise.
//! - <code class="code-header">fn [`Named::flag`](Named::flag)`<T>`(self, T, T) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces first `T` if a named item is present, second `T` - otherwise.
//! - <code class="code-header">fn [`Named::req_flag`](Named::req_flag)`<T>`(self, T) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces `T` if a named item is present or
//!   indicates a [missing item] otherwise.
//! - <code class="code-header">fn [`Named::argument`](Named::argument)<T: [`FromStr`]>(self, &str) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces `T` by consuming a value and parsing it according to `FromStr` instance if
//!   a named item is present or indicates a [missing item] otherwise.
//!
//! ## Positional items or operands
//!
//! <div class="code-wrap">
//! <pre>
//! $ cat <span style="font-weight: bold">Cargo.toml</span>
//! </pre>
//! </div>
//!
//! Positional items are pretty much anything else that doesn't start with a dash. *bpaf*
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! - <code class="code-header">fn [`Named::argument`](Named::argument)<T: [`FromStr`]>(self, &str) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces `T` by consuming a value and parsing it according to `FromStr` instance if
//!   a named item is present or indicates a [missing item] otherwise.
//!
//! ## Positional items or operands
//!
//! <div class="code-wrap">
//! <pre>
//! $ cat <span style="font-weight: bold">Cargo.toml</span>
//! </pre>
//! </div>
//!
//! Positional items are pretty much anything else that doesn't start with a dash. *bpaf*
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! - <code class="code-header">fn [`Named::argument`](Named::argument)<T: [`FromStr`]>(self, &str) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces `T` by consuming a value and parsing it according to `FromStr` instance if
//!   a named item is present or indicates a [missing item] otherwise.
//!
//! ## Positional items or operands
//!
//! <div class="code-wrap">
//! <pre>
//! $ cat <span style="font-weight: bold">Cargo.toml</span>
//! </pre>
//! </div>
//!
//! Positional items are pretty much anything else that doesn't start with a dash. *bpaf*
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//!   parser produces `T` if a named item is present or
//!   indicates a [missing item] otherwise.
//! - <code class="code-header">fn [`Named::argument`](Named::argument)<T: [`FromStr`]>(self, &str) -> impl [`Parser<T>`](Parser)</code>,
//!   parser produces `T` by consuming a value and parsing it according to `FromStr` instance if
//!   a named item is present or indicates a [missing item] otherwise.
//!
//! ## Positional items or operands
//!
//! <div class="code-wrap">
//! <pre>
//! $ cat <span style="font-weight: bold">Cargo.toml</span>
//! </pre>
//! </div>
//!
//! Positional items are pretty much anything else that doesn't start with a dash. *bpaf*
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! Positional items are pretty much anything else that doesn't start with a dash. *bpaf*
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! will consume them in order. To create one you simply call [`positional`] and add
//! [`help`](Positional::help)
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//!
//! - <code class="code-header">fn [`positional`](crate::positional)(&str) -> [`Positional`]</code>
//! - <code class="code-header">fn [`Positional::help`](crate::Positional::help)(&str) -> [`Positional`]</code>
//! ## Subcommands and composite parsers
//!
//! <div class="code-wrap">
//! <pre>
//! $ cargo <span style="font-weight: bold">build</span>
//! $ app <span style="font-weight: bold">--set key value</span>
//! </pre>
//! </div>
//!
//!
//! TODO: document [`OptionParser::command`], [`literal`], [`Named::nest`]
//!
//! ## Custom lexers
//!
//! TODO: document [`any`], implement and document [`any_os`], implement and document
//! [`lexed`]
//!
//! [missing item]: crate::api::fallback
//!
//!
//! <div class="code-wrap">
//! <pre>
//! $ ghc <span style="font-weight: bold">+RTS</span>
//! $ dd <span style="font-weight: bold">of=/dev/null</span>
//! </pre>
//! </div>
//! </div>

pub use bpaf_core::*;
pub use bpaf_derive::Bpaf;
