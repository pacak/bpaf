//! Help message generation and rendering

#![allow(clippy::write_with_newline)]
use std::marker::PhantomData;

use crate::{
    args::{Args, Conflict},
    item::Item,
    meta_help::render_help,
    params::short,
    parsers::ParseCommand,
    Meta, ParseFailure, Parser,
};

/// Unsuccessful command line parsing outcome, internal representation
#[derive(Debug)]
pub enum Error {
    /// Parsing failed, it is still possible to improve the error message
    Message(String),
    /// Parsing failed and this is the final output
    ParseFailure(ParseFailure),
    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<Item>),
}

impl Error {
    #[must_use]
    pub(crate) fn combine_with(self, other: Self) -> Self {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // help output takes priority
            (a @ Error::ParseFailure(_), _) => a,
            (_, b @ Error::ParseFailure(_)) => b,

            // parsing failure takes priority
            (a @ Error::Message(_), _) => a,
            (_, b @ Error::Message(_)) => b,

            // combine missing elements
            (Error::Missing(mut a), Error::Missing(mut b)) => {
                a.append(&mut b);
                Error::Missing(a)
            }
        }
    }
}

/// Information about the parser
///
/// No longer public, users are only interacting with it via [`OptionParser`]
#[derive(Debug, Clone, Default)]
#[doc(hidden)]
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
    fn help_parser(&self) -> impl Parser<ExtraParams> {
        ParseExtraParams {
            version: self.version,
        }
    }
}

#[derive(Clone, Debug)]
enum ExtraParams {
    Help,
    Version(&'static str),
}

fn check_unexpected(args: &Args) -> Result<(), Error> {
    match args.items_iter().next() {
        None => Ok(()),
        Some((ix, item)) => {
            let mut msg = format!("{} is not expected in this context", item);
            if let Some(Conflict::Conflicts(acc, rej)) = args.conflicts.get(&ix) {
                use std::fmt::Write;
                write!(msg, ": {} cannot be used at the same time as {}", rej, acc).unwrap();
            }
            Err(Error::Message(msg))
        }
    }
}

/// Ready to run [`Parser`] with additional information attached
///
/// Created with [`to_options`](Parser::to_options)
pub struct OptionParser<T> {
    pub(crate) inner: Box<dyn Parser<T>>,
    pub(crate) inner_type: PhantomData<T>,
    pub(crate) info: Info,
}

impl<T> OptionParser<T> {
    /// Execute the [`OptionParser`], extract a parsed value or print some diagnostic and exit
    ///
    /// # Usage
    /// ```no_run
    /// # use bpaf::*;
    /// /// Parses number of repetitions of `-v` on a command line
    /// fn verbosity() -> OptionParser<usize> {
    ///     let parser = short('v')
    ///         .req_flag(())
    ///         .many()
    ///         .map(|xs|xs.len());
    ///
    ///     parser
    ///         .to_options()
    ///         .descr("Takes verbosity flag and does nothing else")
    /// }
    ///
    /// fn main() {
    ///     let verbosity: usize = verbosity().run();
    /// }
    /// ```
    #[must_use]
    pub fn run(self) -> T
    where
        Self: Sized,
    {
        match self.run_inner(Args::current_args()) {
            Ok(t) => t,
            Err(err) => std::process::exit(err.exit_code()),
        }
    }

