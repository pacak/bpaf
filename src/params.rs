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
use std::ffi::OsString;

use super::{Args, Error, OptionParser, Parser};
use crate::{
    args::{Arg, Word},
    item::ShortLong,
    Item, Meta,
};

/// A named thing used to create flag, switch or argument.
///
/// # Ways to consume data
/// bpaf supports several different ways user can specify values on a command line:
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
/// - [`switch`](Named::switch) - similar to `flag`, but instead of custom values `bool` is used,
/// mostly serves as a convenient alias to `.flag(true, false)`
///
/// ```console
/// $ app -a -bc
/// ```
///
/// - [`argument`](Named::argument) - a short or long `flag` followed by a string literal. String
/// can be separated from the flag by a space or by `=` sign: `-f foo`, `--flag bar` or `-o=-` are
/// all valid flags. Note, string literal must not start with `-` unless separated from the flag
/// with `=` and should be valid utf8 only. To consume [`OsString`](std::ffi::OsString) encoded
/// values you can use [`argument_os`](Named::argument_os).
///
/// ```console
/// $ app -o file.txt
/// ```
///
/// - [`positional`] - an arbitrary utf8 string literal passed on a command line, there's also
/// [`positional_os`] variant that deals with `OsString` named. Usually represents input
/// files such as `cat file.txt`, but can serve other purposes.
///
/// ```console
/// $ cat file.txt
/// ```
///
/// - [`command`] - a fixed utf8 string literal that starts a separate subparser that only
/// gets executed when command name is present. For example `cargo build` invokes
/// command `"build" and after build `cargo` starts accepting values it won't accept otherwise
///
/// ```console
/// $ cargo build --out-dir my_target
/// // works since command "build" supports --out-dir argument
/// $ cargo check --out-dir my_target
/// // fails since --out-dir is not a valid argument for command "check"
/// ```
///
/// As most of the other parsers bpaf treats anything to the right of `--` symbol as positional
/// arguments regardless their names:
///
/// ```console
/// $ app -o file.txt -- --those --are --positional --items
/// ```
///
/// # Combinatoric usage
///
/// Named items (`argument`, `flag` and `switch`) can have up to 2 visible names (short and long)
/// and multiple hidden short and long aliases if needed. It is also possible to consume items from
/// environment variables using [`env`](Named::env). You usually start with [`short`] or [`long`]
/// function, then apply [`short`](Named::short) / [`long`](Named::long) / [`env`](Named::env) /
/// [`help`](Named::help) repeatedly until desired set of names is achieved then transform it into
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
///     ...
///     -i --item <ITEM>  [env:ITEM: N/A]
///                       A string used by this example
///     ...
/// ```
///
/// # Derive usage
///
/// Unlike combinatoric API where you forced to specify names for your subparsers derive API allows
/// to omit some or all the details:
/// 1. If no naming information is present at all - bpaf would use field name as a long name (or a
///    short name if field name consists of a single character)
/// 2. If `short` or `long` annotation is present without an argument - bpaf would use first character
///    or a full name as long and short name respectively. It will not try to add implicit long or
///    short name from the previous item.
/// 3. If `short` or `long` annotation is present with an argument - those are values bpaf would
///    use instead of the original field name
/// 4. If `env` annotation is present - it would be used to generate `.env(...)` method:
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
///    ...
///         --flag-1          flag with no annotation
///    -f                    explicit short suppresses long
///    -z                    explicit short with custom letter
///    -d, --deposit         explicit short and long
///        --database <ARG>  [env:top_secret_database: N/A]
///                          implicit long + env variable from DB constant
///        --user <ARG>      [env:USER = "pacak"]
///                              implicit long + env variable "USER"
///    ...
/// ```
#[derive(Clone, Debug)]
pub struct Named {
    pub(crate) short: Vec<char>,
    pub(crate) long: Vec<&'static str>,
    env: Vec<&'static str>,
    help: Option<String>,
}

