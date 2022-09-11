//! Primitives to define parsers
//!
//! # Terminology
//!
//! ## Flag
//!
//! A simple no-argument command line option that takes no extra parameters, when decoded produces
//! a fixed value. Can have a short (`-f`) or a long (`--flag`) name, see [`Named::flag`] and
//! [`Named::req_flag`]. `bpaf` parses flag into a fixed value known at a compile time.
//!
//! For example `--help` and `-q` are long and short flags accepted by `cargo`
//! ```txt
//! % cargo --help -q
//! ```
//!
//! ## Switch
//!
//! A special case of a flag that gets decoded into a `bool`, see [`Named::switch`].
//!
//! It's possible to represent flags `--help` and `-q` as booleans, `true` for present and `false`
//! for absent.
//! ```txt
//! % cargo --help -q
//! ```
//!
//! ## Argument
//!
//! A command line option with a name that also takes a value. Can have a short (`-f value`) or a
//! long (`--flag value`) name, see [`Named::argument`].
//!
//! For example `rustc` takes a long argument `--explain` with a value containing error code:
//! ```txt
//! % rustc --explain E0571
//! ```
//!
//! ## Positional
//!
//! A positional command with no additonal name, for example in `vim main.rs` `main.rs`
//! is a positional argument. See [`positional`].
//!
//! For example `rustc` takes input as positional argument:
//! ```txt
//! % rustc hello.rs
//! ```
//!
//! ## Command
//!
//! A command defines a starting point for an independent subparser. See [`command`].
//!
//! For example `cargo` contains a command `check` that accepts `--workspace` switch.
//! ```txt
//! % cargo check --workspace
//! ```
//!
use std::{ffi::OsString, marker::PhantomData};

use super::{Args, Error, OptionParser, Parser};
use crate::{
    args::{Arg, Word},
    item::ShortLong,
    Item, Meta,
};

