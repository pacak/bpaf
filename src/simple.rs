use crate::{
    error::Message,
    from_os_str::parse_os_str,
    params::{build_argument, build_flag_parser, Anything, Argument, Flag, Named},
    parsers::{Command, Positional},
    structs::{parse_option, Collect, Many, Many1, Optional, Pure, PureWith},
    Doc, Error, Meta, Parser, State,
};
use std::{marker::PhantomData, str::FromStr};

#[cfg(doc)]
use crate::{construct, pure, pure_with, OptionParser};

/// A basic building block for your parsers
///
/// This structure implements different methods depending on how it was created - pay attention to
/// the type parameter. Some versions of the structure also implement [`Parser`](crate::Parser) trait.
///
/// Depending on a purpose you should use one of those constructors to make `SimpleParser` and
/// later consume it using one of the methods listed later. Normally you don't keep `SimpleParser`
/// around long enough to have it visible in the API. With combinatoric usage you chain multiple
/// methods producing something that implements [`Parser`] trait and use `impl Parser<X>` in type
/// signature. With derive API proc macro takes care of it.
///
/// ## Ways to construct `SimpleParser`
///
/// For named parsers (arguments or flags) you use one of those constructors:
///
/// - [`short`] or its alias - [`SimpleParser::with_short`]
/// - [`long`] or its alias - [`SimpleParser::with_long`]
/// - [`env`](env()) or its alias - [`SimpleParser::with_env`]
/// - For positional items the parser is [`positional`] or its alias [`SimpleParser::positional`]
/// - To parse something unusual you can use [`any`] or its alias [`SimpleParser::with_any`]
///
/// # Ways to consume `SimpleParser`
///
/// ## Create a parser flag
///
/// A [`flag`](SimpleParser::flag) is a string that consists of two dashes and a name (`--flag`) or
/// single dash and a single character (`-f`) created from a Named parser, made with [`long`] or
/// [`short`] functions respectively. Depending if this name is present or absent on the command
/// line primitive flag parser produces one of two values. User can combine several short flags in
/// a single invocation: `-a -b -c` is the same as `-abc`.
///
/// Once you have a `SimpleParser<Named>` you can add more short or long names with
/// [`long`](SimpleParser::long) and [`short`](SimpleParser::short). First name of each type will
/// be visible and more become hidden aliases.
///
/// ## Create a parser for required flag
///
/// Similar to `flag`, but instead of falling back to the second value required flag parser would
/// fail with "value not found" error. Mostly useful in combination with other parsers, created
/// with [`SimpleParser::req_flag`].
///
/// ## Create a parser for switch
///
/// A special case of a flag that gets decoded into a `bool`, mostly serves as a convenient
/// shortcut to `.flag(true, false)`. Created with [`SimpleParser::switch`].
///
/// ## Create an argument parser
///
/// An [`argument`](SimpleParser::argument) is a short or long name, (see `flag`) followed by
/// either a space or `=` and then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all
/// valid argument examples. Note, string literal can't start with a dash (`-`) unless separated
/// from the flag with `=`. For short flags value can follow immediately: `-fbar`.
///
/// ## Create a positional
///
/// A [`positional`] argument is an argument with no additonal name, for example in `vim main.rs`
/// `main.rs` is a positional argument. They can't start with `-`. Parsers created by `bpaf` will
/// treat anything to the right of `--` string on a command line as a positional item - with this
/// it is possible for users to have positional items that do start with `-`. `cat -- --help` -
/// will try to show file called `--help`.
///
/// ## Parse an arbitrary item from a command line
///
/// Also a positional argument with no additional name, but unlike [`positional`] itself, [`any`]
/// isn't restricted to positional looking structure and would consume any items as they appear on
/// a command line. Can be useful to collect anything unused to pass to other applications.
///
/// ## Parse a subcommand
///
/// A command defines a starting point for an independent subparser. Name must be a valid utf8
/// string. For example `cargo build` invokes command `"build"` and after `"build"` `cargo` starts
/// accepting values it won't accept otherwise. To create a subcommand parser you start by creating
/// an [`OptionParser`] that handles all the arguments then calling [`OptionParser::command`] to go
/// back to `SimpleParser`.
///
#[derive(Debug, Clone)]
pub struct SimpleParser<I>(pub(crate) I);

