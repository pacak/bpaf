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

use super::{Args, Error, OptionParser, Parser, Rc};
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
/// let switch: Parser<bool> =
///     short('f')
///         .short('F')
///         .long("flag")
///         .help("a flag that does a thing")
///         .switch();
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
/// let switch: Parser<bool> =
///     short('f')
///         .long("flag")
///         .long("Flag")
///         .help("a flag that does a thing")
///         .switch();
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
///            .argument("KEY");
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
    /// let switch: Parser<bool> =
    ///     short('f')
    ///         .short('F')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch();
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
    /// let switch: Parser<bool> =
    ///     short('f')
    ///         .long("flag")
    ///         .long("Flag")
    ///         .help("a flag that does a thing")
    ///         .switch();
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
    ///            .argument("KEY");
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
    /// let switch: Parser<bool> =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch();
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
    /// let switch: Parser<bool> =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .switch();
    /// # drop(switch);
    /// ```
    #[must_use]
    pub fn switch(self) -> Parser<bool> {
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
    /// let switch: Parser<Flag> =
    ///     short('f')
    ///         .long("flag")
    ///         .help("a flag that does a thing")
    ///         .flag(Flag::Present, Flag::Absent);
    /// # drop(switch);
    /// ```
    #[must_use]
    pub fn flag<T>(self, present: T, absent: T) -> Parser<T>
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
    /// let state: Parser<Decision> = on.or_else(off).fallback(Decision::Undecided);
    /// # drop(state);
    /// ```
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // counts how many times flag `-v` is given on a command line
    /// let verbosity: Parser<usize> = short('v').req_flag(()).many().map(|v| v.len());
    /// # drop(verbosity);
    /// ```
    ///
    #[must_use]
    pub fn req_flag<T>(self, present: T) -> Parser<T>
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
    pub fn argument(self, metavar: &'static str) -> Parser<String> {
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
    pub fn argument_os(self, metavar: &'static str) -> Parser<OsString> {
        build_argument(self, metavar).map(|x| x.os)
    }
}

/// Positional argument that can be encoded as String
///
/// ```rust
/// # use bpaf::*;
/// let arg: Parser<String> = positional("INPUT");
/// # drop(arg)
/// ```
#[must_use]
pub fn positional(metavar: &'static str) -> Parser<String> {
    build_positional(metavar).parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
}

/// Positional argument that can be encoded as String and will be taken only if check passes
///
/// ```rust
/// # use bpaf::*;
/// let is_short = |s: &str| s.len() < 10;
/// // skip this positional argument unless it's less than 10 bytes long
/// let arg: Parser<Option<String>> = positional_if("INPUT", is_short);
/// # drop(arg)
/// ```
pub fn positional_if<F>(metavar: &'static str, check: F) -> Parser<Option<String>>
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
/// let arg: Parser<OsString> = positional_os("INPUT");
/// # drop(arg)
/// ```
#[must_use]
pub fn positional_os(metavar: &'static str) -> Parser<OsString> {
    build_positional(metavar).map(|x| x.os)
}

/// Subcommand parser
///
/// ```rust
/// # use bpaf::*;
/// // Define a parser to use in a subcommand in a usual way.
/// // This parser accepts a single --workspace switch
/// let ws = long("workspace").help("Check all packages in the workspace").switch();
/// let decorated: OptionParser<bool> = Info::default()
///     .descr("Check a package for errors")
///     .for_parser(ws);
///
/// // Convert subparser into a parser.
/// // Note description "Check a package for errors" is specified twice:
/// // - Parser uses version from `descr` when user calls `% prog check --help`,
/// // - Parser uses version from `command` user calls `% prog --help` along
/// //   with descriptions for other commands if present.
/// let check: Parser<bool> = command("check", Some("Check a local package for errors"), decorated);
///
/// // when ther's several commands it can be a good idea to wrap each into a enum either before
/// // or after converting it into subparser:
/// #[derive(Clone, Debug)]
/// enum Command {
///     Check(bool)
/// }
/// let check: Parser<Command> = check.map(Command::Check);
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
pub fn command<T, M>(name: &'static str, help: Option<M>, subparser: OptionParser<T>) -> Parser<T>
where
    T: 'static,
    M: Into<String>,
{
    let item = Item::Command {
        name,
        help: help.map(Into::into),
    };

    let meta = Meta::Item(item);
    let meta2 = meta.clone();
    let parse = move |mut args: Args| {
        if args.take_cmd(name) {
            (subparser.parse)(args)
        } else {
            Err(Error::Missing(vec![meta2.clone()]))
        }
    };

    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn short_or_long_flag(arg: &Arg, shorts: &[char], longs: &[&str]) -> bool {
    shorts.iter().any(|&c| arg.is_short(c)) || longs.iter().any(|s| arg.is_long(s))
}

fn build_flag_parser<T>(present: T, absent: Option<T>, named: Named) -> Parser<T>
where
    T: Clone + 'static,
{
    if !named.env.is_empty() {
        // mostly cosmetic reasons
        assert!(
            !(named.short.is_empty() && named.long.is_empty()),
            "env fallback can only be used if name is present"
        );
    }

    let item = Item::Flag {
        name: ShortLong::from(&named),
        help: named.help.clone(),
    };
    let meta = item.required(absent.is_none());

    let missing = if absent.is_none() {
        Error::Missing(vec![meta.clone()])
    } else {
        Error::Stdout(String::new())
    };

    let parse = move |mut args: Args| {
        if args.take_flag(|arg| short_or_long_flag(arg, &named.short, &named.long))
            || named.env.iter().find_map(std::env::var_os).is_some()
        {
            Ok((present.clone(), args))
        } else {
            Ok((
                absent.as_ref().ok_or_else(|| missing.clone())?.clone(),
                args,
            ))
        }
    };
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_argument(named: Named, metavar: &'static str) -> Parser<Word> {
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
        help: named.help,
        env: named.env.first().copied(),
    };
    let meta = item.required(true);
    let meta2 = meta.clone();
    let parse = move |mut args: Args| {
        #[allow(clippy::option_if_let_else)]
        if let Some(w) = args.take_arg(|arg| short_or_long_flag(arg, &named.short, &named.long))? {
            Ok((w, args))
        } else if let Some(val) = named.env.iter().find_map(std::env::var_os) {
            Ok((Word::from(val), args))
        } else {
            Err(Error::Missing(vec![meta2.clone()]))
        }
    };

    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_positional(metavar: &'static str) -> Parser<Word> {
    let item = Item::Positional { metavar };
    let meta = item.required(true);

    let meta2 = meta.clone();

    let parse = move |mut args: Args| match args.take_positional_word()? {
        Some(word) => Ok((word, args)),
        None => Err(Error::Missing(vec![meta2.clone()])),
    };
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_positional_if<F>(metavar: &'static str, check: F) -> Parser<Option<Word>>
where
    F: Fn(&Word) -> bool + 'static,
{
    let item = Item::Positional { metavar };
    let meta = item.required(false);
    let meta2 = meta.clone();
    let parse = move |mut args: Args| match args.peek() {
        Some(Arg::Word(w_ref)) => {
            if check(w_ref) {
                let w_owned = args
                    .take_positional_word()?
                    .expect("We just confirmed it's there");
                Ok((Some(w_owned), args))
            } else {
                Ok((None, args))
            }
        }

        Some(_) => Err(Error::Missing(vec![meta2.clone()])),
        None => Ok((None, args)),
    };
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}
