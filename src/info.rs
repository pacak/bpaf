//! Help message generation and rendering

#![allow(clippy::write_with_newline)]
use std::rc::Rc;

use crate::{args::Args, params::short, DynParse, Item, Meta, Parser};

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

/// Parser with atteched meta information
#[derive(Clone)]
pub struct OptionParser<T> {
    pub(crate) parse: Rc<DynParse<T>>,
    pub(crate) parser_meta: Meta,
    pub(crate) help_meta: Meta,
    pub(crate) info: Info,
}

impl<T> OptionParser<T> {
    /// Return current help message for outer parser as a string
    pub fn render_help(&self) -> Result<String, std::fmt::Error> {
        self.info
            .clone()
            .render_help(self.parser_meta.clone(), self.help_meta.clone())
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

    fn help_parser(&self) -> Parser<ExtraParams> {
        let help = short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(ExtraParams::Help);

        match self.version {
            Some(v) => help.or_else(
                short('V')
                    .long("version")
                    .help("Prints version information")
                    .req_flag(ExtraParams::Version(v)),
            ),
            None => help,
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
    pub fn for_parser<T>(self, parser: Parser<T>) -> OptionParser<T>
    where
        T: 'static + Clone + std::fmt::Debug,
    {
        let parser_meta = parser.meta.clone();
        let help_meta = self.help_parser().meta;
        let Parser {
            parse: p_parse,
            meta: p_meta,
        } = parser;
        let info = self.clone();
        let p = move |args: Args| {
            let err = match p_parse(args.clone()).and_then(check_unexpected) {
                Ok(r) => return Ok(r),

                // Stderr means
                Err(Error::Stderr(e)) => Error::Stderr(e),

                // Stdout usually means a happy path such as calling --help or --version on one of
                // the nested commands
                Err(Error::Stdout(e)) => return Err(Error::Stdout(e)),
                Err(err) => err,
            };

            match (self.help_parser().parse)(args) {
                Ok((ExtraParams::Help, _)) => {
                    let msg = self
                        .clone()
                        .render_help(p_meta.clone(), self.help_parser().meta)
                        .expect("Couldn't render help");
                    return Err(Error::Stdout(msg));
                }
                Ok((ExtraParams::Version(v), _)) => {
                    return Err(Error::Stdout(format!("Version: {}", v)));
                }
                Err(_) => {}
            }
            Err(err)
        };
        OptionParser {
            parse: Rc::new(p),
            info,
            parser_meta,
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

fn check_unexpected<T>((t, args): (T, Args)) -> Result<(T, Args), Error> {
    match args.peek() {
        None => Ok((t, args)),
        Some(item) => Err(Error::Stderr(format!(
            "{} is not expected in this context",
            item
        ))),
    }
}
