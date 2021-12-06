//! Primitives to define parsers
//!
//! # Terminology
//!
//! ## Flag
//!
//! A simple no-argument command line option that takes no extra parameters, when decoded produces
//! a fixed value. Can have a short (`-f`) or a long (`--flag`) name, see [`Named::flag`] and
//! [`Named::req_flag`].
//!
//! ## Switch
//!
//! A special case of a flag that gets decoded into a `bool`, see [`Named::switch`].
//!
//! ## Argument
//!
//! A command line option with a name that also takes a value. Can have a short (`-f value`) or a
//! long (`--flag value`) name, see [`Named::argument`].
//!
//! ## Positional
//!
//! A positional command with no additonal name, for example in `vim main.rs` `main.rs`
//! is a positional argument. See [`positional`].
//!
//! ## Command
//!
//! A command is used to define a starting point for an independent subparser, for example in
//! `cargo check --workspace` `check` defines a subparser that acceprts `--workspace` switch. See
//! [`command`]
//!
use std::ffi::OsString;

use super::*;
use crate::{
    args::Word,
    info::{ItemKind, Meta},
};

/// A named thing used to create Flag, Switch or Argument.
#[derive(Clone, Debug)]
pub struct Named {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
}

/// A flag/switch/argument that has a short name
///
/// You can specify it multiple times, items past the first one will become
/// a hidden aliases.
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
pub fn short(short: char) -> Named {
    Named {
        short: vec![short],
        long: Vec::new(),
        help: None,
    }
}

/// A flag/switch/argument that has a long name
///
/// You can specify it multiple times, items past the first one will become
/// a hidden aliases.
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
pub fn long(long: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: vec![long],
        help: None,
    }
}

impl Named {
    /// Add a short name to a flag/switch/argument
    ///
    /// You can specify it multiple times, items past the first one will become
    /// a hidden aliases.
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
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }

    /// Add a long name to a flag/switch/argument
    ///
    /// You can specify it multiple times, items past the first one will become
    /// a hidden aliases.
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
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
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
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

impl Named {
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
    pub fn switch(self) -> Parser<bool> {
        build_flag_parser(true, Some(false), self.short, self.long, self.help)
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
    pub fn flag<T>(self, present: T, absent: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self.short, self.long, self.help)
    }

    /// Required flag with custom value
    ///
    /// Parser produces a value if present and fails otherwise.
    /// Designed to be used with combination of other parser.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let on = long("on").req_flag(true);
    /// let off = long("off").req_flag(false);
    /// // Requires user to specify either `--on` or `--off`
    /// let state: Parser<bool> = on.or_else(off);
    /// # drop(state);
    /// ```
    ///
    /// ```rust
    /// use bpaf::*;
    /// // counts how many times flag `-v` is given on a command line
    /// let verbosity: Parser<usize> = short('v').req_flag(()).many().map(|v| v.len());
    /// # drop(verbosity);
    /// ```
    ///
    pub fn req_flag<T>(self, present: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, None, self.short, self.long, self.help)
    }

    /// utf encoded named argument
    pub fn argument(self, metavar: &'static str) -> Parser<String> {
        build_argument(self.short, self.long, self.help, metavar)
            .parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    /// os encoded named argument
    pub fn argument_os(self, metavar: &'static str) -> Parser<OsString> {
        build_argument(self.short, self.long, self.help, metavar).map(|x| x.os)
    }
}

/// utf encoded positional argument
pub fn positional(metavar: &'static str) -> Parser<String> {
    build_positional(metavar).parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
}

/// os encoded positional argument
pub fn positional_os(metavar: &'static str) -> Parser<OsString> {
    build_positional(metavar).map(|x| x.os)
}

/// Command
pub fn command<T, M>(name: &'static str, help: M, p: ParserInfo<T>) -> Parser<T>
where
    T: 'static,
    M: Into<String>,
{
    let parse = move |mut i: Args| match i.take_word(name) {
        Some(i) => (p.parse)(i),
        None => Err(Error::Stderr(format!("expected {}", name))),
    };
    let meta = Meta::from(Item {
        short: None,
        long: Some(name),
        metavar: None,
        help: Some(help.into()),
        kind: ItemKind::Command,
    });
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_flag_parser<T>(
    present: T,
    absent: Option<T>,
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
) -> Parser<T>
where
    T: Clone + 'static,
{
    let item = Item {
        short: short.first().copied(),
        long: long.first().copied(),
        metavar: None,
        help,
        kind: ItemKind::Flag,
    };
    let required = absent.is_none();
    let meta = item.required(required);

    let missing = if required {
        Error::Missing(vec![meta.clone()])
    } else {
        Error::Stdout(String::new())
    };

    let parse = move |mut i: Args| {
        for &short in short.iter() {
            if let Some(i) = i.take_short_flag(short) {
                return Ok((present.clone(), i));
            }
        }
        for long in long.iter() {
            if let Some(i) = i.take_long_flag(long) {
                return Ok((present.clone(), i));
            }
        }
        Ok((absent.as_ref().ok_or_else(|| missing.clone())?.clone(), i))
    };
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_argument(
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
    metavar: &'static str,
) -> Parser<Word> {
    let item = Item {
        kind: ItemKind::Flag,
        short: short.first().copied(),
        long: long.first().copied(),
        metavar: Some(metavar),
        help,
    };
    let meta = item.required(true);
    let meta2 = meta.clone();
    let parse = move |mut i: Args| {
        for &short in short.iter() {
            if let Some((w, c)) = i.take_short_arg(short)? {
                return Ok((w, c));
            }
        }
        for long in long.iter() {
            if let Some((w, c)) = i.take_long_arg(long)? {
                return Ok((w, c));
            }
        }
        Err(Error::Missing(vec![meta2.clone()]))
    };

    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

fn build_positional(metavar: &'static str) -> Parser<Word> {
    let item = Item {
        short: None,
        long: None,
        metavar: Some(metavar),
        help: None,
        kind: ItemKind::Positional,
    };
    let meta = item.required(true);
    let meta2 = meta.clone();

    let parse = move |mut args: Args| match args.take_positional() {
        Some((word, args)) => return Ok((word, args)),
        None => Err(Error::Missing(vec![meta2.clone()])),
    };
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}