impl SimpleParser<Named> {
    /// Create a parser that has a short name
    ///
    /// **This is an alias for [`short`] standalone function**, and exists to have all the
    /// constructors for `SimpleParser` collected in one place. You shouldn't use it directly.
    pub fn with_short(name: char) -> Self {
        short(name)
    }
}

impl SimpleParser<Named> {
    /// Create a parser that has a long name
    ///
    /// **This is an alias for [`long`] standalone function**, and exists to have all the
    /// constructors for `SimpleParser` collected in one place. You shouldn't use it directly.
    pub fn with_long(name: &'static str) -> Self {
        long(name)
    }
}

impl SimpleParser<Named> {
    /// Add a short name to a named parser
    ///
    /// This method will add a short name to a named parser. `bpaf` would use first name as a
    /// visible name and from second onwards as hidden aliases. You can chain this method multiple
    /// times.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/short_alias.md"))]
    pub fn short(mut self, short_name: char) -> Self {
        self.0.short.push(short_name);
        self
    }
}

impl SimpleParser<Named> {
    /// Add a long name to a named parser
    ///
    /// This method will add a long name to a named parser. `bpaf` would use first name as a
    /// visible name and from second onwards as hidden aliases. You can chain this method multiple
    /// times.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/long_alias.md"))]
    pub fn long(mut self, long_name: &'static str) -> Self {
        self.0.long.push(long_name);
        self
    }
}

impl SimpleParser<Named> {
    /// Create a parser for an environment variable
    ///
    /// **This is an alias for [`env`](env()) standalone function**, and exists to have all the
    /// constructors for `SimpleParser` collected in one place. You shouldn't use it directly.
    pub fn with_env(name: &'static str) -> Self {
        Self(Named {
            short: Vec::new(),
            long: Vec::new(),
            env: vec![name],
            help: None,
        })
    }
}

impl SimpleParser<Named> {
    /// Add a fallback to an environment to a named parser
    ///
    /// Parser will try to consume command line items first, if they are not present - it will try
    /// to fallback to an environment variable. You can specify it multiple times, `bpaf` would use
    /// items past the first one as hidden aliases.
    ///
    /// For [`flag`](SimpleParser::flag) and [`switch`](SimpleParser::switch) environment variable
    /// being present gives the same result as the flag being present, allowing to implement things
    /// like `NO_COLOR` variables:
    ///
    /// ```console
    /// $ NO_COLOR=1 app --do-something
    /// ```
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/env.md"))]
    pub fn env(mut self, name: &'static str) -> Self {
        self.0.env.push(name);
        self
    }
}

impl SimpleParser<Named> {
    /// Add a help message to a named parser
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
    #[cfg_attr(not(doctest), doc = include_str!("_docs/switch_help.md"))]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
        self
    }
}

impl SimpleParser<Named> {
    /// Simple boolean flag
    ///
    /// A special case of a [`flag`](SimpleParser::flag) that gets decoded into a `bool`, mostly
    /// serves as a convenient shortcut to `.flag(true, false)`.
    ///
    /// In Derive API bpaf would use `switch` for `bool` fields inside named structs that don't
    /// have other consumer annotations such as [`flag`](SimpleParser::flag),
    /// [`argument`](SimpleParser::argument), etc.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/switch.md"))]
    #[must_use]
    pub fn switch(self) -> SimpleParser<Flag<bool>> {
        SimpleParser(build_flag_parser(true, Some(false), self.0))
    }
}

