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
    buffer::Style,
    error::{Message, MissingItem},
    from_os_str::parse_os_str,
    item::ShortLong,
    meta_help::Metavar,
    Doc, Error, Item, Meta, OptionParser, Parser,
};

/// A named thing used to create [`flag`](NamedArg::flag), [`switch`](NamedArg::switch) or
/// [`argument`](NamedArg::argument)
///
/// # Combinatoric usage
///
/// Named items (`argument`, `flag` and `switch`) can have up to 2 visible names (one short and one long)
/// and multiple hidden short and long aliases if needed. It's also possible to consume items from
/// environment variables using [`env`](NamedArg::env). You usually start with [`short`] or [`long`]
/// function, then apply [`short`](NamedArg::short) / [`long`](NamedArg::long) / [`env`](NamedArg::env) /
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
    env: Vec<&'static str>,
    pub(crate) help: Option<Doc>,
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

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a short name
///
/// You can chain multiple of [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn short(short: char) -> NamedArg {
    NamedArg {
        short: vec![short],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    }
}

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a long name
///
/// You can chain multiple of [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn long(long: &'static str) -> NamedArg {
    NamedArg {
        short: Vec::new(),
        long: vec![long],
        env: Vec::new(),
        help: None,
    }
}

/// Parse an environment variable
///
/// You can chain multiple of [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
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
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
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
    pub fn switch(self) -> impl Parser<bool> {
        build_flag_parser(true, Some(false), self)
    }

    /// Flag with custom present/absent values
    ///
    /// More generic version of [`switch`](NamedArg::switch) that can use arbitrary type instead of
    /// [`bool`].
    #[cfg_attr(not(doctest), doc = include_str!("docs2/flag.md"))]
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
    /// succeed if user specifies its name on a command line.
    /// Wworks best in combination with other parsers.
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

/// Parse a positional argument
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
/// Without using `--` `bpaf` would only accept items that don't start with `-` as positional, you
/// can use [`any`] to work around this restriction.
///
/// By default `bpaf` accepts positional items with or without `--` where values permit, you can
/// further restrict the parser to accept positionals only on the right side of `--` using
/// [`strict`](ParsePositional::strict).
#[cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))]
#[must_use]
pub fn positional<T>(metavar: &'static str) -> ParsePositional<T> {
    build_positional(metavar)
}

