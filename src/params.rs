//! Tools to define primitive parsers
//!
//! # Ways to consume data
//!
//! ## Flag
//!
//! - [`flag`](NamedArg::flag) - a string that consists of two dashes (`--flag`) and a name and a single
//! dash and a single character (`-f`) created with [`long`](NamedArg::long) and [`short`](NamedArg::short)
//! respectively. Depending if this name is present or absent on the command line
//! primitive flag parser produces one of two values. User can combine several short flags in a single
//! invocation: `-a -b -c` is the same as `-abc`.
//!
#![doc = include_str!("docs/flag.md")]
//!
//! ## Required flag
//!
//! Similar to `flag`, but instead of falling back to the second value required flag parser would
//! fail. Mostly useful in combination with other parsers, created with [`NamedArg::req_flag`].
//!
#![doc = include_str!("docs/req_flag.md")]
//!
//! ## Switch
//!
//! A special case of a flag that gets decoded into a `bool`, mostly serves as a convenient
//! shortcut to `.flag(true, false)`. Created with [`NamedArg::switch`].
//!
#![doc = include_str!("docs/switch.md")]
//!
//! ## Argument
//!
//! A short or long `flag` followed by either a space or `=` and
//! then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all valid argument examples. Note, string
//! literal can't start with `-` unless separated from the flag with `=`. For short flags value
//! can follow immediately: `-fbar`.
//!
#![doc = include_str!("docs/argument.md")]
//!
//! ## Positional
//!
//! A positional argument with no additonal name, for example in `vim main.rs` `main.rs`
//! is a positional argument. Can't start with `-`, created with [`positional`].
//!
#![doc = include_str!("docs/positional.md")]
//!
//! ## Any
//!
//! Also a positional argument with no additional name, but unlike [`positional`] itself, [`any`]
//! isn't restricted to positional looking structure and would consume any items as they appear on
//! a command line. Can be useful to collect anything unused to pass to other applications.
//!
#![doc = include_str!("docs/any.md")]
//!
//! ## Command
//!
//! A command defines a starting point for an independent subparser. Name must be a valid utf8
//! string. For example `cargo build` invokes command `"build"` and after `"build"` `cargo`
//! starts accepting values it won't accept otherwise
//!
#![doc = include_str!("docs/command.md")]
//!
use std::{ffi::OsString, marker::PhantomData, str::FromStr};

use super::{Args, Error, OptionParser, Parser};
use crate::{
    args::Arg, from_os_str::parse_os_str, item::ShortLong, meta_help::Metavar, Item, Meta,
};

/// A named thing used to create [`flag`](NamedArg::flag), [`switch`](NamedArg::switch) or
/// [`argument`](NamedArg::argument)
///
/// Named items (`argument`, `flag` and `switch`) can have up to 2 visible names (one short and one long)
/// and multiple hidden short and long aliases if needed. It's also possible to consume items from
/// environment variables using [`env`](NamedArg::env). You usually start with [`short`] or [`long`]
/// function, then apply [`short`](NamedArg::short) / [`long`](NamedArg::long) / [`env`](NamedArg::env) /
/// [`help`](NamedArg::help) repeatedly to build a desired set of names then transform it into
/// a parser using `flag`, `switch` or `positional`.
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
pub struct NamedArg {
    pub(crate) short: Vec<char>,
    pub(crate) long: Vec<&'static str>,
    env: Vec<&'static str>,
    pub(crate) help: Option<String>,
}

impl NamedArg {
    pub(crate) fn flag_item(&self) -> Item {
        Item::Flag {
            name: ShortLong::from(self),
            help: self.help.clone(),
            env: self.env.first().copied(),
            shorts: self.short.clone(),
        }
    }
}

/// A flag/switch/argument that has a short name
///
/// You can specify it multiple times, `bpaf` would use items past the first of each `short` and `long` as
/// hidden aliases.
///
#[doc = include_str!("docs/short_long_env.md")]
#[must_use]
pub fn short(short: char) -> NamedArg {
    NamedArg {
        short: vec![short],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    }
}

