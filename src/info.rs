//! Help message generation and rendering

use crate::{
    args::{Args, State},
    error::Message,
    inner_buffer::{Color, Token},
    meta_help::render_help,
    parsers::{NamedArg, ParseCommand},
    short, Buffer, Error, Meta, ParseFailure, Parser,
};

/// Information about the parser
///
/// No longer public, users are only interacting with it via [`OptionParser`]
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct Info {
    /// version field, see [`version`][Info::version]
    pub version: Option<Buffer>,
    /// Custom description field, see [`descr`][Info::descr]
    pub descr: Option<Buffer>,
    /// Custom header field, see [`header`][Info::header]
    pub header: Option<Buffer>,
    /// Custom footer field, see [`footer`][Info::footer]
    pub footer: Option<Buffer>,
    /// Custom usage field, see [`usage`][Info::usage]
    pub usage: Option<Buffer>,
    pub help_arg: NamedArg,
    pub version_arg: NamedArg,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            version: Default::default(),
            descr: Default::default(),
            header: Default::default(),
            footer: Default::default(),
            usage: Default::default(),
            help_arg: short('h').long("help").help("Prints help information"),
            version_arg: short('V')
                .long("version")
                .help("Prints version information"),
        }
    }
}

/// Ready to run [`Parser`] with additional information attached
///
/// Created with [`to_options`](Parser::to_options)
pub struct OptionParser<T> {
    pub(crate) inner: Box<dyn Parser<T>>,
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
    ///         .run_inner(&["--help"])
    ///         .unwrap_err()
    ///         .unwrap_stdout();
    ///     let expected_help = "\
    /// Usage: FILE
    ///
    /// Available positional items:
    ///     FILE        File to process
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
    pub fn run_inner<'a>(&self, args: impl Into<Args<'a>>) -> Result<T, ParseFailure>
    where
        Self: Sized,
    {
        // prepare available short flags and arguments for disambiguation
        let mut short_flags = Vec::new();
        let mut short_args = Vec::new();
        self.inner
            .meta()
            .collect_shorts(&mut short_flags, &mut short_args);
        let mut err = None;
        let mut state = State::construct(args.into(), &short_flags, &short_args, &mut err);

        // this only handles disambiguation failure in construct
        if let Some(msg) = err {
            return Err(msg.render(&state, &self.inner.meta()));
        }

        self.run_subparser(&mut state)
    }

    /// Run subparser, implementation detail
    pub(crate) fn run_subparser(&self, args: &mut State) -> Result<T, ParseFailure> {
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
        if let Err(Error(Message::ParseFailure(failure))) = res {
            return Err(failure);
        }
        #[cfg(feature = "autocomplete")]
        if let Some(comp) = args.check_complete() {
            return Err(ParseFailure::Stdout(comp));
        }

        let err = match res {
            Ok(ok) => {
                if let Some((ix, _)) = args.items_iter().next() {
                    Message::Unconsumed(ix)
                } else {
                    return Ok(ok);
                }
            }
            Err(Error(err)) => err,
        };

        // handle --help and --version messages
        if let Ok(extra) = self.info.eval(args) {
            let mut detailed = false;
            let buffer = match extra {
                ExtraParams::Help(d) => {
                    detailed = d;
                    render_help(
                        &args.path,
                        &self.info,
                        &self.inner.meta(),
                        &self.info.meta(),
                    )
                }
                ExtraParams::Version(v) => {
                    let mut buffer = Buffer::default();
                    buffer.text("Version: ");
                    buffer.buffer(&v);
                    buffer.token(Token::LineBreak);
                    buffer
                }
            };
            return Err(ParseFailure::Stdout(
                buffer.render(detailed, Color::default()),
            ));
        }
        Err(err.render(args, &self.inner.meta()))
    }

    /// Get first line of description if Available
    ///
    /// Used internally to avoid duplicating description for [`command`].
    #[must_use]
    pub(crate) fn short_descr(&self) -> Option<Buffer> {
        self.info.descr.as_ref().and_then(|b| b.first_line())
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
    pub fn version<B: Into<Buffer>>(mut self, version: B) -> Self {
        self.info.version = Some(version.into());
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
    pub fn descr<B: Into<Buffer>>(mut self, descr: B) -> Self {
        self.info.descr = Some(descr.into());
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
    pub fn header<B: Into<Buffer>>(mut self, header: B) -> Self {
        self.info.header = Some(header.into());
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
    pub fn footer<M: Into<Buffer>>(mut self, footer: M) -> Self {
        self.info.footer = Some(footer.into());
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
    pub fn usage<B>(mut self, usage: B) -> Self
    where
        B: Into<Buffer>,
    {
        self.info.usage = Some(usage.into());
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
        self.inner.meta().positional_invariant_check(true);
    }

    pub fn help_parser(mut self, parser: NamedArg) -> Self {
        self.info.help_arg = parser;
        self
    }

    pub fn version_parser(mut self, parser: NamedArg) -> Self {
        self.info.version_arg = parser;
        self
    }
}

impl Info {
    #[inline(never)]
    fn mk_help_parser(&self) -> impl Parser<()> {
        self.help_arg.clone().req_flag(())
    }
    #[inline(never)]
    fn mk_version_parser(&self) -> impl Parser<()> {
        self.version_arg.clone().req_flag(())
    }
}

impl Parser<ExtraParams> for Info {
    fn eval(&self, args: &mut State) -> Result<ExtraParams, Error> {
        let help = self.mk_help_parser();
        if help.eval(args).is_ok() {
            return Ok(ExtraParams::Help(help.eval(args).is_ok()));
        }

        if let Some(version) = &self.version {
            if self.mk_version_parser().eval(args).is_ok() {
                return Ok(ExtraParams::Version(version.clone()));
            }
        }

        // error message is not actually used anywhere
        Err(Error(Message::ParseFail("not a version or help")))
    }

    fn meta(&self) -> Meta {
        let help = self.mk_help_parser().meta();
        match &self.version {
            Some(_) => Meta::And(vec![help, self.mk_version_parser().meta()]),
            None => help,
        }
    }
}

#[derive(Clone, Debug)]
enum ExtraParams {
    Help(bool),
    Version(Buffer),
}
