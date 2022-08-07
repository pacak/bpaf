//! Help message generation and rendering

#![allow(clippy::write_with_newline)]
use std::marker::PhantomData;

use crate::{
    args::{self, Args},
    item::Item,
    params::short,
    Meta, ParseConstruct, ParseFailure, Parser,
};

/// Unsuccessful command line parsing outcome, internal representation
#[derive(Clone, Debug)]
pub enum Error {
    /// Terminate and print this to stdout
    Stdout(String),
    /// Terminate and print this to stderr
    Stderr(String),
    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<Meta>),
}

impl Error {
    #[doc(hidden)]
    #[must_use]
    pub fn combine_with(self, other: Self) -> Self {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // help output takes priority
            (a @ Error::Stdout(_), _) => a,
            (_, b @ Error::Stdout(_)) => b,

            // parsing failure takes priority
            (a @ Error::Stderr(_), _) => a,
            (_, b @ Error::Stderr(_)) => b,

            // missing elements are combined
            (Error::Missing(mut a), Error::Missing(mut b)) => {
                a.append(&mut b);
                Error::Missing(a)
            }
        }
    }
}

/// Information about the parser
///
/// ```rust
/// # use bpaf::*;
/// let info = Info::default()
///                .version(env!("CARGO_PKG_VERSION"))
///                .descr("Does mothing")
///                .footer("Beware of the Leopard");
/// # drop(info);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Info {
    /// version field, see [`version`][Info::version]
    pub version: Option<&'static str>,
    /// Custom description field, see [`descr`][Info::descr]
    pub descr: Option<&'static str>,
    /// Custom header field, see [`header`][Info::header]
    pub header: Option<&'static str>,
    /// Custom footer field, see [`footer`][Info::footer]
    pub footer: Option<&'static str>,
    /// Custom usage field, see [`usage`][Info::usage]
    pub usage: Option<&'static str>,
}

impl Info {
    /// Set a version field.
    ///
    /// By default bpaf won't include any version info and won't accept `--version` switch
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let info = Info::default().version(env!("CARGO_PKG_VERSION"));
    /// # drop(info);
    /// ```
    #[must_use]
    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    /// Set a program description
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let info = Info::default().descr("This program calculates rectangle's area");
    /// # drop(info);
    /// ```
    /// See complete example in `examples/rectangle.rs`
    #[must_use]
    pub const fn descr(mut self, descr: &'static str) -> Self {
        self.descr = Some(descr);
        self
    }

    /// Set a custom header before all the options
    /// ```rust
    /// # use bpaf::*;
    /// let info = Info::default().header("header");
    /// # drop(info);
    /// ```
    /// See complete example in `examples/rectangle.rs`
    #[must_use]
    pub const fn header(mut self, header: &'static str) -> Self {
        self.header = Some(header);
        self
    }

    /// Set a custom header after all the options
    /// ```rust
    /// # use bpaf::*;
    /// let info = Info::default().header("footer");
    /// # drop(info);
    /// ```
    /// See complete example in `examples/rectangle.rs`
    #[must_use]
    pub const fn footer(mut self, footer: &'static str) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Replace generated usage string with a custom one
    /// ```rust
    /// # use bpaf::*;
    /// let info = Info::default().usage("example [-v] -w <PX> -h <PX>");
    /// # drop(info);
    /// ```
    /// See complete example in `examples/rectangle.rs`
    #[must_use]
    pub const fn usage(mut self, usage: &'static str) -> Self {
        self.usage = Some(usage);
        self
    }

    fn help_parser(&self) -> impl Parser<ExtraParams> {
        ParseExtraParams {
            version: self.version,
        }
    }

    fn render_help(&self, parser_meta: Meta, help_meta: Meta) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut res = String::new();
        if let Some(t) = self.descr {
            write!(res, "{t}\n\n")?;
        }
        if let Some(u) = self.usage {
            write!(res, "{u}\n")?;
        } else if let Some(usage) = parser_meta.as_usage_meta() {
            write!(res, "Usage: {}\n", usage)?;
        }
        if let Some(t) = self.header {
            write!(res, "\n{}\n", t)?;
        }
        let meta = Meta::And(vec![parser_meta, help_meta]);

        let flags = &meta.flags();
        if !flags.is_empty() {
            let max_width = flags.iter().map(Item::full_width).max().unwrap_or(0);
            write!(res, "\nAvailable options:\n")?;
            for i in flags {
                write!(res, "{:#padding$}\n", i, padding = max_width)?;
            }
        }

        let commands = &meta.commands();
        if !commands.is_empty() {
            write!(res, "\nAvailable commands:\n")?;
            let max_width = commands.iter().map(Item::full_width).max().unwrap_or(0);
            for i in commands {
                write!(res, "{:#padding$}\n", i, padding = max_width)?;
            }
        }
        Ok(res)
    }