/// A named thing used to create [`flag`](Named::flag), [`switch`](Named::switch) or
/// [`argument`](Named::argument)
///
/// Create it with [`short`] or [`long`].
///
/// # Ways to consume data
/// `bpaf` supports several different ways user can specify values on a command line:
///
/// - [`flag`](Named::flag) - a string that consists of two dashes (`--flag`) and a name and a single
/// dash and a single character (`-f`) - [`long`](Named::long) and [`short`](Named::short) name respectively.
/// Depending on it present or absent from the command line
/// primitive flag parser takes one of two values. User can combine several short flags in a single
/// invocation: `-a -b -c` is the same as `-abc`.
///
/// ```console
/// $ app -a -bc
/// ```
///
/// - [`switch`](Named::switch) - similar to `flag`, but instead of custom values `bpaf` uses `bool`.
/// `switch` mostly serves as a convenient alias to `.flag(true, false)`
///
/// ```console
/// $ app -a -bc
/// ```
///
/// - [`argument`](Named::argument) - a short or long `flag` followed by either a space or `=` and
/// then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all valid argument examples. Note, string
/// literal can't start with `-` unless separated from the flag with `=` and should be valid
/// utf8 only. To consume [`OsString`](std::ffi::OsString) encoded values you can use
/// [`argument_os`](Named::argument_os).
///
/// ```console
/// $ app -o file.txt
/// ```
///
/// - [`positional`] - an arbitrary utf8 string literal (that can't start with `-`) passed on a
/// command line, there's also [`positional_os`] variant that deals with `OsString` named. Usually
/// represents input files such as `cat file.txt`, but can serve other purposes.
///
/// ```console
/// $ cat file.txt
/// ```
///
/// - [`command`] - a fixed utf8 string literal that starts a separate subparser that only
/// gets executed when command name is present. For example `cargo build` invokes
/// command `"build"` and after `"build"` `cargo` starts accepting values it won't accept otherwise
///
/// ```console
/// $ cargo build --out-dir my_target
/// // works since command "build" supports --out-dir argument
/// $ cargo check --out-dir my_target
/// // fails since --out-dir isn't a valid argument for command "check"
/// ```
///
/// As most of the other parsers `bpaf` treats anything to the right of `--` symbol as positional
/// arguments regardless their names:
///
/// ```console
/// $ app -o file.txt -- --those --are --positional --items
/// ```
///
/// # Combinatoric usage
///
/// Named items (`argument`, `flag` and `switch`) can have up to 2 visible names (short and long)
/// and multiple hidden short and long aliases if needed. It's also possible to consume items from
/// environment variables using [`env`](Named::env). You usually start with [`short`] or [`long`]
/// function, then apply [`short`](Named::short) / [`long`](Named::long) / [`env`](Named::env) /
/// [`help`](Named::help) repeatedly to build a desired set of names then transform it into
/// a parser using `flag`, `switch` or `positional`.
///
/// ```rust
/// # use bpaf::*;
/// fn an_item() -> impl Parser<String> {
///     short('i')
///         .long("item")
///         .long("also-item") // but hidden
///         .env("ITEM")
///         .help("A string used by this example")
///         .argument("ITEM")
/// }
/// ```
/// # Example
/// ```console
/// $ app --help
///     <skip>
///     -i --item <ITEM>  [env:ITEM: N/A]
///                       A string used by this example
///     <skip>
/// ```
///
/// # Derive usage
///
/// Unlike combinatoric API where you forced to specify names for your parsers derive API allows
/// to omit some or all the details:
/// 1. If no naming information is present at all - `bpaf_derive` would use field name as a long name
///    (or a short name if field name consists of a single character)
/// 2. If `short` or `long` annotation is present without an argument - `bpaf_derive` would use first character
///    or a full name as long and short name respectively. It won't try to add implicit long or
///    short name from the previous item.
/// 3. If `short` or `long` annotation is present with an argument - those are values `bpaf_derive` would
///    use instead of the original field name
/// 4. If `env(arg)` annotation is present - `bpaf_derive` would generate `.env(arg)` method:
///
///    ```rust
///    # use bpaf::*;
///    const DB: &str = "top_secret_database";
///
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    pub struct Config {
///       /// flag with no annotation
///       pub flag_1: bool,
///
///       /// explicit short suppresses long
///       #[bpaf(short)]
///       pub flag_2: bool,
///
///       /// explicit short with custom letter
///       #[bpaf(short('z'))]
///       pub flag_3: bool,
///
///       /// explicit short and long
///       #[bpaf(short, long)]
///       pub deposit: bool,
///
///       /// implicit long + env variable from DB constant
///       #[bpaf(env(DB))]
///       pub database: String,
///
///       /// implicit long + env variable "USER"
///       #[bpaf(env("USER"))]
///       pub user: String,
///    }
///    ```
///
/// # Example
/// ```console
/// $ app --help
///    <skip>
///         --flag-1         flag with no annotation
///    -f                    explicit short suppresses long
///    -z                    explicit short with custom letter
///    -d, --deposit         explicit short and long
///        --database <ARG>  [env:top_secret_database: N/A]
///                          implicit long + env variable from DB constant
///        --user <ARG>      [env:USER = "pacak"]
///                          implicit long + env variable "USER"
///    <skip>
/// ```
#[derive(Clone, Debug)]
pub struct Named {
    pub(crate) short: Vec<char>,
    pub(crate) long: Vec<&'static str>,
    env: Vec<&'static str>,
    pub(crate) help: Option<String>,
}

impl Named {
    pub(crate) fn flag_item(&self) -> Item {
        Item::Flag {
            name: ShortLong::from(self),
            help: self.help.clone(),
        }
    }
}

/// A flag/switch/argument that has a short name
///
/// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
///
/// ```rust
/// # use bpaf::*;
/// fn parse_bool() -> impl Parser<bool> {
///     short('f')
///         .short('F')
///         .long("flag")
///         .help("a flag that does a thing")
///         .switch()
/// }
/// ```
/// See [`Named`] for more details
#[must_use]
pub fn short(short: char) -> Named {
    Named {
        short: vec![short],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    }
}

/// A flag/switch/argument that has a long name
///
/// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
///
/// ```rust
/// # use bpaf::*;
/// fn parse_bool() -> impl Parser<bool> {
///     short('f')
///         .long("flag")
///         .long("Flag")
///         .help("a flag that does a thing")
///         .switch()
/// }
/// ```
/// See [`Named`] for more details
#[must_use]
pub fn long(long: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: vec![long],
        env: Vec::new(),
        help: None,
    }
}