/// A flag/switch/argument that has a long name
///
/// You can specify it multiple times, `bpaf` would use items past the first of each `short` and `long` as
/// hidden aliases.
///
#[doc = include_str!("docs/short_long_env.md")]
#[must_use]
pub fn long(long: &'static str) -> NamedArg {
    NamedArg {
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
/// For [`flag`](NamedArg::flag) and [`switch`](NamedArg::switch) environment variable being present
/// gives the same result as the flag being present, allowing to implement things like `NO_COLOR`
/// variables:
///
/// ```console
/// $ NO_COLOR=1 app --do-something
/// ```
///
/// If you don't specify a short or a long name - whole argument is going to be absent from the
/// help message. Use it combined with a named or positional argument to have a hidden fallback
/// that wouldn't leak sensitive info.

///
#[doc = include_str!("docs/short_long_env.md")]
#[must_use]
pub fn env(variable: &'static str) -> NamedArg {
    NamedArg {
        short: Vec::new(),
        long: Vec::new(),
        help: None,
        env: vec![variable],
    }
}

impl NamedArg {
    /// Add a short name to a flag/switch/argument
    ///
    #[doc = include_str!("docs/short_long_env.md")]
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }

    /// Add a long name to a flag/switch/argument
    ///
    #[doc = include_str!("docs/short_long_env.md")]
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
        self
    }

    /// Environment variable fallback
    ///
    /// If named value isn't present - try to fallback to this environment variable.
    ///
    /// You can specify it multiple times, `bpaf` would use items past the first one as hidden aliases.
    ///
    /// For [`flag`](NamedArg::flag) and [`switch`](NamedArg::switch) environment variable being present
    /// gives the same result as the flag being present, allowing to implement things like `NO_COLOR`
    /// variables:
    ///
    /// ```console
    /// $ NO_COLOR=1 app --do-something
    /// ```
    #[doc = include_str!("docs/short_long_env.md")]
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
    /// See [`NamedArg`] for more details
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }

    /// Simple boolean flag
    ///
    /// A special case of a [`flag`](NamedArg::flag) that gets decoded into a `bool`, mostly serves as a convenient
    /// shortcut to `.flag(true, false)`.
    ///
    #[doc = include_str!("docs/switch.md")]
    #[must_use]
    /// See [`NamedArg`] for more details
    pub fn switch(self) -> impl Parser<bool> {
        build_flag_parser(true, Some(false), self)
    }

    /// Flag with custom present/absent values
    ///
    /// More generic version of [`switch`](NamedArg::switch) that uses arbitrary type instead of
    /// [`bool`].
    #[doc = include_str!("docs/flag.md")]
    ///
    #[must_use]
    pub fn flag<T>(self, present: T, absent: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self)
    }

    /// Required flag with custom value
    ///
    /// Similar to [`flag`](NamedArg::flag) takes no option arguments, but would only
    /// succeed if user specifies it on a command line.
    /// Not very useful by itself and works best in combination with other parsers.
    ///
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
    /// Additionally `bpaf_derive` handles `()` fields as `req_flag` see
    /// [`adjacent`](Parser::adjacent) for more details.
    /// See [`NamedArg`] for more details
    #[doc = include_str!("docs/req_flag.md")]
    #[must_use]
    pub fn req_flag<T>(self, present: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, None, self)
    }

    /// Argument
    ///
    /// A short (`-a`) or long (`--name`) name followed by  either a space or `=` and
    /// then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all valid argument examples. Note, string
    /// literal can't start with `-` unless separated from the flag with `=`. For short flags value
    /// can follow immediately: `-fbar`.
    ///
    /// When using combinatoring API you can specify the type with turbofish, for parsing types
    /// that don't implement [`FromStr`] you can use consume a `String`/`OsString` first and parse
    /// it by hands.
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_arg() -> impl Parser<usize> {
    ///     short('a').argument::<usize>("ARG")
    /// }
    /// ```
    #[doc = include_str!("docs/argument.md")]
    #[must_use]
    pub fn argument<T>(self, metavar: &'static str) -> ParseArgument<T>
    where
        T: FromStr + 'static,
    {
        build_argument(self, metavar)
    }

    #[track_caller]
    /// `adjacent` requires for the argument to be present in the same word as the flag:
    /// `-f bar` - no, `-fbar` or `-f=bar` - yes.
    pub(crate) fn matches_arg(&self, arg: &Arg, adjacent: bool) -> bool {
        match arg {
            Arg::Short(s, is_adj, _) => self.short.contains(s) && (!adjacent || *is_adj),
            Arg::Long(l, is_adj, _) => self.long.contains(&l.as_str()) && (!adjacent || *is_adj),
            Arg::Word(_) | Arg::PosWord(_) | Arg::Ambiguity(..) => false,
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
/// When using combinatoring API you can specify the type with turbofish, for parsing types
/// that don't implement [`FromStr`] you can use consume a `String`/`OsString` first and parse
/// it by hands.
/// ```no_run
/// # use bpaf::*;
/// fn parse_pos() -> impl Parser<usize> {
///     positional::<usize>("POS")
/// }
/// ```
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
/// # let pos = ||positional::<String>("POS");
/// let valid = construct!(non_pos(), pos());
/// let invalid = construct!(pos(), non_pos());
/// ```
///
/// **`bpaf` panics during help generation unless if this restriction holds**
///
#[doc = include_str!("docs/positional.md")]
#[must_use]
pub fn positional<T>(metavar: &'static str) -> ParsePositional<T> {
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
#[doc = include_str!("docs/command.md")]
///
#[must_use]
pub fn command<T>(name: &'static str, subparser: OptionParser<T>) -> ParseCommand<T>
where
    T: 'static,
{
    ParseCommand {
        longs: vec![name],
        shorts: Vec::new(),
        help: subparser.short_descr().map(Into::into),
        subparser,
    }
}

/// Builder structure for the [`command`]
///
/// Created with [`command`], implements parser for the inner structure, gives access to [`help`](ParseCommand::help).
pub struct ParseCommand<T> {
    longs: Vec<&'static str>,
    shorts: Vec<char>,
    help: Option<String>,
    subparser: OptionParser<T>,
}

impl<P> ParseCommand<P> {
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
    /// Behavior is similar to [`short`](NamedArg::short), only first short name is visible.
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.shorts.push(short);
        self
    }

    /// Add a custom hidden long alias for a command
    ///
    /// Behavior is similar to [`long`](NamedArg::long), but since you had to specify the first long
    /// name when making the command - this one becomes a hidden alias.
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.longs.push(long);
        self
    }
}

impl<T> Parser<T> for ParseCommand<T> {
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
            let res = self
                .subparser
                .run_subparser(args)
                .map_err(Error::ParseFailure);
            res
        } else {
            #[cfg(feature = "autocomplete")]
            args.push_command(self.longs[0], self.shorts.first().copied(), &self.help);

            Err(Error::Missing(vec![self.item()]))
        }
    }

    fn meta(&self) -> Meta {
        Meta::from(self.item())
    }
}

impl<T> ParseCommand<T> {
    fn item(&self) -> Item {
        Item::Command {
            name: self.longs[0],
            short: self.shorts.first().copied(),
            help: self.help.clone(),
            meta: Box::new(self.subparser.inner.meta()),
            info: Box::new(self.subparser.info.clone()),
        }
    }
}

fn build_flag_parser<T>(present: T, absent: Option<T>, named: NamedArg) -> ParseFlag<T>
where
    T: Clone + 'static,
{
    ParseFlag {
        present,
        absent,
        named,
    }
}

#[derive(Clone)]
struct ParseFlag<T> {
    present: T,
    absent: Option<T>,
    named: NamedArg,
}

impl<T: Clone + 'static> Parser<T> for ParseFlag<T> {
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

fn build_argument<T>(named: NamedArg, metavar: &'static str) -> ParseArgument<T> {
    ParseArgument {
        named,
        metavar,
        ty: PhantomData,
        adjacent: false,
    }
}

/// Parser for a named argument, created with [`argument`](NamedArg::argument).
#[derive(Clone)]
pub struct ParseArgument<T> {
    ty: PhantomData<T>,
    named: NamedArg,
    metavar: &'static str,
    adjacent: bool,
}

impl<T> ParseArgument<T> {
    /// Restrict parsed arguments to have both flag and a value in the same word:
    ///
    /// In other words adjacent restricted `ParseArgument` would accept `--flag=value` or
    /// `-fbar` but not `--flag value`. Note, this is different from [`adjacent`](Parser::adjacent),
    /// just plays a similar role.
    ///
    /// Should allow to parse some of the more unusual things
    ///
    #[doc = include_str!("docs/pos_adjacent.md")]
    #[must_use]
    pub fn adjacent(mut self) -> Self {
        self.adjacent = true;
        self
    }

    fn item(&self) -> Item {
        Item::Argument {
            name: ShortLong::from(&self.named),
            metavar: Metavar(self.metavar),
            env: self.named.env.first().copied(),
            help: self.named.help.clone(),
            shorts: self.named.short.clone(),
        }
    }

    fn take_argument(&self, args: &mut Args) -> Result<OsString, Error> {
        if self.named.short.is_empty() && self.named.long.is_empty() {
            if let Some(name) = self.named.env.first() {
                let msg = format!("env variable {} is not set", name);
                return Err(Error::Message(msg, true));
            }
        }
        match args.take_arg(&self.named, self.adjacent) {
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
                    Ok(val)
                } else {
                    Err(Error::Missing(vec![self.item()]))
                }
            }
        }
    }
}

