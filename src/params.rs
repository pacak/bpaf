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
#![cfg_attr(not(doctest), doc = include_str!("docs2/flag.md"))]
//!
//! ## Required flag
//!
//! Similar to `flag`, but instead of falling back to the second value required flag parser would
//! fail. Mostly useful in combination with other parsers, created with [`NamedArg::req_flag`].
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/req_flag.md"))]
//!
//! ## Switch
//!
//! A special case of a flag that gets decoded into a `bool`, mostly serves as a convenient
//! shortcut to `.flag(true, false)`. Created with [`NamedArg::switch`].
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/switch.md"))]
//!
//! ## Argument
//!
//! A short or long `flag` followed by either a space or `=` and
//! then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all valid argument examples. Note, string
//! literal can't start with `-` unless separated from the flag with `=`. For short flags value
//! can follow immediately: `-fbar`.
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/argument.md"))]
//!
//! ## Positional
//!
//! A positional argument with no additonal name, for example in `vim main.rs` `main.rs`
//! is a positional argument. Can't start with `-`, created with [`positional`].
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))]
//!
//! ## Any
//!
//! Also a positional argument with no additional name, but unlike [`positional`] itself, [`any`]
//! isn't restricted to positional looking structure and would consume any items as they appear on
//! a command line. Can be useful to collect anything unused to pass to other applications.
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/any_simple.md"))]
#![cfg_attr(not(doctest), doc = include_str!("docs2/any_literal.md"))]
//!
//! ## Command
//!
//! A command defines a starting point for an independent subparser. Name must be a valid utf8
//! string. For example `cargo build` invokes command `"build"` and after `"build"` `cargo`
//! starts accepting values it won't accept otherwise
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/command.md"))]
//!
use std::{ffi::OsString, marker::PhantomData, str::FromStr};

use crate::{
    args::{Arg, State},
    error::{Message, MissingItem},
    from_os_str::parse_os_str,
    item::ShortLong,
    meta_help::Metavar,
    Doc, Error, Item, Meta, OptionParser, Parser,
};

#[cfg(doc)]
use crate::{any, command, env, long, positional, short};

/// A named thing used to create [`flag`](NamedArg::flag), [`switch`](NamedArg::switch) or
/// [`argument`](NamedArg::argument)
///
/// # Combinatoric usage
///
/// Named items (`argument`, `flag` and `switch`) can have up to 2 visible names (one short and one long)
/// and multiple hidden short and long aliases if needed. It's also possible to consume items from
/// environment variables using [`env`](NamedArg::env()). You usually start with [`short`] or [`long`]
/// function, then apply [`short`](NamedArg::short) / [`long`](NamedArg::long) / [`env`](NamedArg::env()) /
/// [`help`](NamedArg::help) repeatedly to build a desired set of names then transform it into
/// a parser using `flag`, `switch` or `positional`.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/named_arg_combine.md"))]
///
/// # Derive usage
///
/// When using derive API it is possible to omit some or all the details:
/// 1. If no naming information is present at all - `bpaf` would use field name as a long name
///    (or a short name if field name consists of a single character)
/// 2. If `short` or `long` annotation is present without an argument - `bpaf` would use first character
///    or a full name as long and short name respectively. It won't try to add implicit long or
///    short name from the previous item.
/// 3. If `short` or `long` annotation is present with an argument - those are values `bpaf` would
///    use instead of the original field name
/// 4. You can specify many `short` and `long` names, any past the first one of each type will
///    become hidden aliases
/// 5. If `env(arg)` annotation is present - in addition to long/short names derived according to
///    rules 1..3 `bpaf` would also parse environment variable `arg` which can be a string literal
///    or an expression.
#[cfg_attr(not(doctest), doc = include_str!("docs2/named_arg_derive.md"))]
#[derive(Clone, Debug)]
pub struct NamedArg {
    pub(crate) short: Vec<char>,
    pub(crate) long: Vec<&'static str>,
    pub(crate) env: Vec<&'static str>,
    pub(crate) help: Option<Doc>,
}

impl NamedArg {
    pub(crate) fn flag_item(&self) -> Option<Item> {
        Some(Item::Flag {
            name: ShortLong::try_from(self).ok()?,
            help: self.help.clone(),
            env: self.env.first().copied(),
            shorts: self.short.clone(),
        })
    }
}

