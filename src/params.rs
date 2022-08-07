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

/// A named thing used to create Flag, Switch or Argument.
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
/// let switch =
///     short('f')
///         .short('F')
///         .long("flag")
///         .help("a flag that does a thing")
///         .switch(); // impl Parser<bool>
/// # drop(switch);
/// ```
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
/// let switch =
///     short('f')
///         .long("flag")
///         .long("Flag")
///         .help("a flag that does a thing")
///         .switch(); // impl Parser<bool> =
/// # drop(switch);
/// ```
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
/// You must specify either short or long key if you start the chain from `env`.
///
/// ```rust
/// # use bpaf::*;
/// let key = short('k')
///            .long("key")
///            .env("API_KEY")
///            .help("Use this API key to access the API")
///            .argument("KEY"); // impl Parser<String>
/// # drop(key)
/// ```
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
    /// let switch =
    ///     short('f')
    ///         .short('F')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch(); // impl Parser<bool>
    /// # drop(switch);
    /// ```
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
    /// let switch =
    ///     short('f')
    ///         .long("flag")
    ///         .long("Flag")
    ///         .help("a flag that does a thing")
    ///         .switch(); // impl Parser<bool>
    /// # drop(switch);
    /// ```
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
    /// let key = short('k')
    ///            .long("key")
    ///            .env("API_KEY")
    ///            .help("Use this API key to access the API")
    ///            .argument("KEY"); // impl Parser<String>
    /// # drop(key)
    /// ```
    #[must_use]
    pub fn env(mut self, variable: &'static str) -> Self {
        self.env.push(variable);
        self
    }

    /// Add a help message to a flag/switch/argument
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let switch =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch(); // impl Parser<bool>
    /// # drop(switch);
    /// ```
    #[must_use]
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
    /// let switch =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch(); // impl Parser<bool>
    /// # drop(switch);
    /// ```
    #[must_use]
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
    /// let switch =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .flag(Flag::Present, Flag::Absent); // impl Parser<Flag>
    /// # drop(switch);
    /// ```
    #[must_use]
    pub fn flag<T>(self, present: T, absent: T) -> impl Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self)
    }

    /// Required flag with custom value
    ///
    /// Parser produces a value if present and fails otherwise.
    /// Designed to be used with combination of other parser(s).
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Clone)]
    /// enum Decision {
    ///     On,
    ///     Off,
    ///     Undecided
    /// }
    /// let on = long("on").req_flag(Decision::On);
    /// let off = long("off").req_flag(Decision::Off);
    /// // Requires user to specify either `--on` or `--off`
    /// let state = on.or_else(off).fallback(Decision::Undecided); // impl Parser<Decision>
    /// # drop(state);
    /// ```
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // counts how many times flag `-v` is given on a command line
    /// let verbosity = short('v').req_flag(()).many().map(|v| v.len()); // impl Parser<usize>
    /// # drop(verbosity);
    /// ```
    ///
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
    /// let arg = short('n').long("name").argument("NAME");
    /// # drop(arg)
    /// ```
    #[must_use]
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
    /// let arg = short('n').long("name").argument_os("NAME");
    /// # drop(arg)
    /// ```
    #[must_use]
    pub fn argument_os(self, metavar: &'static str) -> impl Parser<OsString> {
        build_argument(self, metavar).map(|x| x.os)
    }
}

/// Positional argument that can be encoded as String
///
/// ```rust
/// # use bpaf::*;
/// let arg = positional("INPUT"); // impl Parser<String>
/// # drop(arg)
/// ```
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
    let check = move |w: &Word| match &w.utf8 {
        Some(s) => check(s),
        None => false,
    };

    build_positional_if(metavar, check).parse(|x| match x {
        Some(Word { utf8: Some(w), .. }) => Ok(Some(w)),
        Some(_) => Err("not utf8"),
        None => Ok(None),
    })
}

/// Positional argument in OS specific encoding
///
/// ```rust
/// # use bpaf::*;
/// # use std::ffi::OsString;
/// let arg = positional_os("INPUT"); // impl Parser<OsString>
/// # drop(arg)
/// ```
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
pub fn command<P, T, M>(name: &'static str, help: Option<M>, subparser: P) -> impl Parser<T>
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
struct Command<P> {
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

#[derive(Clone)]
struct BuildPositionalIf<F> {
    meta: Meta,
    check: F,
}

impl<F> Parser<Option<Word>> for BuildPositionalIf<F>
where
    F: Fn(&Word) -> bool + 'static,
{
    fn run(&self, args: &mut Args) -> Result<Option<Word>, Error> {
        match args.peek() {
            Some(Arg::Word(w_ref)) => {
                if (self.check)(w_ref) {
                    let w_owned = args
                        .take_positional_word()?
                        .expect("We just confirmed it's there");
                    Ok(Some(w_owned))
                } else {
                    Ok(None)
                }
            }

            //            Some(_) => Err(Error::Missing(vec![self.meta.clone()])),
            _ => Ok(None),
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}

fn build_positional_if<F>(metavar: &'static str, check: F) -> impl Parser<Option<Word>>
where
    F: Fn(&Word) -> bool + 'static,
{
    let item = Item::Positional { metavar };
    let meta = item.required(false);
    BuildPositionalIf { meta, check }
}