impl SimpleParser<Named> {
    /// Flag with custom present/absent values
    ///
    /// This is a more generic version of [`switch`](SimpleParser::switch). With `flag` you can
    /// specify two values of the same type and the parser will return first one if flag is present
    /// on the command line and second one if it's absent.
    ///
    /// There are two typical use cases:
    /// - implementing flags that disable something `--no-logging`. Instead of using switchig and
    ///   flipping the boolean value with [`Parser::map`] you can use `flag(false, true)`
    /// - implementing flags that use custom enum instead of boolean
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/flag.md"))]
    #[must_use]
    pub fn flag<V>(self, present: V, absent: V) -> SimpleParser<Flag<V>>
    where
        V: Clone + 'static,
    {
        SimpleParser(build_flag_parser(present, Some(absent), self.0))
    }
}

impl SimpleParser<Named> {
    /// Required flag with custom value
    ///
    /// Similar to [`flag`](SimpleParser::flag) required flag consumed no option arguments, but
    /// would only succeed if user specifies its name on a command line. Works best in
    /// combination with other parsers.
    ///
    /// In derive style API `bpaf` would transform field-less enum variants into a parser
    /// that accepts one of it's variant names as `req_flag`. Additionally `bpaf` handles `()`
    /// fields as `req_flag`.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/req_flag.md"))]
    #[must_use]
    pub fn req_flag<V>(self, present: V) -> SimpleParser<Flag<V>>
    where
        V: Clone + 'static,
    {
        SimpleParser(build_flag_parser(present, None, self.0))
    }
}

impl SimpleParser<Named> {
    /// Parse a named option argument
    ///
    /// This parser consumes a short (`-a`) or long (`--name`) name followed by  either a space or
    /// `=` and then by a string literal.  `-f foo`, `--flag bar` or `-o=-` are all valid argument
    /// examples. Note, string literal can't start with `-` unless separated from the flag with
    /// `=`: `-n=-3`. For short flags value can follow immediately: `-fbar`.
    ///
    /// When using combinatoring API you can specify the type with turbofish, for parsing types
    /// that don't implement [`FromStr`] you can use consume a `String`/`OsString` first and parse
    /// it by hands.
    ///
    /// For `metavar` value you should pick something short and descriptive about the parameter,
    /// usually in capital letters. For example for an abstract file parameter it could be
    /// `"FILE"`, for a username - `"USER"`, etc.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/argument.md"))]
    ///
    /// You can further restrict it using [`adjacent`](SimpleParser::adjacent)
    #[must_use]
    pub fn argument<T>(self, metavar: &'static str) -> SimpleParser<Argument<T>>
    where
        T: FromStr + 'static,
    {
        SimpleParser(build_argument(self.0, metavar))
    }
}

impl<T> SimpleParser<Flag<T>> {
    /// Add a help message to a flag parser
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
    #[cfg_attr(not(doctest), doc = include_str!("_docs/switch_help.md"))]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.named.help = Some(help.into());
        self
    }
}

impl<T> SimpleParser<Argument<T>> {
    /// Restrict parsed arguments to have both flag and a value in the same word:
    ///
    /// In other words if you restruct `SimpleParser<Argument>` parser with `adjacent` it will
    /// accept `-fbar` or `--flag=bar` but not `--flag value`. Note, this is different from
    /// [`adjacent`](crate::ParseCon::adjacent), but plays a similar role.
    ///
    /// Should allow to parse some of the more unusual things and might require users to be more
    /// specific.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/adjacent_argument.md"))]
    #[must_use]
    pub fn adjacent(mut self) -> Self {
        self.0.adjacent = true;
        self
    }
}

impl<T> SimpleParser<Pure<T>> {
    /// Parser that produces a fixed value
    ///
    /// This parser produces `T` without consuming anything from the command line, which can be useful
    /// with [`construct!`]. As with any parsers, `T` should be `Clone` and `Debug`.
    ///
    /// Both `pure` and [`pure_with`] are designed to put values into structures, to generate fallback
    /// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
    ///
    /// See also [`pure_with`] for a pure computation that can fail.
    ///
    /// **This is an alias for [`pure`] standalone function**, and exists to have all the constructors
    /// for [`SimpleParser`] collected in one place. You shouldn’t use it directly.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/pure.md"))]
    pub fn with_pure(val: T) -> Self {
        Self(Pure(val))
    }
}