    /// Execute the [`OptionParser`], extract a parsed value or return a [`ParseFailure`]
    ///
    /// In most cases using [`run`](OptionParser::run) is sufficient, you can use `try_run` if you
    /// want to control the exit code or you need to perform a custom cleanup.
    ///
    /// # Usage
    /// ```no_run
    /// # use bpaf::*;
    /// /// Parses number of repetitions of `-v` on a command line
    /// fn verbosity() -> OptionParser<usize> {
    ///     let parser = short('v')
    ///         .req_flag(())
    ///         .many()
    ///         .map(|xs|xs.len());
    ///
    ///     parser
    ///         .to_options()
    ///         .descr("Takes verbosity flag and does nothing else")
    /// }
    ///
    /// fn main() {
    ///     let verbosity: Option<usize> = match verbosity().try_run() {
    ///         Ok(v) => Some(v),
    ///         Err(ParseFailure::Stdout(msg)) => {
    ///             print!("{}", msg); // completions are sad otherwise
    ///             None
    ///         }
    ///         Err(ParseFailure::Stderr(msg)) => {
    ///             eprintln!("{}", msg);
    ///             None
    ///         }
    ///     };
    ///
    ///     // Run cleanup tasks
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ParseFailure`] represents parsing errors, autocomplete results and generated `--help`
    /// output.
    #[deprecated = "You should switch to equivalent parser.run_inner(Args::current_args())"]
    pub fn try_run(self) -> Result<T, ParseFailure>
    where
        Self: Sized,
    {
        self.run_inner(Args::current_args())
    }

    /// Execute the [`OptionParser`] and produce a values for unit tests or manual processing
    ///
    /// ```rust
    /// # use bpaf::*;
    /// # /*
    /// #[test]
    /// fn positional_argument() {
    /// # */
    ///     let parser =
    ///         positional::<String>("FILE")
    ///             .help("File to process")
    ///             .to_options();
    ///
    ///     let help = parser
    ///         .run_inner(Args::from(&["--help"]))
    ///         .unwrap_err()
    ///         .unwrap_stdout();
    ///     let expected_help = "\
    /// Usage: <FILE>
    ///
    /// Available positional items:
    ///     <FILE>  File to process
    ///
    /// Available options:
    ///     -h, --help  Prints help information
    /// ";
    ///     assert_eq!(expected_help, help);
    /// # /*
    /// }
    /// # */
    /// ```
    ///
    /// See also [`Args`] and it's `From` impls to produce input and
    /// [`ParseFailure::unwrap_stderr`] / [`ParseFailure::unwrap_stdout`] for processing results.
    ///
    /// # Errors
    ///
    /// If parser can't produce desired result `run_inner` returns [`ParseFailure`]
    /// which represents runtime behavior: one branch to print something to stdout and exit with
    /// success and the other branch to print something to stderr and exit with failure.
    ///
    /// `bpaf` generates contents of this `ParseFailure` using expected textual output from
    /// [`parse`](Parser::parse), stdout/stderr isn't actually captured.
    ///
    /// Exact string reperentations may change between versions including minor releases.
    pub fn run_inner(&self, mut args: Args) -> Result<T, ParseFailure>
    where
        Self: Sized,
    {
        let mut avail_flags = Vec::new();
        let mut avail_args = Vec::new();
        self.inner
            .meta()
            .collect_shorts(&mut avail_flags, &mut avail_args);
        args.disambiguate(&avail_flags, &avail_args)?;
        match self.run_subparser(&mut args) {
            Ok(t) if args.is_empty() => Ok(t),
            Ok(_) => Err(ParseFailure::Stderr(format!("unexpected {:?}", args))),
            Err(err) => Err(err),
        }
    }

    /// Run subparser, implementation detail
    pub(crate) fn run_subparser(&self, args: &mut Args) -> Result<T, ParseFailure> {
        // process should work like this:
        // - inner parser is evaluated, it returns Error
        // - if error is finalized (ParseFailure) - it is simply propagated outwards,
        //   otherwise we are making a few attempts at improving it after dealing with
        //   autocomplete/help
        //
        // - generate autocomplete, if enabled
        // - produce --help, --version
        // - Try to improve error message and finalize it otherwise
        //
        // outer parser gets value in ParseFailure format

        let res = self.inner.eval(args);
        if let Err(Error::ParseFailure(failure)) = res {
            return Err(failure);
        }
        #[cfg(feature = "autocomplete")]
        if let Some(comp) = args.check_complete() {
            return Err(ParseFailure::Stdout(comp));
        }

        let err = match res {
            Ok(r) => {
                if let Err(err) = check_unexpected(args) {
                    err
                } else {
                    return Ok(r);
                }
            }
            Err(err) => err,
        };
        Err(improve_error(args, &self.info, &self.inner.meta(), err))
    }

    /// Get first line of description if Available
    ///
    /// Used internally to avoid duplicating description for [`command`].
    #[must_use]
    pub(crate) fn short_descr(&self) -> Option<&'static str> {
        self.info.descr.and_then(|descr| descr.lines().next())
    }