/// Environment variable fallback
///
/// If named value isn't present - try to fallback to this environment variable.
///
/// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
///
/// For [`flag`](Named::flag) and [`switch`](Named::switch) environment variable being present
/// gives the same result as the flag being present, allowing to implement things like `NO_COLOR`
/// variables:
///
/// ```console
/// $ NO_COLOR=1 app --do-something
/// ```
///
/// # Combinatoric usage
/// **You must specify either short or long key if you start the chain from `env`.**
///
/// ```rust
/// # use bpaf::*;
/// fn parse_string() -> impl Parser<String> {
///     short('k')
///         .long("key")
///         .env("API_KEY")
///         .help("Use this API key to access the API")
///         .argument("KEY")
/// }
/// ```
///
/// # Derive usage
/// `enum` annotation takes a string literal or an expression of type `&'static str`.
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     /// Use this API key to access the API
///     #[bpaf(short, long, env("API_KEY"))]
///     key: String,
/// }
/// ```
///
/// # Example
/// ```console
/// $ app --help
///     --key <KEY>  [env:ACCESS_KEY: N/A]
///                  access key to use
/// $ app
/// // fails due to missing --key argument
/// $ app --key SECRET
/// // "SECRET"
/// $ KEY=TOP_SECRET app
/// // "TOP_SECRET"
/// $ KEY=TOP_SECRET app --key SECRET
/// // "SECRET" - argument takes a priority
/// ```
/// See [`Named`] for more details
#[must_use]
pub fn env(variable: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: Vec::new(),
        help: None,
        env: vec![variable],
    }
}

impl Named {
    /// Add a short name to a flag/switch/argument
    ///
    /// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_bool() -> impl Parser<bool> {
    ///     short('f')
    ///         .short('F')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch()
    /// }
    /// ```
    /// See [`Named`] for more details
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }

    /// Add a long name to a flag/switch/argument
    ///
    /// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_bool() -> impl Parser<bool> {
    ///     short('f')
    ///         .long("flag")
    ///         .long("Flag")
    ///         .help("a flag that does a thing")
    ///         .switch()
    /// }
    /// ```
    /// See [`Named`] for more details
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
        self
    }

    /// Environment variable fallback
    ///
    /// If named value isn't present - try to fallback to this environment variable.
    /// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_string() -> impl Parser<String> {
    ///     short('k')
    ///         .long("key")
    ///         .env("API_KEY")
    ///         .help("Use this API key to access the API")
    ///         .argument("KEY")
    /// }
    /// ```
    /// See [`Named`] and [`env`](env()) for more details and examples
    #[must_use]
    pub fn env(mut self, variable: &'static str) -> Self {
        self.env.push(variable);
        self
    }

    /// Add a help message to a flag/switch/argument
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_bool() -> impl Parser<bool> {
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch()
    /// }
    /// ```
    ///
    /// # Derive usage
    /// `bpaf_derive` converts doc comments into option help by following those rules:
    /// 1. It skips blank lines, if present.
    /// 2. It stops parsing after a double blank line.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     /// This line is part of help message
    ///     ///
    ///     /// So is this one
    ///     ///
    ///     ///
    ///     /// But this one isn't
    ///     key: String,
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }

    /// Simple boolean flag
    ///
    /// Parser produces `true` if flag is present in a command line or `false` otherwise
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_bool() -> impl Parser<bool> {
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch()
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn switch(self) -> impl Parser<bool> {
        build_flag_parser(true, Some(false), self)
    }

    /// Flag with custom present/absent values
    ///
    /// # Combinatoric usage
    /// Parser produces `present` if flag is present in a command line or `absent` otherwise
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Clone)]
    /// enum Flag {
    ///     Absent,
    ///     Present,
    /// }
    /// fn parse_flag() -> impl Parser<Flag> {
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .flag(Flag::Present, Flag::Absent)
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// Currently available only with `external` annotation
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone)]
    /// enum Flag {
    ///     Absent,
    ///     Present,
    /// }
    ///
    /// fn flag() -> impl Parser<Flag> {
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .flag(Flag::Present, Flag::Absent)
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(external)]
    ///     pub flag: Flag,
    /// }
    /// ```
    ///
    #[must_use]
    /// See [`Named`] for more details
    pub fn flag<T>(self, present: T, absent: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self)
    }

    /// Required flag with custom value
    ///
    /// Similar to [`flag`](Named::flag) takes no option arguments, but will only
    /// succeed if user specifies it on a command line.
    /// Not very useful by itself and works best in combination with other parsers.
    /// ## Using `req_flag` to implement 3-state options.
    ///
    /// In derive mode `bpaf` would transform field-less enum variants into `req_flag`
    /// In addition to naming annotations (`short`, `long` and `env`) such variants also
    /// accepts `hide` and `default` annotations. Former hides it from `--help` (see
    /// [`hide`](Parser::hide), later makes it a default choice if preceeding variants
    /// fail to parse. You shoud only use `default` annotation on the last variant of
    /// enum. To better convey the meaning you might want to use a combination of
    /// `skip` and `fallback` annotations, see examples.
    ///
    ///
    /// See [`Named`] for more details
    #[doc = include_str!("docs/req_flag.md")]
    #[must_use]
    pub fn req_flag<T>(self, present: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, None, self)
    }

    /// Named argument in utf8 (String) encoding
    ///
    /// Argument must contain only valid utf8 characters.
    /// For OS specific encoding see [`argument_os`][Named::argument_os].
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_string() -> impl Parser<String> {
    ///     short('n')
    ///         .long("name")
    ///         .argument("NAME")
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// `bpaf_derive` would automatically pick between `argument` and
    /// [`argument_os`](Named::argument_os) depending on
    /// a field type but you can specify it manually to override the metavar value
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short('n'), argument("NAME"))]
    ///     name: String,
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn argument(self, metavar: &'static str) -> impl Parser<String> {
        build_argument(self, metavar).parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    /// Named argument in OS specific encoding
    ///
    /// If you prefer to panic on non utf8 encoding see [`argument`][Named::argument].
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_osstring() -> impl Parser<std::ffi::OsString> {
    ///     short('n')
    ///         .long("name")
    ///         .argument_os("NAME")
    /// }
    /// ```
    ///
    ///
    /// # Derive usage
    /// `bpaf_derive` would automatically pick between [`argument`](Named::argument) and
    /// `argument_os` depending on
    /// a field type but you can specify it manually to override the metavar value
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short('n'), argument_os("NAME"))]
    ///     name: std::ffi::OsString,
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn argument_os(self, metavar: &'static str) -> impl Parser<OsString> {
        build_argument(self, metavar).map(|x| x.os)
    }

    pub(crate) fn matches_arg(&self, arg: &Arg) -> bool {
        match arg {
            Arg::Short(s, _) => self.short.contains(s),
            Arg::Long(l, _) => self.long.contains(&l.as_str()),
            Arg::Word(_) => false,
        }
    }
}