impl<T, F, E> SimpleParser<PureWith<T, F, E>>
where
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    /// Wrap a calculated value into a `Parser`
    ///
    /// This parser represents a possibly failing equivalent to [`pure`].
    /// It produces `T` by invoking the provided callback without consuming anything from the command
    /// line, which can be useful with [`construct!`]. As with any parsers, `T` should be `Clone`
    /// and `Debug`.
    ///
    /// Both [`pure`] and `pure_with` are designed to put values into structures, to generate fallback
    /// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
    ///
    /// See also [`pure`] for a pure computation that can't fail.
    ///
    /// **This is an alias for [`pure_with`] standalone function**, and exists to have all the constructors
    /// for [`SimpleParser`] collected in one place. You shouldn’t use it directly.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/pure_with.md"))]
    pub fn with_pure_with(val: F) -> PureWith<T, F, E> {
        PureWith(val)
    }
}

impl<T: Clone + 'static> Parser<T> for SimpleParser<Flag<T>> {
    fn eval(&self, args: &mut crate::State) -> Result<T, crate::Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> crate::Meta {
        self.0.meta()
    }
}

impl<T> Parser<T> for SimpleParser<Positional<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut crate::State) -> Result<T, crate::Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> crate::Meta {
        self.0.meta()
    }
}

impl<T> Parser<T> for SimpleParser<Argument<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = self.0.take_argument(args)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }

    fn meta(&self) -> Meta {
        if let Some(item) = self.0.item() {
            Meta::from(item)
        } else {
            Meta::Skip
        }
    }
}

/// Parse a [`flag`](SimpleParser::flag)/[`switch`](SimpleParser::switch)/[`argument`](SimpleParser::argument) that has a short name
///
/// You can chain multiple [`short`](SimpleParser::short), [`long`](SimpleParser::long) and
/// [`env`](SimpleParser::env()) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn short(name: char) -> SimpleParser<Named> {
    SimpleParser(Named {
        short: vec![name],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    })
}

/// Parse a [`flag`](SimpleParser::flag)/[`switch`](SimpleParser::switch)/[`argument`](SimpleParser::argument) that has a long name
///
/// You can chain multiple [`short`](SimpleParser::short), [`long`](SimpleParser::long) and
/// [`env`](SimpleParser::env()) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn long(name: &'static str) -> SimpleParser<Named> {
    SimpleParser(Named {
        long: vec![name],
        env: Vec::new(),
        short: Vec::new(),
        help: None,
    })
}

/// Parse an environment variable
///
/// This parser lets you to consume a value from an environment variable. For
/// [`argument`](SimpleParser::argument) `env` parser produces the value itself, for
/// [`flag`](SimpleParser::flag) and [`switch`](SimpleParser::switch) environment variable
/// being present gives the same result as the flag being present, allowing to implement things
/// like `NO_COLOR` variables:
///
/// ```console
/// $ NO_COLOR=1 app --do-something
/// ```
///
/// Since env parser is a name parser you can also add a name - short or long one. If parser
/// succeeds parsing by name - this result takes a priority.
///
/// If you don't specify a short or a long name - whole argument is going to be absent from the
/// help message. Use it combined with a named or positional argument to have a hidden fallback
/// that wouldn't leak sensitive info.
///
/// You can chain multiple [`short`](SimpleParser::short), [`long`](SimpleParser::long) and
/// [`env`](SimpleParser::env()) for multiple names. You can specify multiple names of the same type,
/// `bpaf` would use items past the first one as hidden aliases.
#[cfg_attr(not(doctest), doc = include_str!("_docs/env.md"))]
#[must_use]
pub fn env(variable: &'static str) -> SimpleParser<Named> {
    SimpleParser(Named {
        short: Vec::new(),
        long: Vec::new(),
        help: None,
        env: vec![variable],
    })
}