/// A flag/switch/argument that has a short name
///
/// You can specify it multiple times, items past the first one represent
/// hidden aliases.
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
/// You can specify it multiple times, items past the first represent
/// hidden aliases.
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
/// If named value is not present - try to fallback to this environment variable.
/// You can specify it multiple times, items past the first one will become hidden aliases.
///
/// # Combinatoric usage
/// You must specify either short or long key if you start the chain from `env`.
///
/// ```rust
/// # use bpaf::*;
/// fn parse_string() -> impl Parser<String> {
///     short('k')
///            .long("key")
///            .env("API_KEY")
///            .help("Use this API key to access the API")
///            .argument("KEY")
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
    /// You can specify it multiple times, items past the first one represent
    /// hidden aliases.
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
    /// You can specify it multiple times, items past the first one will become
    /// hidden aliases.
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
    /// If named value is not present - try to fallback to this environment variable.
    /// You can specify it multiple times, items past the first one will become hidden aliases.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_string() -> impl Parser<String> {
    ///     short('k')
    ///            .long("key")
    ///            .env("API_KEY")
    ///            .help("Use this API key to access the API")
    ///            .argument("KEY")
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
    /// Doc comments are converted into help messages according to following rules:
    /// 1. Blank lines are dropped
    /// 2. Parsing stops at a double blank line
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
    ///     /// But this one is not
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
    /// Parser produces a value if present and fails otherwise.
    /// Designed to be used with combination of other parsers.
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Clone)]
    /// enum Decision {
    ///     On,
    ///     Off,
    ///     Undecided
    /// }
    ///
    /// // user can specify either --on or --off, parser would fallback to `Undecided`
    /// fn parse_decision() -> impl Parser<Decision> {
    ///     let on = long("on").req_flag(Decision::On);
    ///     let off = long("off").req_flag(Decision::Off);
    ///     on.or_else(off).fallback(Decision::Undecided)
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```console
    /// $ app --on
    /// // Decision::On
    /// $ app
    /// // Decision::Undecided
    /// ```
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // counts how many times flag `-v` is given on a command line
    /// fn verbosity() -> impl Parser<usize> {
    ///     short('v').req_flag(()).many().map(|v| v.len())
    /// }
    /// ```
    /// # Example
    /// ```console
    /// $ app
    /// // 0
    /// $ app -vvv
    /// // 3
    /// ```
    ///
    /// # Derive usage
    /// bpaf would transform field-less enum variants into values combined by `req_flag`.
    /// In addition to naming annotations (`short`, `long` and `env`) such variants also accept
    /// `hide` and `default` annotations. Former hides it from `--help` (see
    /// [`hide`](Parser::hide), later makes it a default choice if preceeding variants fail to
    /// parse. You shoud only use `default` annotation on the last variant of enum.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// enum Decision {
    ///     On,
    ///     Off,
    ///     #[bpaf(long, hide, default)]
    ///     Undecided,
    /// }
    /// ```
    ///
    /// See [`Named`] for more details
    #[must_use]
    pub fn req_flag<T>(self, present: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, None, self)
    }

    /// Named argument that can be encoded as String
    ///
    /// Argument must be present (but can be made into [`Option`] using
    /// [`optional`][Parser::optional]) and it must contain only valid unicode characters.
    /// For OS specific encoding see [`argument_os`][Named::argument_os].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_string() -> impl Parser<String> {
    ///     short('n')
    ///         .long("name")
    ///         .argument("NAME")
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn argument(self, metavar: &'static str) -> impl Parser<String> {
        build_argument(self, metavar).parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    /// Named argument in OS specific encoding
    ///
    /// Argument must be present but can be made into [`Option`] using
    /// [`optional`][Parser::optional]. If you prefer to panic on non utf8 encoding see
    /// [`argument`][Named::argument].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn parse_osstring() -> impl Parser<std::ffi::OsString> {
    ///     short('n')
    ///         .long("name")
    ///         .argument_os("NAME")
    /// }
    /// ```
    #[must_use]
    /// See [`Named`] for more details
    pub fn argument_os(self, metavar: &'static str) -> impl Parser<OsString> {
        build_argument(self, metavar).map(|x| x.os)
    }
}