impl NamedArg {
    /// Add a short name to a flag/switch/argument
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }

    /// Add a long name to a flag/switch/argument
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
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
    #[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
    #[must_use]
    pub fn env(mut self, variable: &'static str) -> Self {
        self.env.push(variable);
        self
    }

    /// Add a help message to a `flag`/`switch`/`argument`
    ///
    /// `bpaf` converts doc comments and string into help by following those rules:
    /// 1. Everything up to the first blank line is included into a "short" help message
    /// 2. Everything is included into a "long" help message
    /// 3. `bpaf` preserves linebreaks followed by a line that starts with a space
    /// 4. Linebreaks are removed otherwise
    ///
    /// You can pass anything that can be converted into [`Doc`], if you are not using
    /// documentation generation functionality ([`doc`](crate::doc)) this can be `&str`.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/switch_help.md"))]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.help = Some(help.into());
        self
    }

    /// Simple boolean flag
    ///
    /// A special case of a [`flag`](NamedArg::flag) that gets decoded into a `bool`, mostly serves as a convenient
    /// shortcut to `.flag(true, false)`.
    ///
    /// In Derive API bpaf would use `switch` for `bool` fields inside named structs that don't
    /// have other consumer annotations ([`flag`](NamedArg::flag),
    /// [`argument`](NamedArg::argument), etc).
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/switch.md"))]
    #[must_use]
    pub fn switch(self) -> ParseFlag<bool> {
        build_flag_parser(true, Some(false), self)
    }

    /// Flag with custom present/absent values
    ///
    /// More generic version of [`switch`](NamedArg::switch) that can use arbitrary type instead of
    /// [`bool`].
    #[cfg_attr(not(doctest), doc = include_str!("docs2/flag.md"))]
    #[must_use]
    pub fn flag<T>(self, present: T, absent: T) -> ParseFlag<T>
    where
        T: Clone + 'static,
    {
        build_flag_parser(present, Some(absent), self)
    }

    /// Required flag with custom value
    ///
    /// Similar to [`flag`](NamedArg::flag) takes no option arguments, but would only
    /// succeed if user specifies its name on a command line.
    /// Works best in combination with other parsers.
    ///
    /// In derive style API `bpaf` would transform field-less enum variants into a parser
    /// that accepts one of it's variant names as `req_flag`. Additionally `bpaf` handles `()`
    /// fields as `req_flag`.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/req_flag.md"))]
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
    ///
    /// For `metavar` value you should pick something short and descriptive about the parameter,
    /// usually in capital letters. For example for an abstract file parameter it could be
    /// `"FILE"`, for a username - `"USER"`, etc.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/argument.md"))]
    ///
    /// You can further restrict it using [`adjacent`](ParseArgument::adjacent)
    #[must_use]
    pub fn argument<T>(self, metavar: &'static str) -> ParseArgument<T>
    where
        T: FromStr + 'static,
    {
        build_argument(self, metavar)
    }

    /// `adjacent` requires for the argument to be present in the same word as the flag:
    /// `-f bar` - no, `-fbar` or `-f=bar` - yes.
    pub(crate) fn matches_arg(&self, arg: &Arg, adjacent: bool) -> bool {
        match arg {
            Arg::Short(s, is_adj, _) => self.short.contains(s) && (!adjacent || *is_adj),
            Arg::Long(l, is_adj, _) => self.long.contains(&l.as_str()) && (!adjacent || *is_adj),
            Arg::ArgWord(_) | Arg::Word(_) | Arg::PosWord(_) => false,
        }
    }
}