impl<T> Parser<T> for ParseArgument<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let os = self.take_argument(args)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(args.word_parse_error(&err)),
        }
    }

    fn meta(&self) -> Meta {
        if self.named.short.is_empty() && self.named.long.is_empty() {
            Meta::Skip
        } else {
            Meta::from(self.item())
        }
    }
}

fn build_positional<T>(metavar: &'static str) -> ParsePositional<T> {
    ParsePositional {
        metavar,
        help: None,
        result_type: PhantomData,
        strict: false,
    }
}

/// Parse a positional item, created with [`positional`]
///
/// You can add extra information to positional parsers with [`help`](Self::help)
/// and [`strict`](Self::strict) on this struct.
#[derive(Clone)]
pub struct ParsePositional<T> {
    metavar: &'static str,
    help: Option<String>,
    result_type: PhantomData<T>,
    strict: bool,
}

impl<T> ParsePositional<T> {
    /// Add a help message to a [`positional`] parser
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_name() -> impl Parser<String> {
    ///     positional::<String>("NAME")
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
    /// See also [`NamedArg::help`]
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
    /// $ cargo run --example basic -- --help
    /// ```
    /// here `cargo` takes a `--help` as a positional item and passes it to the example
    ///
    /// `bpaf` allows to require user to pass `--` for positional items with `strict` annotation.
    /// `bpaf` would display such positional elements differently in usage line as well. If your
    /// app requires several different strict positional elements - it's better to place
    /// this annotation only to the first one.
    ///
    /// # Example
    /// Usage line for a cargo-run like app that takes an app and possibly many strictly
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
    ///     positional::<std::ffi::OsString>("OPTS")
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
        Meta::from(Item::Positional {
            metavar: Metavar(self.metavar),
            help: self.help.clone(),
            strict: self.strict,
        })
    }
}