    /// Set the version field.
    ///
    /// By default `bpaf` won't include any version info and won't accept `--version` switch.
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// use bpaf::*;
    /// fn options() -> OptionParser<bool>  {
    ///    short('s')
    ///        .switch()
    ///        .to_options()
    ///        .version(env!("CARGO_PKG_VERSION"))
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// `version` annotation is available after `options` and `command` annotations, takes
    /// an optional argument - version value to use, otherwise `bpaf_derive` would use value from cargo.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version)]
    /// struct Options {
    ///     #[bpaf(short)]
    ///     switch: bool
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --version
    /// Version: 0.5.0
    /// ```
    #[must_use]
    pub fn version(mut self, version: &'static str) -> Self {
        self.info.version = Some(version);
        self
    }
    /// Set the description field
    ///
    /// Description field should be 1-2 lines long briefly explaining program purpose. If
    /// description field is present `bpaf` would print it right before the usage line.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn options() -> OptionParser<bool>  {
    ///    short('s')
    ///        .switch()
    ///        .to_options()
    ///        .descr("This is a description")
    ///        .header("This is a header")
    ///        .footer("This is a footer")
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// `bpaf_derive` uses doc comments on the `struct` / `enum` to derive description, it skips single empty
    /// lines and uses double empty lines break it into blocks. `bpaf_derive` would use first block as the
    /// description, second block - header, third block - footer.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version)]
    /// /// This is a description
    /// ///
    /// ///
    /// /// This is a header
    /// ///
    /// ///
    /// /// This is a footer
    /// ///
    /// ///
    /// /// This is just a comment
    /// struct Options {
    ///     #[bpaf(short)]
    ///     switch: bool
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```console
    /// This is a description
    ///
    /// Usage: [-s]
    ///
    /// This is a header
    ///
    /// Available options:
    ///     -s
    ///     -h, --help     Prints help information
    ///     -V, --version  Prints version information
    ///
    /// This is a footer
    /// ```
    #[must_use]
    pub fn descr(mut self, descr: &'static str) -> Self {
        self.info.descr = Some(descr);
        self
    }
    /// Set the header field
    ///
    /// `bpaf` displays the header between the usage line and a list of the available options in `--help` output
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn options() -> OptionParser<bool>  {
    ///    short('s')
    ///        .switch()
    ///        .to_options()
    ///        .descr("This is a description")
    ///        .header("This is a header")
    ///        .footer("This is a footer")
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// `bpaf_derive` uses doc comments on the `struct` / `enum` to derive description, it skips single empty
    /// lines and uses double empty lines break it into blocks. `bpaf_derive` would use first block as the
    /// description, second block - header, third block - footer.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version)]
    /// /// This is a description
    /// ///
    /// ///
    /// /// This is a header
    /// ///
    /// ///
    /// /// This is a footer
    /// ///
    /// ///
    /// /// This is just a comment
    /// struct Options {
    ///     #[bpaf(short)]
    ///     switch: bool
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```console
    /// This is a description
    ///
    /// Usage: [-s]
    ///
    /// This is a header
    ///
    /// Available options:
    ///     -s
    ///     -h, --help     Prints help information
    ///     -V, --version  Prints version information
    ///
    /// This is a footer
    /// ```
    #[must_use]
    pub fn header(mut self, header: &'static str) -> Self {
        self.info.header = Some(header);
        self
    }
    /// Set the footer field
    ///
    /// `bpaf` displays the footer after list of the available options in `--help` output
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn options() -> OptionParser<bool>  {
    ///    short('s')
    ///        .switch()
    ///        .to_options()
    ///        .descr("This is a description")
    ///        .header("This is a header")
    ///        .footer("This is a footer")
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// `bpaf_derive` uses doc comments on the `struct` / `enum` to derive description, it skips single empty
    /// lines and uses double empty lines break it into blocks. `bpaf_derive` would use first block as the
    /// description, second block - header, third block - footer.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version)]
    /// /// This is a description
    /// ///
    /// ///
    /// /// This is a header
    /// ///
    /// ///
    /// /// This is a footer
    /// ///
    /// ///
    /// /// This is just a comment
    /// struct Options {
    ///     #[bpaf(short)]
    ///     switch: bool
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// ```console
    /// This is a description
    ///
    /// Usage: [-s]
    ///
    /// This is a header
    ///
    /// Available options:
    ///     -s
    ///     -h, --help     Prints help information
    ///     -V, --version  Prints version information
    ///
    /// This is a footer
    /// ```
    #[must_use]
    pub fn footer(mut self, footer: &'static str) -> Self {
        self.info.footer = Some(footer);
        self
    }
    /// Set custom usage field
    ///
    /// Custom usage field to use instead of one derived by `bpaf`. Custom message should contain
    /// `"Usage: "` prefix if you want to display one.
    ///
    /// Before using it `bpaf` would replace `"{usage}"` tokens inside a custom usage string with
    /// automatically generated usage.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn options() -> OptionParser<bool>  {
    ///    short('s')
    ///        .switch()
    ///        .to_options()
    ///        .usage("Usage: my_program: {usage}")
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, usage("Usage: my_program: {usage}"))]
    /// struct Options {
    ///     #[bpaf(short)]
    ///     switch: bool
    /// }
    /// ```
    #[must_use]
    pub fn usage(mut self, usage: &'static str) -> Self {
        self.info.usage = Some(usage);
        self
    }