impl<T> OptionParser<T> {
    /// Parse a subcommand
    ///
    /// Subcommands allow to use a totally independent parser inside a current one. Inner parser
    /// can have its own help message, description, version and so on. You can nest them arbitrarily
    /// too.
    ///
    /// # Important restriction
    /// When parsing command arguments from command lines you should have parsers for all your
    /// named values before parsers for commands and positional items. In derive API fields parsed as
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
    /// You can attach a single visible short alias and multiple hiddden short and long aliases
    /// using [`short`](ParseCommand::short) and [`long`](ParseCommand::long) methods.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/command.md"))]
    ///
    /// To represent multiple possible commands it is convenient to use enums
    #[cfg_attr(not(doctest), doc = include_str!("docs2/command_enum.md"))]
    #[must_use]
    pub fn command(self, name: &'static str) -> ParseCommand<T>
    where
        T: 'static,
    {
        ParseCommand {
            longs: vec![name],
            shorts: Vec::new(),
            help: self.short_descr().map(Into::into),
            subparser: self,
            adjacent: false,
        }
    }
}

/// Builder structure for the [`command`]
///
/// Created with [`command`], implements parser for the inner structure, gives access to [`help`](ParseCommand::help).
pub struct ParseCommand<T> {
    pub(crate) longs: Vec<&'static str>,
    pub(crate) shorts: Vec<char>,
    // short help!
    pub(crate) help: Option<Doc>,
    pub(crate) subparser: OptionParser<T>,
    pub(crate) adjacent: bool,
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
    ///     inner().command("mystery")
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
        M: Into<Doc>,
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

    /// Allow for the command to succeed even if there are non consumed items present
    ///
    /// Normally a subcommand parser should handle the rest of the unconsumed elements thus
    /// allowing only "vertical" chaining of commands. `adjacent` modifier lets command parser to
    /// succeed if there are leftovers for as long as all comsumed items form a single adjacent
    /// block. This opens possibilities to chain commands sequentially.
    ///
    /// Let's consider two examples with consumed items marked in bold :
    ///
    /// - <code>**cmd** **-a** -b **-c** -d</code>
    /// - <code>**cmd** **-a** **-c** -b -d</code>
    ///
    /// In the first example `-b` breaks the adjacency for all the consumed items so parsing will fail,
    /// while here in the second one the name and all the consumed items are adjacent to each other so
    /// parsing will succeed.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_command.md"))]
    #[must_use]
    pub fn adjacent(mut self) -> Self {
        self.adjacent = true;
        self
    }
}

impl<T> Parser<T> for ParseCommand<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
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
                return Err(Error(Message::Missing(Vec::new())));
            }

            if let Some(cur) = args.current {
                args.set_scope(cur..args.scope().end);
            }

            args.path.push(self.longs[0].to_string());
            if self.adjacent {
                let mut orig_args = args.clone();

                // narrow down the scope to adjacently available elements
                args.set_scope(args.adjacently_available_from(args.scope().start + 1));

                match self
                    .subparser
                    .run_subparser(args)
                    .map_err(Message::ParseFailure)
                {
                    Ok(ok) => {
                        args.set_scope(orig_args.scope());
                        Ok(ok)
                    }
                    Err(err) => {
                        let orig_scope = args.scope();
                        if let Some(narrow_scope) = args.adjacent_scope(&orig_args) {
                            orig_args.set_scope(narrow_scope);
                            if let Ok(res) = self.subparser.run_subparser(&mut orig_args) {
                                orig_args.set_scope(orig_scope);
                                std::mem::swap(&mut orig_args, args);
                                return Ok(res);
                            }
                        }
                        Err(Error(err))
                    }
                }
            } else {
                self.subparser
                    .run_subparser(args)
                    .map_err(|e| Error(Message::ParseFailure(e)))
            }
        } else {
            #[cfg(feature = "autocomplete")]
            args.push_command(self.longs[0], self.shorts.first().copied(), &self.help);

            let missing = MissingItem {
                item: self.item(),
                position: args.scope().start,
                scope: args.scope(),
            };
            Err(Error(Message::Missing(vec![missing])))
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
/// Parser for a named switch, created with [`NamedArg::flag`] or [`NamedArg::switch`]
pub struct ParseFlag<T> {
    present: T,
    absent: Option<T>,
    named: NamedArg,
}

impl<T: Clone + 'static> Parser<T> for ParseFlag<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
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
                None => {
                    if let Some(item) = self.named.flag_item() {
                        let missing = MissingItem {
                            item,
                            position: args.scope().start,
                            scope: args.scope(),
                        };
                        Err(Error(Message::Missing(vec![missing])))
                    } else if let Some(name) = self.named.env.first() {
                        Err(Error(Message::NoEnv(name)))
                    } else {
                        todo!("no key!")
                    }
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        if let Some(item) = self.named.flag_item() {
            item.required(self.absent.is_none())
        } else {
            Meta::Skip
        }
    }
}