/// Positional argument in utf8 (`String`) encoding
///
/// For named flags and arguments ordering generally doesn't matter: most programs would
/// understand `-O2 -v` the same way as `-v -O2`, but for positional items order matters: in unix
/// `cat hello world` and `cat world hello` would display contents of the same two files but in
/// different order.
///
/// # Important restriction
/// To parse positional arguments from a command line you should place parsers for all your
/// named values before parsers for positional items and commands. In derive API fields parsed as
/// positional items or commands should be at the end of your `struct`/`enum`. Same rule applies
/// to parsers with positional fields or commands inside: such parsers should go to the end as well.
///
/// Use [`check_invariants`](OptionParser::check_invariants) in your test to ensure correctness.
///
/// For example for non positional `non_pos` and positional `pos` parsers
/// ```rust
/// # use bpaf::*;
/// # let non_pos = || short('n').switch();
/// # let pos = ||positional("POS");
/// let valid = construct!(non_pos(), pos());
/// let invalid = construct!(pos(), non_pos());
/// ```
///
/// **`bpaf` panics during help generation unless if this restriction holds**
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn input() -> impl Parser<String> {
///     positional("INPUT")
/// }
/// ```
///
/// # Derive usage
///
/// `bpaf_derive` converts fields in tuple-like structures into positional items
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options(String);
/// ```
/// `positional` and `positional_os` annotations also accept an optional metavar name
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     #[bpaf(positional("INPUT"))]
///     input: String,
/// }
/// ```
///
/// See also [`positional_os`] - a simiar function
#[must_use]
pub fn positional(metavar: &'static str) -> Positional<String> {
    build_positional(metavar)
}

/// Positional argument in OS specific encoding
///
/// For named flags and arguments ordering generally doesn't matter: most programs would
/// understand `-O2 -v` the same way as `-v -O2`, but for positional items order matters: in unix
/// `cat hello world` and `cat world hello` would display contents of the same two files but in
/// different order.
///
/// # Important restriction
/// To parse positional arguments from a command line you should place parsers for all your
/// named values before parsers for positional items and commands. In derive API fields parsed as
/// positional items or commands should be at the end of your `struct`/`enum`. Same rule applies
/// to parsers with positional fields or commands inside: such parsers should go to the end as well.
///
/// Use [`check_invariants`](OptionParser::check_invariants) in your test to ensure correctness.
///
/// For example for non positional `non_pos` and positional `pos` parsers
/// ```rust
/// # use bpaf::*;
/// # let non_pos = || short('n').switch();
/// # let pos = ||positional("POS");
/// let valid = construct!(non_pos(), pos());
/// let invalid = construct!(pos(), non_pos());
/// ```
///
/// **`bpaf` panics during help generation unless if this restriction holds**
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn input() -> impl Parser<std::ffi::OsString> {
///     positional_os("INPUT")
/// }
/// ```

/// # Derive usage
///
/// `bpaf_derive` converts fields in tuple-like structures into positional items and automatically
/// uses `positional_os` for `OsString` and `PathBuf`
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options(std::ffi::OsString);
/// ```
/// `positional` and `positional_os` annotations also accept an optional metavar name
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     #[bpaf(positional_os("INPUT"))]
///     input: std::path::PathBuf,
/// }
/// ```
///
/// See also [`positional_os`] - a simiar function
#[must_use]
pub fn positional_os(metavar: &'static str) -> Positional<OsString> {
    build_positional(metavar)
}

/// Subcommand parser
///
/// Subcommands allow to use a totally independent parser inside a current one. Inner parser
/// can have its own help message, description, version and so on. You can nest them arbitrarily
/// too.
///
/// Alternatively you can create commands using [`command`](OptionParser::command)
///
/// # Important restriction
/// When parsing command arguments from command lines you should have parsers for all your
/// named values and command before parsers for positional items. In derive API fields parsed as
/// positional should be at the end of your `struct`/`enum`. Same rule applies
/// to parsers with positional fields or commands inside: such parsers should go to the end as well.
///
/// Use [`check_invariants`](OptionParser::check_invariants) in your test to ensure correctness.
///
/// For example for non positional `non_pos` and a command `command` parsers
/// ```rust
/// # use bpaf::*;
/// # let non_pos = || short('n').switch();
/// # let command = || pure(()).to_options().command("POS");
/// let valid = construct!(non_pos(), command());
/// let invalid = construct!(command(), non_pos());
/// ```
///
/// **`bpaf` panics during help generation unless if this restriction holds**
///
/// # Combinatoric use
///
/// Structure [`Command`] you get by calling this method is a builder that allows to add additional
/// aliases with [`short`](Command::short), [`long`](Command::long) (only first short and first
/// long names are visible to `--help`) and override [`help`](Command::help). Without help override
/// bpaf would use first line from the description
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone)]
/// enum Cmd {
///     Check {
///         workspace: bool,
///     }
/// };
///
/// // First of all you need an inner parser
/// fn check_workspace() -> OptionParser<Cmd> {
///     // Define a parser to use in a subcommand in a usual way.
///     // This parser accepts a single --workspace switch
///     let workspace = long("workspace")
///         .help("Check all packages in the workspace")
///         .switch();
///
///     // and attach some meta information to it in a usual way
///     construct!(Cmd::Check { workspace })
///         .to_options()
///         // description to use for command's help
///         .descr("Check a package for errors")
/// }
///
/// // Convert subparser into a parser.
/// fn check_workspace_command() -> impl Parser<Cmd> {
///     command("check", check_workspace())
///         // help to use to list the command
///         .help("Check a package command")
/// }
/// ```
///
/// # Derive usage
/// ```rust
/// # use bpaf::*;
/// #[derive(Clone, Debug, Bpaf)]
/// enum Cmd {
///     #[bpaf(command)]
///     /// Check a package command
///     Check {
///         /// Check all the packages in the workspace
///         workspace: bool
///     }
/// }
/// ```
///
/// # Example
/// ```console
/// $ app --help
/// // displays global help, not listed in this example
/// $ app check --help
/// // displays help for check: "Check a package command"
/// $ app check
/// // Cmd::Check(CheckWorkspace(false))
/// $ app check --workspace
/// // Cmd::Check(CheckWorkspace(true))
/// ```
///
#[must_use]
pub fn command<T>(name: &'static str, subparser: OptionParser<T>) -> Command<T>
where
    T: 'static,
{
    Command {
        longs: vec![name],
        shorts: Vec::new(),
        help: subparser.short_descr().map(Into::into),
        subparser,
    }
}

/// Builder structure for the [`command`]
///
/// Created with [`command`], implements parser for the inner structure, gives access to [`help`](Command::help).
pub struct Command<T> {
    longs: Vec<&'static str>,
    shorts: Vec<char>,
    help: Option<String>,
    subparser: OptionParser<T>,
}

impl<P> Command<P> {
    /// Add a brief description to a command
    ///
    /// `bpaf` uses this description along with the command name
    /// in help output so it shouldn't exceed one or two lines. If `help` isn't specified
    /// `bpaf` falls back to [`descr`](OptionParser::descr) from the inner parser.
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn inner() -> OptionParser<bool> {
    ///     short('i')
    ///         .help("Mysterious inner switch")
    ///         .switch()
    ///         .to_options()
    ///         .descr("performs an operation")
    /// }
    ///
    /// fn mysterious_parser() -> impl Parser<bool> {
    ///     command("mystery", inner())
    ///         .help("This command performs a mystery operation")
    /// }
    /// ```
    ///
    /// # Derive usage
    /// `bpaf_derive` uses doc comments for inner parser, no specific options are available.
    /// See [`descr`](OptionParser::descr) for more details
    /// ```rust
    /// # use bpaf::*;
    /// /// This command performs a mystery operation
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(command)]
    /// struct Mystery {
    ///     #[bpaf(short)]
    ///     /// Mysterious inner switch
    ///     inner: bool,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --help
    ///     <skip>
    /// Available commands:
    ///     mystery  This command performs a mystery operation
    /// ```
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }

    /// Add a custom short alias for a command
    ///
    /// Behavior is similar to [`short`](Named::short), only first short name is visible.
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.shorts.push(short);
        self
    }

    /// Add a custom hidden long alias for a command
    ///
    /// Behavior is similar to [`long`](Named::long), but since you had to specify the first long
    /// name when making the command - this one becomes a hidden alias.
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.longs.push(long);
        self
    }
}