    /// Attach additional information to the parser
    #[must_use]
    pub fn for_parser<P, T>(self, parser: P) -> impl OptionParser<T>
    where
        P: Parser<T>,
        T: 'static + Clone + std::fmt::Debug,
    {
        let help_meta = self.help_parser().meta();
        let info = self.clone();
        let parser_meta = parser.meta();
        let p = move |args: &mut Args| {
            let mut reg_args = args.clone();
            let err = match parser.run(&mut reg_args) {
                Ok(r) => {
                    if let Err(err) = check_unexpected(&reg_args) {
                        err
                    } else {
                        *args = reg_args;
                        return Ok(r);
                    }
                }

                // Stderr means nested parser couldn't parse something, store it,
                // report it if parsing --help and --version also fails
                Err(Error::Stderr(e)) => Error::Stderr(e),

                // Stdout usually means a happy path such as calling --help or --version on one of
                // the nested commands
                Err(Error::Stdout(e)) => return Err(Error::Stdout(e)),
                Err(err) => err,
            };

            match self.help_parser().run(args) {
                Ok(ExtraParams::Help) => {
                    let msg = self
                        .render_help(parser.meta(), self.help_parser().meta())
                        .expect("Couldn't render help");
                    return Err(Error::Stdout(msg));
                }
                Ok(ExtraParams::Version(v)) => {
                    return Err(Error::Stdout(format!("Version: {}", v)));
                }
                Err(_) => {}
            }
            Err(err)
        };
        OptionParserStruct {
            inner: ParseConstruct {
                inner: p,
                meta: parser_meta,
            },
            inner_type: PhantomData,
            info,
            help_meta,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum ExtraParams {
    Help,
    Version(&'static str),
}

fn check_unexpected(args: &Args) -> Result<(), Error> {
    match args.peek() {
        None => Ok(()),
        Some(item) => Err(Error::Stderr(format!(
            "{} is not expected in this context",
            item
        ))),
    }
}

#[derive(Clone)]
/// Parser with atteched meta information
pub struct OptionParserStruct<T, P> {
    pub(crate) inner: P,
    pub(crate) inner_type: PhantomData<T>,
    pub(crate) help_meta: Meta,
    pub(crate) info: Info,
}

/// Argument parser with additional information attached, created with [`Info::for_parser`].
pub trait OptionParser<T> {
    /// Execute the [`OptionParser`], extract a parsed value or print some diagnostic and exit
    ///
    /// ```no_run
    /// # use bpaf::*;
    /// let verbose = short('v').req_flag(()).many().map(|xs|xs.len());
    /// let info = Info::default().descr("Takes verbosity flag and does nothing else");
    ///
    /// let opt = info.for_parser(verbose).run();
    /// // At this point `opt` contains number of repetitions of `-v` on a command line
    /// # drop(opt)
    /// ```
    #[must_use]
    fn run(self) -> T
    where
        Self: Sized,
    {
        let mut pos_only = false;
        let mut vec = Vec::new();
        for arg in std::env::args_os().skip(1) {
            args::push_vec(&mut vec, arg, &mut pos_only);
        }

        match self.run_inner(Args::from(vec)) {
            Ok(t) => t,
            Err(ParseFailure::Stdout(msg)) => {
                println!("{}", msg);
                std::process::exit(0);
            }
            Err(ParseFailure::Stderr(msg)) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
    }

    /// Execute the [`OptionParser`] and produce a value that can be used in unit tests
    ///
    /// ```
    /// #[test]
    /// fn positional_argument() {
    ///     let p = positional("FILE").help("File to process");
    ///     let parser = Info::default().for_parser(p);
    ///
    ///     let help = parser
    ///         .run_inner(Args::from(&["--help"]))
    ///         .unwrap_err()
    ///         .unwrap_stdout();
    ///     let expected_help = "\
    /// Usage: <FILE>
    ///
    /// Available options:
    ///     -h, --help   Prints help information
    /// ";
    ///     assert_eq!(expected_help, help);
    /// }
    /// ```
    ///
    /// See also [`Args`] and it's `From` impls to produce input and
    /// [`ParseFailure::unwrap_stderr`] / [`ParseFailure::unwrap_stdout`] for processing results.
    ///
    /// # Errors
    ///
    /// If parser can't produce desired outcome `run_inner` will return [`ParseFailure`]
    /// which represents runtime behavior: one branch to print something to stdout and exit with
    /// success and the other branch to print something to stderr and exit with failure.
    ///
    /// Parser is not really capturing anything. If parser detects `--help` or `--version` it will
    /// always produce something that can be consumed with [`ParseFailure::unwrap_stdout`].
    /// Otherwise it will produce [`ParseFailure::unwrap_stderr`]  generated either by the parser
    /// itself in case someone required field is missing or by user's [`Parser::guard`] or
    /// [`Parser::parse`] functions.
    ///
    /// API for those is constructed to only produce a [`String`]. If you try to print something inside
    /// [`Parser::map`] or [`Parser::parse`] - it will not be captured. Depending on a test case
    /// you'll know what to use: `unwrap_stdout` if you want to test generated help or `unwrap_stderr`
    /// if you are testing `parse` / `guard` / missing parameters.
    ///
    /// Exact string reperentations may change between versions including minor releases.
    fn run_inner(&self, mut args: Args) -> Result<T, ParseFailure>
    where
        Self: Sized,
    {
        match self.run_subparser(&mut args) {
            Ok(t) if args.is_empty() => Ok(t),
            Ok(_) => Err(ParseFailure::Stderr(format!("unexpected {:?}", args))),
            Err(Error::Missing(metas)) => Err(ParseFailure::Stderr(format!(
                "Expected {}, pass --help for usage information",
                Meta::Or(metas)
            ))),
            Err(Error::Stdout(stdout)) => Err(ParseFailure::Stdout(stdout)),
            Err(Error::Stderr(stderr)) => Err(ParseFailure::Stderr(stderr)),
        }
    }

    fn run_subparser(&self, args: &mut Args) -> Result<T, Error>;
}

impl<T, P> OptionParser<T> for OptionParserStruct<T, P>
where
    P: Parser<T>,
{
    fn run_subparser(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.run(args)
    }
}

struct ParseExtraParams {
    version: Option<&'static str>,
}

impl Parser<ExtraParams> for ParseExtraParams {
    fn run(&self, args: &mut Args) -> Result<ExtraParams, Error> {
        if let Ok(ok) = ParseExtraParams::help().run(args) {
            return Ok(ok);
        }
        let not_ok = Error::Stderr(String::from("Not a version or help flag"));
        let ver = self.version.ok_or_else(|| not_ok.clone())?;

        if let Ok(ok) = Self::ver(ver).run(args) {
            return Ok(ok);
        }
        Err(not_ok)
    }

    fn meta(&self) -> Meta {
        match self.version {
            Some(ver) => Meta::And(vec![Self::help().meta(), Self::ver(ver).meta()]),
            None => Self::help().meta(),
        }
    }
}

impl ParseExtraParams {
    #[inline(never)]
    fn help() -> impl Parser<ExtraParams> {
        short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(ExtraParams::Help)
    }
    #[inline(never)]
    fn ver(version: &'static str) -> impl Parser<ExtraParams> {
        short('V')
            .long("version")
            .help("Prints version information")
            .req_flag(ExtraParams::Version(version))
    }
}

impl<T, P> OptionParserStruct<T, P> {
    /// Return current help message for outer parser as a string
    pub fn render_help(&self) -> Result<String, std::fmt::Error>
    where
        P: Parser<T>,
    {
        self.info
            .render_help(self.inner.meta(), self.help_meta.clone())
    }
}