impl<T> ParseFlag<T> {
    /// Add a help message to `flag`
    ///
    /// See [`NamedArg::help`]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.named.help = Some(help.into());
        self
    }
}

impl<T> ParseArgument<T> {
    /// Add a help message to an `argument`
    ///
    /// See [`NamedArg::help`]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.named.help = Some(help.into());
        self
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
    /// `-fbar` but not `--flag value`. Note, this is different from [`adjacent`](crate::ParseCon::adjacent),
    /// just plays a similar role.
    ///
    /// Should allow to parse some of the more unusual things
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_argument.md"))]
    #[must_use]
    pub fn adjacent(mut self) -> Self {
        self.adjacent = true;
        self
    }

    fn item(&self) -> Option<Item> {
        Some(Item::Argument {
            name: ShortLong::try_from(&self.named).ok()?,
            metavar: Metavar(self.metavar),
            env: self.named.env.first().copied(),
            help: self.named.help.clone(),
            shorts: self.named.short.clone(),
        })
    }

    fn take_argument(&self, args: &mut State) -> Result<OsString, Error> {
        match args.take_arg(&self.named, self.adjacent, Metavar(self.metavar)) {
            Ok(Some(w)) => {
                #[cfg(feature = "autocomplete")]
                if args.touching_last_remove() {
                    args.push_metavar(self.metavar, &self.named.help, true);
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
                    return Ok(val);
                }

                if let Some(item) = self.item() {
                    let missing = MissingItem {
                        item,
                        position: args.scope().start,
                        scope: args.scope(),
                    };
                    Err(Error(Message::Missing(vec![missing])))
                } else if let Some(name) = self.named.env.first() {
                    Err(Error(Message::NoEnv(name)))
                } else {
                    unreachable!()
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = self.take_argument(args)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }

    fn meta(&self) -> Meta {
        if let Some(item) = self.item() {
            Meta::from(item)
        } else {
            Meta::Skip
        }
    }
}

pub(crate) fn build_positional<T>(metavar: &'static str) -> ParsePositional<T> {
    ParsePositional {
        metavar,
        help: None,
        position: Position::Unrestricted,
        ty: PhantomData,
    }
}

/// Parse a positional item, created with [`positional`](crate::positional)
///
/// You can add extra information to positional parsers with [`help`](Self::help),
/// [`strict`](Self::strict), or [`non_strict`](Self::non_strict) on this struct.
#[derive(Clone)]
pub struct ParsePositional<T> {
    metavar: &'static str,
    help: Option<Doc>,
    position: Position,
    ty: PhantomData<T>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Position {
    Unrestricted,
    Strict,
    NonStrict,
}

impl<T> ParsePositional<T> {
    /// Add a help message to a [`positional`] parser
    ///
    /// `bpaf` converts doc comments and string into help by following those rules:
    /// 1. Everything up to the first blank line is included into a "short" help message
    /// 2. Everything is included into a "long" help message
    /// 3. `bpaf` preserves linebreaks followed by a line that starts with a space
    /// 4. Linebreaks are removed otherwise
    ///
    /// You can pass anything that can be converted into [`Doc`], if you are not using
    /// documentation generation functionality ([`doc`](crate::doc)) this can be `&str`.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.help = Some(help.into());
        self
    }

    /// Changes positional parser to be a "strict" positional
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
    ///
    /// here `cargo` takes a `--help` as a positional item and passes it to the example
    ///
    /// `bpaf` allows to require user to pass `--` for positional items with `strict` annotation.
    /// `bpaf` would display such positional elements differently in usage line as well.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/positional_strict.md"))]
    #[must_use]
    #[inline(always)]
    pub fn strict(mut self) -> ParsePositional<T> {
        self.position = Position::Strict;
        self
    }

    /// Changes positional parser to be a "not strict" positional
    ///
    /// Ensures the parser always rejects "strict" positions to the right of the separator, `--`.
    /// Essentially the inverse operation to [`ParsePositional::strict`], which can be used to ensure
    /// adjacent strict and nonstrict args never conflict with eachother.
    #[must_use]
    #[inline(always)]
    pub fn non_strict(mut self) -> Self {
        self.position = Position::NonStrict;
        self
    }

    #[inline(always)]
    fn meta(&self) -> Meta {
        let meta = Meta::from(Item::Positional {
            metavar: Metavar(self.metavar),
            help: self.help.clone(),
        });
        match self.position {
            Position::Strict => Meta::Strict(Box::new(meta)),
            _ => meta,
        }
    }
}

fn parse_pos_word(
    args: &mut State,
    metavar: Metavar,
    help: &Option<Doc>,
    position: Position,
) -> Result<OsString, Error> {
    match args.take_positional_word(metavar) {
        Ok((ix, is_strict, word)) => {
            match position {
                Position::Strict => {
                    if !is_strict {
                        #[cfg(feature = "autocomplete")]
                        args.push_pos_sep();
                        return Err(Error(Message::StrictPos(ix, metavar)));
                    }
                }
                Position::NonStrict => {
                    if is_strict {
                        return Err(Error(Message::NonStrictPos(ix, metavar)));
                    }
                }
                Position::Unrestricted => {}
            }

            #[cfg(feature = "autocomplete")]
            if args.touching_last_remove() && !args.check_no_pos_ahead() {
                args.push_metavar(metavar.0, help, false);
                args.set_no_pos_ahead();
            }
            Ok(word)
        }
        Err(err) => {
            #[cfg(feature = "autocomplete")]
            if !args.check_no_pos_ahead() {
                args.push_metavar(metavar.0, help, false);
                args.set_no_pos_ahead();
            }
            Err(err)
        }
    }
}

impl<T> Parser<T> for ParsePositional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = parse_pos_word(args, Metavar(self.metavar), &self.help, self.position)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }

    #[inline(always)]
    fn meta(&self) -> Meta {
        self.meta()
    }
}

/// Consume an arbitrary value that satisfies a condition, created with [`any`], implements
/// [`anywhere`](ParseAny::anywhere).
pub struct ParseAny<T> {
    pub(crate) metavar: Doc,
    pub(crate) help: Option<Doc>,
    pub(crate) check: Box<dyn Fn(OsString) -> Option<T>>,
    pub(crate) anywhere: bool,
}

impl<T> ParseAny<T> {
    pub(crate) fn item(&self) -> Item {
        Item::Any {
            metavar: self.metavar.clone(),
            help: self.help.clone(),
            anywhere: self.anywhere,
        }
    }