impl<T> Parser<T> for Command<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        // used to avoid allocations for short names
        let mut tmp = String::new();
        if self.longs.iter().any(|long| args.take_cmd(long))
            || self.shorts.iter().any(|s| {
                tmp.clear();
                tmp.push(*s);
                args.take_cmd(&tmp)
            })
        {
            #[cfg(feature = "autocomplete")]
            if args.touching_last_remove() {
                // in completion mode prefer to autocomplete the command name vs going inside the
                // parser
                args.clear_comps();
                args.push_command(self.longs[0], self.shorts.first().copied(), &self.help);
                return Err(Error::Missing(Vec::new()));
            }

            args.head = usize::MAX;
            args.depth += 1;
            // `or_else` would prefer failures past this point to preceeding levels
            #[allow(clippy::let_and_return)]
            let res = self.subparser.run_subparser(args);
            res
        } else {
            #[cfg(feature = "autocomplete")]
            args.push_command(self.longs[0], self.shorts.first().copied(), &self.help);

            Err(Error::Missing(vec![self.item()]))
        }
    }

    fn meta(&self) -> Meta {
        Meta::Item(self.item())
    }
}

impl<T> Command<T> {
    fn item(&self) -> Item {
        Item::Command {
            name: self.longs[0],
            short: self.shorts.first().copied(),
            help: self.help.clone(),
            meta: Box::new(self.subparser.inner.meta()),
        }
    }
}