/// Positional argument that can be encoded as String
///
/// For named flags and arguments ordering generally does not matter: most programs would
/// understand `-O2 -v` the same way as `-v -O2`, but for positional items order matters: in unix
/// `cat hello world` and `cat world hello` would display contents of the same two files but in
/// different order.
///
/// # Important restriction
/// When parsing positional arguments from command lines you should have parsers for all your
/// named values and command before parsers for positional items. In derive API fields parsed as
/// positional should be at the end of your `struct`/`enum`. If positional field is nested inside
/// some other field - they should go at the end as well. Failing to do can result in behavior
/// confusing for end user.
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
/// Fields in tuple-like structures are parsed as positional items
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options(String);
/// ```
/// Additionally annotations `positional` and `positional_os` can be used with optional metavar
/// name
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
pub fn positional(metavar: &'static str) -> impl Parser<String> {
    build_positional(metavar).parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
}

/// Positional argument that can be encoded as String and will be taken only if check passes
///
/// ```rust
/// # use bpaf::*;
/// let is_short = |s: &str| s.len() < 10;
/// // skip this positional argument unless it's less than 10 bytes long
/// let arg  = positional_if("INPUT", is_short); // impl Parser<Option<String>>
/// # drop(arg)
/// ```
pub fn positional_if<F>(metavar: &'static str, check: F) -> impl Parser<Option<String>>
where
    F: Fn(&str) -> bool + 'static,
{
    positional(metavar).guard(move |s| check(s), "").optional()
}

/// Positional argument in OS specific encoding
///
/// For named flags and arguments ordering generally does not matter: most programs would
/// understand `-O2 -v` the same way as `-v -O2`, but for positional items order matters: in unix
/// `cat hello world` and `cat world hello` would display contents of the same two files but in
/// different order.
///
/// # Important restriction
/// When parsing positional arguments from command lines you should have parsers for all your
/// named values and command before parsers for positional items. In derive API fields parsed as
/// positional should be at the end of your `struct`/`enum`. If positional field is nested inside
/// some other field - they should go at the end as well. Failing to do can result in behavior
/// confusing for end user.
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
/// Fields in tuple-like structures are parsed as positional items, bpaf would automatically
/// substitute `positional_os` annotation for `OsString` and `PathBuf`.
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options(std::ffi::OsString);
/// ```
/// Additionally annotations `positional` and `positional_os` can be used with optional metavar
/// name
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
pub fn positional_os(metavar: &'static str) -> impl Parser<OsString> {
    build_positional(metavar).map(|x| x.os)
}

/// Subcommand parser
///
/// ```rust
/// # use bpaf::*;
/// // Define a parser to use in a subcommand in a usual way.
/// // This parser accepts a single --workspace switch
/// let ws = long("workspace").help("Check all packages in the workspace").switch();
/// let decorated = Info::default()
///     .descr("Check a package for errors")
///     .for_parser(ws); // impl OptionParser<bool>
///
/// // Convert subparser into a parser.
/// // Note description "Check a package for errors" is specified twice:
/// // - Parser uses version from `descr` when user calls `% prog check --help`,
/// // - Parser uses version from `command` user calls `% prog --help` along
/// //   with descriptions for other commands if present.
/// let check = command("check", Some("Check a local package for errors"), decorated); // impl Parser<bool>
///
/// // when ther's several commands it can be a good idea to wrap each into a enum either before
/// // or after converting it into subparser:
/// #[derive(Clone, Debug)]
/// enum Command {
///     Check(bool)
/// }
/// let check = check.map(Command::Check); // impl Parser<Command>
///
/// // at this point command line accepts following commands:
/// // `% prog --help`            - display a global help and exit
/// // `% prog check --help`      - display help specific to check subcommand and exit
/// // `% prog check`             - produce `Command::Check(false)`
/// // `% prog check --workspace` - produce `Command::Check(true)`
/// let opt = Info::default().for_parser(check);
/// # drop(opt)
/// ```
#[must_use]
pub fn command<P, T, M>(name: &'static str, help: Option<M>, subparser: P) -> Command<P>
where
    P: OptionParser<T>,
    T: 'static,
    M: Into<String>,
{
    let meta = Meta::Item(Item::Command {
        name,
        help: help.map(Into::into),
    });
    Command {
        name,
        meta,
        subparser,
    }
}

#[derive(Clone)]
pub struct Command<P> {
    name: &'static str,
    meta: Meta,
    subparser: P,
}

impl<P, T> Parser<T> for Command<P>
where
    P: OptionParser<T>,
{
    fn run(&self, args: &mut Args) -> Result<T, Error> {
        if args.take_cmd(self.name) {
            self.subparser.run_subparser(args)
        } else {
            Err(Error::Missing(vec![self.meta.clone()]))
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}

fn short_or_long_flag(arg: &Arg, shorts: &[char], longs: &[&str]) -> bool {
    shorts.iter().any(|&c| arg.is_short(c)) || longs.iter().any(|s| arg.is_long(s))
}

fn build_flag_parser<T>(present: T, absent: Option<T>, named: Named) -> impl Parser<T>
where
    T: Clone + 'static,
{
    let item = Item::Flag {
        name: ShortLong::from(&named),
        help: named.help.clone(),
    };

    let meta = item.required(absent.is_none());

    BuildFlagParser {
        present,
        absent,
        shorts: named.short,
        longs: named.long,
        envs: named.env,
        meta,
    }
}

#[derive(Clone)]
struct BuildFlagParser<T> {
    present: T,
    absent: Option<T>,
    shorts: Vec<char>,
    longs: Vec<&'static str>,
    envs: Vec<&'static str>,
    meta: Meta,
}

impl<T: Clone + 'static> Parser<T> for BuildFlagParser<T> {
    fn run(&self, args: &mut Args) -> Result<T, Error> {
        if args.take_flag(|arg| short_or_long_flag(arg, &self.shorts, &self.longs))
            || self.envs.iter().find_map(std::env::var_os).is_some()
        {
            Ok(self.present.clone())
        } else {
            match &self.absent {
                Some(ok) => Ok(ok.clone()),
                None => Err(Error::Missing(vec![self.meta.clone()])),
            }
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
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
    let item = Item::Argument {
        name: ShortLong::from(&named),
        metavar,
        env: named.env.first().copied(),
        help: named.help,
    };
    BuildArgument {
        shorts: named.short,
        longs: named.long,
        envs: named.env,
        meta: item.required(true),
    }
}

#[derive(Clone)]
struct BuildArgument {
    shorts: Vec<char>,
    longs: Vec<&'static str>,
    envs: Vec<&'static str>,
    meta: Meta,
}

impl Parser<Word> for BuildArgument {
    fn run(&self, args: &mut Args) -> Result<Word, Error> {
        if let Some(w) = args.take_arg(|arg| short_or_long_flag(arg, &self.shorts, &self.longs))? {
            Ok(w)
        } else if let Some(val) = self.envs.iter().find_map(std::env::var_os) {
            Ok(Word::from(val))
        } else {
            Err(Error::Missing(vec![self.meta.clone()]))
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}

fn build_positional(metavar: &'static str) -> impl Parser<Word> {
    let item = Item::Positional { metavar };
    let meta = item.required(true);
    BuildPositional { meta }
}

#[derive(Clone)]
struct BuildPositional {
    meta: Meta,
}

impl Parser<Word> for BuildPositional {
    fn run(&self, args: &mut Args) -> Result<Word, Error> {
        match args.take_positional_word()? {
            Some(word) => Ok(word),
            None => Err(Error::Missing(vec![self.meta.clone()])),
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}