    /// Add a help message to [`any`] parser.
    /// See examples in [`any`]
    #[must_use]
    pub fn help<M: Into<Doc>>(mut self, help: M) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Replace metavar with a custom value
    /// See examples in [`any`]
    #[must_use]
    pub fn metavar<M: Into<Doc>>(mut self, metavar: M) -> Self {
        self.metavar = metavar.into();
        self
    }

    /// Try to apply the parser to each unconsumed element instead of just the front one
    ///
    /// By default `any` tries to parse just the front unconsumed item behaving similar to
    /// [`positional`] parser, `anywhere` changes it so it applies to every unconsumed item,
    /// similar to argument parser.
    ///
    /// See examples in [`any`]
    #[must_use]
    pub fn anywhere(mut self) -> Self {
        self.anywhere = true;
        self
    }
}

impl<T> Parser<T> for ParseAny<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        for (ix, x) in args.items_iter() {
            let (os, next) = match x {
                Arg::Short(_, next, os) | Arg::Long(_, next, os) => (os, *next),
                Arg::ArgWord(os) | Arg::Word(os) | Arg::PosWord(os) => (os, false),
            };
            if let Some(i) = (self.check)(os.clone()) {
                args.remove(ix);
                if next {
                    args.remove(ix + 1);
                }

                return Ok(i);
            }
            if !self.anywhere {
                break;
            }
        }
        let missing_item = MissingItem {
            item: self.item(),
            position: args.scope().start,
            scope: args.scope(),
        };
        Err(Error(Message::Missing(vec![missing_item])))
    }

    fn meta(&self) -> Meta {
        Meta::Item(Box::new(self.item()))
    }
}