fn build_flag_parser<T>(present: T, absent: Option<T>, named: Named) -> impl Parser<T>
where
    T: Clone + 'static,
{
    BuildFlagParser {
        present,
        absent,
        named,
    }
}

#[derive(Clone)]
struct BuildFlagParser<T> {
    present: T,
    absent: Option<T>,
    named: Named,
}

impl<T: Clone + 'static> Parser<T> for BuildFlagParser<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        if args.take_flag(&self.named) || self.named.env.iter().find_map(std::env::var_os).is_some()
        {
            #[cfg(feature = "autocomplete")]
            if args.touching_last_remove() {
                args.push_flag(&self.named);
            }
            Ok(self.present.clone())
        } else {
            #[cfg(feature = "autocomplete")]
            args.push_flag(&self.named);
            match &self.absent {
                Some(ok) => Ok(ok.clone()),
                None => Err(Error::Missing(vec![self.named.flag_item()])),
            }
        }
    }

    fn meta(&self) -> Meta {
        self.named.flag_item().required(self.absent.is_none())
    }
}

fn build_argument(named: Named, metavar: &'static str) -> impl Parser<Word> {
    if !named.env.is_empty() {
        // mostly cosmetic reasons
        assert!(
            !(named.short.is_empty() && named.long.is_empty()),
            "env fallback can only be used if name is present"
        );
    }
    BuildArgument { named, metavar }
}