fn parse_word(
    args: &mut Args,
    strict: bool,
    metavar: &'static str,
    help: &Option<String>,
) -> Result<OsString, Error> {
    if let Some((is_strict, word)) = args.take_positional_word(Metavar(metavar))? {
        if strict && !is_strict {
            #[cfg(feature = "autocomplete")]
            args.push_value("--", &Some("-- Positional only items".to_owned()), false);

            return Err(Error::Message(
                format!("Expected <{}> to be on the right side of --", metavar),
                false,
            ));
        }
        #[cfg(feature = "autocomplete")]
        if args.touching_last_remove() && !args.no_pos_ahead {
            args.push_metadata(metavar, help, false);
            args.no_pos_ahead = true;
        }
        Ok(word)
    } else {
        #[cfg(feature = "autocomplete")]
        if !args.no_pos_ahead {
            args.push_metadata(metavar, help, false);
            args.no_pos_ahead = true;
        }

        let item = Item::Positional {
            metavar: Metavar(metavar),
            help: help.clone(),
            strict,
        };
        Err(Error::Missing(vec![item]))
    }
}

impl<T> Parser<T> for ParsePositional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let os = parse_word(args, self.strict, self.metavar, &self.help)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(args.word_parse_error(&err)),
        }
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}

/// Parse the next available item on a command line with no restrictions, created with [`any`].
pub struct ParseAny<T> {
    ty: PhantomData<T>,
    metavar: Metavar,
    strict: bool,
    help: Option<String>,
}