/// Parse a positional argument
///
/// A positional argument is a type of argument that is passed to a command line program without
/// using any special flags or prefixes. The order of positional arguments is important, and the
/// program will expect them to be passed in a specific order. For example `ls` takes positional
/// arguments to specify files or directories that should be listed and lists those specified
/// earlier first.
///
#[cfg_attr(not(doctest), doc = include_str!("_docs/positional.md"))]
///
/// ## See also
///
/// [`strict`](SimpleParser::strict) to require user to use `--`
#[must_use]
pub fn positional<T>(metavar: &'static str) -> SimpleParser<Positional<T>> {
    SimpleParser(Positional {
        metavar,
        help: None,
        result_type: PhantomData,
        strict: false,
    })
}

impl<T> SimpleParser<Positional<T>> {
    /// Parse a positional argument
    ///
    /// This member function exists to have all the ways to construct [`SimpleParser`] in one
    /// place. You should use standalone [`positional`] function instead.
    pub fn positional(metavar: &'static str) -> Self {
        SimpleParser(Positional {
            metavar,
            help: None,
            result_type: PhantomData,
            strict: false,
        })
    }

    /// Add a help message to a positional parser
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
    #[cfg_attr(not(doctest), doc = include_str!("_docs/pos_help.md"))]
    #[must_use]
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
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
        self.0.strict = true;
        self
    }
}

/// Parse a single arbitrary item from a command line
///
/// **`any` is designed to consume items that don't fit into usual
/// flag/switch/argument/positional/command classification, in most cases you don't need to use
/// it**.
///
/// Type parameter `I` is used for intermediate value, normally you'd use [`String`] or
/// [`OsString`](std::ffi::OsString). This parameter only exists to make it possible to work with
/// non-utf8 encoded arguments such as some rare file names, as well as not having to deal with
/// `OsString` if all you want to process is a string that utf8 can correctly represent.
///
/// Type parameter `T` is the type the parser actually produces.
///
/// Parameter `check` takes an intermediate value (`String` or `OsString`) and decides if `any`
/// parser is going to take it by returning `Some` value or `None` if this is not an expected value
/// for this parser.
///
/// By default, `any` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct, after all the named parsers and it will only try to parse
/// the first unconsumed item on the command line. It is possible to lift this restriction by
/// calling [`anywhere`](SimpleParser::anywhere) on the parser.
///
pub fn any<I, T, F>(metavar: &str, check: F) -> SimpleParser<Anything<T>>
where
    I: FromStr + 'static,
    F: Fn(I) -> Option<T> + 'static,

    <I as std::str::FromStr>::Err: std::fmt::Display,
{
    SimpleParser(Anything {
        metavar: [(metavar, crate::buffer::Style::Metavar)][..].into(),
        help: None,
        check: Box::new(move |os: std::ffi::OsString| {
            match crate::from_os_str::parse_os_str::<I>(os) {
                Ok(v) => check(v),
                Err(_) => None,
            }
        }),
        anywhere: false,
    })
}

impl<T> SimpleParser<Anything<T>> {
    /// Parse a single arbitrary item from a command line
    ///
    /// **This is an alias for [`any`] standalone function**, and exists to have all the
    /// constructors for `SimpleParser` collected in one place. You shouldn't use it directly.
    pub fn with_any<F, I>(metavar: &str, check: F) -> Self
    where
        I: FromStr + 'static,
        F: Fn(I) -> Option<T> + 'static,

        <I as std::str::FromStr>::Err: std::fmt::Display,
    {
        any(metavar, check)
    }

    pub fn anywhere(mut self) -> Self {
        self.0.anywhere = true;
        self
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<Doc>,
    {
        self.0.help = Some(help.into());
        self
    }

    /// Replace metavar with a custom value
    /// See examples in [`any`]
    #[must_use]
    pub fn metavar<M: Into<Doc>>(mut self, metavar: M) -> Self {
        self.0.metavar = metavar.into();
        self
    }
}

impl<T> Parser<T> for SimpleParser<Anything<T>> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> Meta {
        self.0.meta()
    }
}