#[doc(hidden)]
#[deprecated = "You should switch from command(name, sub) to sub.command(name)"]
pub fn command<T>(name: &'static str, subparser: OptionParser<T>) -> ParseCommand<T>
where
    T: 'static,
{
    ParseCommand {
        longs: vec![name],
        shorts: Vec::new(),
        help: subparser.short_descr().map(Into::into),
        subparser,
        adjacent: false,
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
    longs: Vec<&'static str>,
    shorts: Vec<char>,
    // short help!
    help: Option<Doc>,
    subparser: OptionParser<T>,
    adjacent: bool,
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
struct ParseFlag<T> {
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
                    let missing = MissingItem {
                        item: self.named.flag_item(),
                        position: args.scope().start,
                        scope: args.scope(),
                    };
                    Err(Error(Message::Missing(vec![missing])))
                }
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

    fn item(&self) -> Item {
        Item::Argument {
            name: ShortLong::from(&self.named),
            metavar: Metavar(self.metavar),
            env: self.named.env.first().copied(),
            help: self.named.help.clone(),
            shorts: self.named.short.clone(),
        }
    }

    fn take_argument(&self, args: &mut State) -> Result<OsString, Error> {
        if self.named.short.is_empty() && self.named.long.is_empty() {
            if let Some(name) = self.named.env.first() {
                return Err(Error(Message::NoEnv(name)));
            }
        }
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
                    Ok(val)
                } else {
                    let missing = MissingItem {
                        item: self.item(),
                        position: args.scope().start,
                        scope: args.scope(),
                    };
                    Err(Error(Message::Missing(vec![missing])))
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
    help: Option<Doc>,
    result_type: PhantomData<T>,
    strict: bool,
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
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    fn meta(&self) -> Meta {
        let meta = Meta::from(Item::Positional {
            metavar: Metavar(self.metavar),
            help: self.help.clone(),
        });
        if self.strict {
            Meta::Strict(Box::new(meta))
        } else {
            meta
        }
    }
}

fn parse_pos_word(
    args: &mut State,
    strict: bool,
    metavar: &'static str,
    help: &Option<Doc>,
) -> Result<OsString, Error> {
    let metavar = Metavar(metavar);
    if let Some((ix, is_strict, word)) = args.take_positional_word(metavar)? {
        if strict && !is_strict {
            #[cfg(feature = "autocomplete")]
            args.push_pos_sep();

            return Err(Error(Message::StrictPos(ix, metavar)));
        }
        #[cfg(feature = "autocomplete")]
        if args.touching_last_remove() && !args.check_no_pos_ahead() {
            args.push_metavar(metavar.0, help, false);
            args.set_no_pos_ahead();
        }
        Ok(word)
    } else {
        #[cfg(feature = "autocomplete")]
        if !args.check_no_pos_ahead() {
            args.push_metavar(metavar.0, help, false);
            args.set_no_pos_ahead();
        }

        let position = args.items_iter().next().map_or(args.scope().end, |x| x.0);
        let missing = MissingItem {
            item: Item::Positional {
                metavar,
                help: help.clone(),
            },
            position,
            scope: args.scope(),
        };
        Err(Error(Message::Missing(vec![missing])))
    }
}

impl<T> Parser<T> for ParsePositional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = parse_pos_word(args, self.strict, self.metavar, &self.help)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }

    fn meta(&self) -> Meta {
        self.meta()
    }
}

/// Consume an arbitrary value that satisfies a condition, created with [`any`], implements
/// [`anywhere`](ParseAny::anywhere).
pub struct ParseAny<T, I, F> {
    metavar: Doc,
    help: Option<Doc>,
    ctx: PhantomData<(I, T)>,
    check: F,
    anywhere: bool,
}

/// Parse a single arbitrary item from a command line
///
/// **`any` is designed to consume items that don't fit into usual [`flag`](NamedArg::flag)
/// /[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument)/[`positional`]/
/// [`command`](OptionParser::command) classification, in most cases you don't need to use it**
///
/// By default `any` behaves similar to [`positional`] so you should be using it near the right
/// most end of the consumer struct and it will only try to parse first unconsumed item on the
/// command line. It is possible to lift this restriction by calling
/// [`anywhere`](ParseAny::anywhere) on the parser.
///
/// `check` argument is a function from any type `I` that implements `FromStr` to `T`.
/// Usually this should be `String` or `OsString`, but feel free to experiment. When
/// running `any` tries to parse an item on a command line into that `I` and applies the `check`
/// function. If `check` succeeds - parser `any` succeeds and produces `T`, otherwise it behaves
/// as if it haven't seen it. If `any` works in `anywhere` mode - it will try to parse all other
/// unconsumed items, otherwise `any` fails.
///
/// # Use `any` to capture remaining arguments
/// Normally you would use [`positional`] with [`strict`](ParsePositional::strict) annotation for
/// that, but using any allows to blur the boundary between arguments for child process and self
/// process a bit more.
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_simple.md"))]
///
/// # Use `any` to parse a non standard flag
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_switch.md"))]
///
/// # Use `any` to parse a non standard argument
/// Normally `any` would try to display itself as a usual metavariable in the usage line and
/// generated help, you can customize that with [`metavar`](ParseAny::metavar) method:
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_literal.md"))]
///
/// # See also
/// [`literal`] - a specialized version of `any` that tries to parse a fixed literal
#[must_use]
pub fn any<I, T, F>(metavar: &str, check: F) -> ParseAny<T, I, F>
where
    I: FromStr + 'static,
    F: Fn(I) -> Option<T>,
{
    ParseAny {
        metavar: [(metavar, Style::Metavar)][..].into(),
        help: None,
        check,
        ctx: PhantomData,
        anywhere: false,
    }
}

/// A specialized version of [`any`] that consumes an arbitrary string
///
/// By default `literal` behaves similar to [`positional`] so you should be using it near the right
/// most end of the consumer struct and it will only try to parse first unconsumed item on the
/// command line. It is possible to lift this restriction by calling
/// [`anywhere`](ParseAny::anywhere) on the parser.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_literal.md"))]
///
/// # See also
/// [`any`] - a generic version of `literal` that uses function to decide if value is to be parsed
/// or not.
#[must_use]
pub fn literal(val: &'static str) -> ParseAny<(), String, impl Fn(String) -> Option<()>> {
    any("", move |s| if s == val { Some(()) } else { None })
        .metavar(&[(val, crate::buffer::Style::Literal)][..])
}

impl<T, I, F> ParseAny<T, I, F> {
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
    /// See examples in [`any`]
    #[must_use]
    pub fn anywhere(mut self) -> Self {
        self.anywhere = true;
        self
    }
}

impl<F, I, T> Parser<T> for ParseAny<T, I, F>
where
    I: FromStr + 'static,
    F: Fn(I) -> Option<T>,
    <I as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        for (ix, x) in args.items_iter() {
            let (os, next) = match x {
                Arg::Short(_, next, os) | Arg::Long(_, next, os) => (os, *next),
                Arg::ArgWord(os) | Arg::Word(os) | Arg::PosWord(os) => (os, false),
            };
            if let Ok(i) = parse_os_str::<I>(os.clone()) {
                if let Some(t) = (self.check)(i) {
                    args.remove(ix);
                    if next {
                        args.remove(ix + 1);
                    }

                    return Ok(t);
                }
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