#[derive(Clone)]
struct BuildArgument {
    named: Named,
    metavar: &'static str,
}

impl BuildArgument {
    fn item(&self) -> Item {
        Item::Argument {
            name: ShortLong::from(&self.named),
            metavar: self.metavar,
            env: self.named.env.first().copied(),
            help: self.named.help.clone(),
        }
    }
}

impl Parser<Word> for BuildArgument {
    fn eval(&self, args: &mut Args) -> Result<Word, Error> {
        match args.take_arg(&self.named) {
            Ok(Some(w)) => {
                #[cfg(feature = "autocomplete")]
                if args.touching_last_remove() {
                    args.push_metadata(self.metavar, &self.named.help, true);
                }
                Ok(w)
            }
            Err(err) => {
                #[cfg(feature = "autocomplete")]
                args.push_argument(&self.named, self.metavar);
                Err(err)
            }
            _ => {
                #[cfg(feature = "autocomplete")]
                args.push_argument(&self.named, self.metavar);
                if let Some(val) = self.named.env.iter().find_map(std::env::var_os) {
                    args.current = None;
                    Ok(crate::args::word(val, false))
                } else {
                    Err(Error::Missing(vec![self.item()]))
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        Meta::Item(self.item())
    }
}

fn build_positional<T>(metavar: &'static str) -> Positional<T> {
    Positional {
        metavar,
        help: None,
        result_type: PhantomData,
        strict: false,
    }
}

/// Parse a positional item, created with [`positional`] and [`positional_os`]
///
/// You can add extra information to positional parsers with [`help`](Positional::help) and
/// [`strict`](Positional::strict) on this struct.
#[derive(Clone)]
pub struct Positional<T> {
    metavar: &'static str,
    help: Option<String>,
    result_type: PhantomData<T>,
    strict: bool,
}

impl<T> Positional<T> {
    /// Add a help message to a [`positional`]/[`positional_os`] parser
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_name() -> impl Parser<String> {
    ///     positional("NAME")
    ///         .help("a flag that does a thing")
    /// }
    /// ```
    ///
    /// # Derive usage
    /// `bpaf_derive` converts doc comments into option help by following those rules:
    /// 1. It skips blank lines, if present.
    /// 2. It stops parsing after a double blank line.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options (
    ///     /// This line is part of help message
    ///     ///
    ///     /// So is this one
    ///     ///
    ///     ///
    ///     /// But this one isn't
    ///     String,
    /// );
    /// ```
    /// See also [`Named::help`]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }

    /// Changes positional parser to be "strict" positional
    ///
    /// Usually positional items can appear anywhere on a command line:
    /// ```console
    /// $ ls -d bpaf
    /// $ ls bpaf -d
    /// ```
    /// here `ls` takes a positional item `bpaf` and a flag `-d`
    ///
    /// But in some cases it might be useful to have a stricter separation between
    /// positonal items and flags, such as passing arguments to a subprocess:
    /// ```console
    /// $ cargo run --example simple -- --help
    /// ```
    /// here `cargo` takes a `--help` as a positional item and passes it to the example
    ///
    /// `bpaf` allows to require user to pass `--` for positional items with `strict` annotation.
    /// `bpaf` would display such positional elements differently in usage line as well. If your
    /// application requires several different strict positional elements - it's better to place
    /// this annotation only to the first one.
    ///
    /// # Example
    /// Usage line for a cargo-run like app that takes an application and possibly many strictly
    /// positional child arguments can look like this:
    /// ```console
    /// $ app --help
    /// Usage: [-p SPEC] [[--bin NAME] | [--example NAME]] [--release] [<BIN>] -- <CHILD_ARG>...
    /// <skip>
    /// ```
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn options() -> impl Parser<Vec<std::ffi::OsString>> {
    ///     positional_os("OPTS")
    ///         .strict()
    ///         .many()
    /// }
    /// ```
    ///
    /// # Derive usage
    /// Not available at the moment
    #[must_use]
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    fn meta(&self) -> Meta {
        Meta::Item({
            Item::Positional {
                metavar: self.metavar,
                help: self.help.clone(),
                strict: self.strict,
            }
        })
    }
}

fn parse_word(
    args: &mut Args,
    strict: bool,
    metavar: &'static str,
    help: &Option<String>,
) -> Result<Word, Error> {
    match args.take_positional_word()? {
        Some(word) => {
            if strict && !word.pos_only {
                #[cfg(feature = "autocomplete")]
                args.push_value("--", &Some("-- Positional only items".to_owned()), false);

                return Err(Error::Stderr(format!(
                    "Expected <{}> to be on the right side of --",
                    metavar,
                )));
            }

            #[cfg(feature = "autocomplete")]
            if args.touching_last_remove() && !args.no_pos_ahead {
                args.push_metadata(metavar, help, false);
                args.no_pos_ahead = true;
            }
            Ok(word)
        }
        None => {
            #[cfg(feature = "autocomplete")]
            if !args.no_pos_ahead {
                args.push_metadata(metavar, help, false);
                args.no_pos_ahead = true;
            }

            let item = Item::Positional {
                metavar,
                help: help.clone(),
                strict,
            };
            Err(Error::Missing(vec![item]))
        }
    }
}

impl Parser<OsString> for Positional<OsString> {
    fn eval(&self, args: &mut Args) -> Result<OsString, Error> {
        let res = parse_word(args, self.strict, self.metavar, &self.help)?;
        Ok(res.os)
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}

impl Parser<String> for Positional<String> {
    fn eval(&self, args: &mut Args) -> Result<String, Error> {
        let res = parse_word(args, self.strict, self.metavar, &self.help)?;
        match res.utf8 {
            Some(ok) => Ok(ok),
            None => Err(Error::Stderr(format!(
                "<{}> is not a valid utf",
                self.metavar
            ))),
        }
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}

pub struct GetAny<T> {
    ty: PhantomData<T>,
    metavar: &'static str,
    strict: bool,
    help: Option<String>,
}

pub fn any(metavar: &'static str) -> GetAny<OsString> {
    GetAny {
        ty: PhantomData,
        metavar,
        strict: false,
        help: None,
    }
}

impl<T> GetAny<T> {
    pub fn str(self) -> GetAny<String> {
        GetAny {
            ty: PhantomData,
            metavar: self.metavar,
            help: self.help,
            strict: self.strict,
        }
    }
    pub fn help<M: ToString>(mut self, help: M) -> Self {
        self.help = Some(help.to_string());
        self
    }

    fn meta(&self) -> Meta {
        Meta::Item(self.item())
    }

    fn item(&self) -> Item {
        Item::Positional {
            metavar: self.metavar,
            strict: self.strict,
            help: self.help.clone(),
        }
    }

    /// returns real items only
    fn next_os_string(&self, args: &mut Args) -> Result<OsString, Error> {
        let (ix, item) = match args.items_iter().next() {
            Some(item_with_index) => item_with_index,
            None => return Err(Error::Missing(vec![self.item()])),
        };
        let os = item.as_os().to_owned();
        args.remove(ix);
        if os.is_empty() {
            todo!("explain error here");
        } else {
            Ok(os)
        }
    }
}

impl Parser<String> for GetAny<String> {
    fn eval(&self, args: &mut Args) -> Result<String, Error> {
        let os = self.next_os_string(args)?;
        match os.to_str() {
            Some(s) => Ok(s.to_owned()),
            None => Err(Error::Stderr(format!(
                "Can't consume {} as String: {} contains non-utf8 characters",
                self.item(),
                os.to_string_lossy(),
            ))),
        }
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}

impl Parser<OsString> for GetAny<OsString> {
    fn eval(&self, args: &mut Args) -> Result<OsString, Error> {
        self.next_os_string(args)
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}