/// A specialized version of [`any`] that consumes an arbitrary string
///
/// By default `literal` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct and it will only try to parse the first unconsumed
/// item on the command line. It is possible to lift this restriction by calling
/// [`anywhere`](SimpleParser::anywhere) on the parser.
///
/// Apart from matching to a specific literal, this function behaves similarly to:
/// [`req_flag`](SimpleParser::req_flag) it produces a value it was given or fails with "item not
/// found" error which you can handle with [`fallback`](Parser::fallback),
/// [`optional`](Parser::optional) or by combining several `literal` parsers together.
///
#[cfg_attr(not(doctest), doc = include_str!("_docs/literal.md"))]
///
/// # See also
/// - [`any`] - a generic version of `literal` that uses function to decide if value is to be parsed
/// or not.
/// - [`req_flag`](SimpleParser::req_flag) - parse a short/long flag from a command line or fail with "item not found"
#[must_use]
pub fn literal<T>(literal: &'static str, value: T) -> SimpleParser<Anything<T>>
where
    T: Clone + 'static,
{
    SimpleParser(Anything {
        metavar: [(literal, crate::buffer::Style::Literal)][..].into(),
        help: None,
        check: Box::new(move |os| {
            if os == literal {
                Some(value.clone())
            } else {
                None
            }
        }),
        anywhere: false,
    })
}

impl<T> Parser<T> for SimpleParser<Command<T>> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.0.eval(args)
    }

    fn meta(&self) -> Meta {
        self.0.meta()
    }
}

impl<P, T> Parser<Option<T>> for SimpleParser<Optional<P>>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<Option<T>, Error> {
        let mut len = usize::MAX;
        crate::structs::parse_option(&self.0.inner, &mut len, args, self.0.catch)
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.0.inner.meta()))
    }
}

impl<P> SimpleParser<Optional<P>> {
    #[must_use]
    /// Handle parse failures for optional parsers
    ///
    /// Can be useful to decide to skip parsing of some items on a command line.
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several parser types support this attribute: `Optional`, `Many`
    /// and `Some`, behavior should be similar for all of them.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/optional_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.0.catch = true;
        self
    }
}

impl<P, T> Parser<Vec<T>> for SimpleParser<Many<P>>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<Vec<T>, Error> {
        let mut len = usize::MAX;
        std::iter::from_fn(|| parse_option(&self.0.inner, &mut len, args, self.0.catch).transpose())
            .collect()
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Optional(Box::new(self.0.inner.meta()))))
    }
}

impl<P> SimpleParser<Many<P>> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/many_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.0.catch = true;
        self
    }
}

impl<P, T> Parser<Vec<T>> for SimpleParser<Many1<P>>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<Vec<T>, Error> {
        let mut len = usize::MAX;
        let r = std::iter::from_fn(|| {
            parse_option(&self.0.inner, &mut len, args, self.0.catch).transpose()
        })
        .collect();

        if len == usize::MAX {
            Err(Error(Message::ParseSome(self.0.message)))
        } else {
            r
        }
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Required(Box::new(self.0.inner.meta()))))
    }
}

impl<P> SimpleParser<Many1<P>> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/many1_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.0.catch = true;
        self
    }
}

impl<P, T, C> Parser<C> for SimpleParser<Collect<P, C, T>>
where
    P: Parser<T>,
    C: FromIterator<T>,
{
    fn eval(&self, args: &mut State) -> Result<C, Error> {
        let mut len = usize::MAX;
        std::iter::from_fn(|| parse_option(&self.0.inner, &mut len, args, self.0.catch).transpose())
            .collect()
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Required(Box::new(self.0.inner.meta()))))
    }
}

impl<P, T, C> SimpleParser<Collect<P, C, T>> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("_docs/collect_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.0.catch = true;
        self
    }
}