    /// Turn `OptionParser` into subcommand parser
    ///
    /// This is identical to [`command`](crate::params::command)
    #[must_use]
    pub fn command(self, name: &'static str) -> ParseCommand<T>
    where
        T: 'static,
    {
        crate::params::command(name, self)
    }

    /// Check the invariants `bpaf` relies on for normal operations
    ///
    /// Takes a parameter whether to check for cosmetic invariants or not
    /// (max help width exceeding 120 symbols, etc), currently not in use
    ///
    /// Best used as part of your test suite:
    /// ```no_run
    /// # use bpaf::*;
    /// #[test]
    /// fn check_options() {
    /// # let options = || short('p').switch().to_options();
    ///     options().check_invariants(false)
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// `check_invariants` indicates problems with panic
    pub fn check_invariants(&self, _cosmetic: bool) {
        perform_invariant_check(&self.inner.meta(), true);
    }
}

fn improve_error(args: &mut Args, info: &Info, inner: &Meta, err: Error) -> ParseFailure {
    match info.help_parser().eval(args) {
        Ok(ExtraParams::Help) => {
            let msg = render_help(info, inner, &info.help_parser().meta());
            return ParseFailure::Stdout(msg);
        }
        Ok(ExtraParams::Version(v)) => {
            return ParseFailure::Stdout(format!("Version: {}\n", v));
        }
        Err(_) => {}
    }

    if crate::meta_youmean::should_suggest(&err) {
        if let Some(msg) = crate::meta_youmean::suggest(args, inner) {
            return ParseFailure::Stderr(msg);
        }
    }
    ParseFailure::from(err)
}

impl From<Error> for ParseFailure {
    fn from(value: Error) -> Self {
        match value {
            Error::Message(msg) => ParseFailure::Stderr(msg),
            Error::ParseFailure(pf) => pf,
            Error::Missing(items) => ParseFailure::Stderr(format!(
                "Expected {}, pass --help for usage information",
                Meta::Or(items.into_iter().map(Meta::Item).collect::<Vec<_>>())
            )),
        }
    }
}

/// do a nested invariant check

/// the check itself is performed as part of `to_usage_meta` transformation `fresh` parameter
/// is used to perform it only once for every command parser encountered
fn perform_invariant_check(meta: &Meta, fresh: bool) {
    if fresh {
        println!("Checking\n{:#?}", meta);
        meta.to_usage_meta();
    }
    match meta {
        Meta::And(xs) | Meta::Or(xs) => {
            for i in xs.iter() {
                perform_invariant_check(i, false);
            }
        }
        Meta::HideUsage(x) | Meta::Optional(x) | Meta::Many(x) | Meta::Decorated(x, _) => {
            perform_invariant_check(x, false);
        }
        Meta::Item(i) => match i {
            Item::Command { meta, .. } => perform_invariant_check(meta, true),
            Item::Positional { .. }
            | Item::Flag { .. }
            | Item::Argument { .. }
            | Item::MultiArg { .. } => {}
        },
        Meta::Skip => {}
    }
}

struct ParseExtraParams {
    version: Option<&'static str>,
}

impl Parser<ExtraParams> for ParseExtraParams {
    fn eval(&self, args: &mut Args) -> Result<ExtraParams, Error> {
        if let Ok(ok) = ParseExtraParams::help().eval(args) {
            return Ok(ok);
        }

        match self.version {
            Some(ver) => Self::ver(ver).eval(args),
            None => Err(Error::Message(String::from("Not a version or help flag"))),
        }
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
