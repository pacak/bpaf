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
//! A special case of a flag that gets decoded into a `bool`, see [`Named::switch`] and
//! [`Named::req_switch`]
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

/// Create a flag/switch/argument with a short name
///
/// ```rust
/// # use bpaf::*;
/// let switch = short('f').long("flag").help("a flag").switch();
/// # drop(switch);
/// ```
pub fn short(short: char) -> Named {
    Named {
        short: vec![short],
        long: Vec::new(),
        help: None,
    }
}

/// Create a flag/switch/argument with a long name
pub fn long(long: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: vec![long],
        help: None,
    }
}

impl Named {
    /// Add a short name
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }

    /// Add a long name
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
        self
    }

    /// Add a help message
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

impl Named {
    /// A simple boolean flag
    pub fn switch(self) -> Parser<bool> {
        build_flag_parser(true, Some(false), self.short, self.long, self.help)
    }

    /// A required flag
    pub fn req_switch(self) -> Parser<bool> {
        build_flag_parser(true, None, self.short, self.long, self.help)
    }

    /// present/absent value flag
    pub fn flag<T>(self, present: T, absent: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self.short, self.long, self.help)
    }

    /// required flag
    pub fn req_flag<T>(self, present: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, None, self.short, self.long, self.help)
    }

    /// Positional argument
    pub fn argument(self, metavar: &'static str) -> Argument {
        Argument {
            short: self.short,
            long: self.long,
            help: self.help,
            metavar,
        }
    }
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

/// Named argument that also takes a value
pub struct Argument {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
    metavar: &'static str,
}

impl Argument {
    fn build_both(self) -> Parser<Word> {
        let item = Item {
            kind: ItemKind::Flag,
            short: self.short.first().copied(),
            long: self.long.first().copied(),
            metavar: Some(self.metavar),
            help: self.help,
        };
        let meta = item.required(true);
        let meta2 = meta.clone();
        let parse = move |mut i: Args| {
            for &short in self.short.iter() {
                if let Some((w, c)) = i.take_short_arg(short)? {
                    return Ok((w, c));
                }
            }
            for long in self.long.iter() {
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

    /// Convert parameter into a parser that produces a [`String`]
    pub fn build(self) -> Parser<String> {
        self.build_both().parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    /// Convert parameter into a parser that produces an [`OsString`]
    pub fn build_os(self) -> Parser<OsString> {
        self.build_both().map(|x| x.os)
    }
}

/// Unnamed positional argument
pub struct Positional {
    help: Option<String>,
    metavar: &'static str,
}

/// Unnamed positional argument
pub fn positional(metavar: &'static str) -> Positional {
    Positional {
        metavar,
        help: None,
    }
}

impl Positional {
    fn build_both(self) -> Parser<Word> {
        let item = Item {
            short: None,
            long: None,
            metavar: Some(self.metavar),
            help: self.help,
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

    /// Convert parameter into a parser that produces a [`String`]
    pub fn build(self) -> Parser<String> {
        self.build_both().parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    /// Convert parameter into a parser that produces a [`OsString`]
    pub fn build_os(self) -> Parser<OsString> {
        self.build_both().map(|x| x.os)
    }

    /// Named argument that also takes a value
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}