/// Take next unconsumed item on the command line as raw [`String`] or [`OsString`]
///
/// **`any` is designed to consume items that don't fit into usual `flag`/`switch`/`positional`
/// /`argument`/`command` classification**
///
/// `any` behaves similar to [`positional`] so you should be using it near the right most end of
/// the consumer struct. Note, consuming "anything" also consumes `--help` unless restricted
/// with `guard`. It's better stick to `positional` unless you are trying to consume raw options
/// to pass to some other process or do some special handling.
///
/// When using combinatoring API you can specify the type with turbofish, for parsing types
/// that don't implement [`FromStr`] you can use consume a `String`/`OsString` first and parse
/// it by hands. For `any` you would usually consume it either as a `String` or `OsString`.
/// ```no_run
/// # use bpaf::*;
/// # use std::ffi::OsString;
/// fn parse_any() -> impl Parser<OsString> {
///     any::<OsString>("ANYTHING")
/// }
/// ```
///
#[doc = include_str!("docs/any.md")]
///
/// See [`adjacent`](Parser::adjacent) for more examples
#[must_use]
pub fn any<T>(metavar: &'static str) -> ParseAny<T> {
    ParseAny {
        ty: PhantomData,
        metavar: Metavar(metavar),
        strict: false,
        help: None,
    }
}

impl<T> ParseAny<T> {
    /// Add a help message to [`any`] parser.
    #[doc = include_str!("docs/any.md")]
    #[must_use]
    pub fn help<M: Into<String>>(mut self, help: M) -> Self {
        self.help = Some(help.into());
        self
    }

    fn meta(&self) -> Meta {
        Meta::from(self.item())
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
        match item {
            Arg::Ambiguity(_, s) => {
                let os = s.clone();
                args.remove(ix);
                Ok(os)
            }
            Arg::Short(_, part, s) | Arg::Long(_, part, s) => {
                let os = s.clone();
                if *part {
                    args.remove(ix + 1);
                }
                args.remove(ix);

                Ok(os)
            }

            Arg::Word(w) | Arg::PosWord(w) => {
                let os = w.clone();
                args.remove(ix);
                Ok(os)
            }
        }
    }
}

impl<T> Parser<T> for ParseAny<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let os = self.next_os_string(args)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(args.word_parse_error(&err)), // Error::Stderr(err)),
        }
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}
