//! Help message generation and rendering

use crate::{
    args::{Args, State},
    error::Message,
    meta_help::render_help,
    parsers::NamedArg,
    short, Doc, Error, Meta, ParseFailure, Parser,
};

/// Information about the parser
///
/// No longer public, users are only interacting with it via [`OptionParser`]
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct Info {
    /// version field, see [`version`][Info::version]
    pub version: Option<Doc>,
    /// Custom description field, see [`descr`][Info::descr]
    pub descr: Option<Doc>,
    /// Custom header field, see [`header`][Info::header]
    pub header: Option<Doc>,
    /// Custom footer field, see [`footer`][Info::footer]
    pub footer: Option<Doc>,
    /// Custom usage field, see [`usage`][Info::usage]
    pub usage: Option<Doc>,
    pub help_arg: NamedArg,
    pub version_arg: NamedArg,
    pub help_if_no_args: bool,
    pub max_width: usize,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            version: None,
            descr: None,
            header: None,
            footer: None,
            usage: None,
            help_arg: short('h').long("help").help("Prints help information"),
            version_arg: short('V')
                .long("version")
                .help("Prints version information"),
            help_if_no_args: false,
            max_width: 100,
        }
    }
}

/// Ready to run [`Parser`] with additional information attached
///
/// Created with [`to_options`](Parser::to_options)
///
/// In addition to the inner parser `OptionParser` contains documentation about a program or a
/// subcommand as a whole, version, custom usage, if specified, and handles custom parsers for
/// `--version` and `--help` flags.
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
            Err(err) => {
                err.print_message(self.info.max_width);
                std::process::exit(err.exit_code())
            }
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
    ///         Err(ParseFailure::Stdout(buf, full)) => {
    ///             print!("{}", buf.monochrome(full));
    ///             None
    ///         }
    ///         Err(ParseFailure::Completion(msg)) => {
    ///             print!("{}", msg);
    ///             None
    ///         }
    ///         Err(ParseFailure::Stderr(buf)) => {
    ///             eprintln!("{}", buf.monochrome(true));
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
        short_flags.extend(&self.info.help_arg.short);
        short_flags.extend(&self.info.version_arg.short);
        let args = args.into();
        let mut err = None;
        let mut state = State::construct(args, &short_flags, &short_args, &mut err);

        // this only handles disambiguation failure in construct
        if let Some(msg) = err {
            #[cfg(feature = "autocomplete")]
            let check_disambiguation = state.comp_ref().is_none();

            #[cfg(not(feature = "autocomplete"))]
            let check_disambiguation = false;

            if check_disambiguation {
                return Err(msg.render(&state, &self.inner.meta()));
            }
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

        let no_args = args.is_empty();
        let res = self.inner.eval(args);

        // Don't override inner parser printing usage info
        let parser_failed = match res {
            Ok(_) | Err(Error(Message::ParseFailure(ParseFailure::Stdout(..)))) => false,
            Err(_) => true,
        };

        if parser_failed && self.info.help_if_no_args && no_args {
            let buffer = render_help(
                &args.path,
                &self.info,
                &self.inner.meta(),
                &self.info.meta(),
                true,
            );
            return Err(ParseFailure::Stdout(buffer, false));
        };

        if let Err(Error(Message::ParseFailure(failure))) = res {
            return Err(failure);
        }
        #[cfg(feature = "autocomplete")]
        if let Some(comp) = args.check_complete() {
            return Err(ParseFailure::Completion(comp));
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
                        true,
                    )
                }
                ExtraParams::Version(v) => {
                    use crate::buffer::{Block, Token};
                    let mut buffer = Doc::default();
                    buffer.token(Token::BlockStart(Block::Block));
                    buffer.text("Version: ");
                    buffer.doc(&v);
                    buffer.token(Token::BlockEnd(Block::Block));
                    buffer
                }
            };
            return Err(ParseFailure::Stdout(buffer, detailed));
        }
        Err(err.render(args, &self.inner.meta()))
    }

    /// Get first line of description if Available
    ///
    /// Used internally to avoid duplicating description for [`command`].
    #[must_use]
    pub(crate) fn short_descr(&self) -> Option<Doc> {
        self.info.descr.as_ref().and_then(Doc::first_line)
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
    pub fn version<B: Into<Doc>>(mut self, version: B) -> Self {
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
    pub fn descr<B: Into<Doc>>(mut self, descr: B) -> Self {
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
    pub fn header<B: Into<Doc>>(mut self, header: B) -> Self {
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
    pub fn footer<M: Into<Doc>>(mut self, footer: M) -> Self {
        self.info.footer = Some(footer.into());
        self
    }

    /// Set custom usage field
    ///
    /// Custom usage field to use instead of one derived by `bpaf`.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/usage.md"))]
    #[must_use]
    pub fn usage<B>(mut self, usage: B) -> Self
    where
        B: Into<Doc>,
    {
        self.info.usage = Some(usage.into());
        self
    }

    /// Generate new usage line using automatically derived usage
    ///
    /// You can customize the surroundings of the usage line while still
    /// having part that frequently changes generated by bpaf
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/with_usage.md"))]
    ///
    /// At the moment this method is not directly supported by derive API,
    /// but since it gives you an object of [`OptionParser<T>`](OptionParser)
    /// type you can alter it using Combinatoric API:
    /// ```text
    /// #[derive(Debug, Clone, Bpaf)] {
    /// pub struct Options {
    ///     ...
    /// }
    ///
    /// fn my_decor(usage: Doc) -> Doc {
    ///     ...
    /// }
    ///
    /// fn main() {
    ///     let options = options().with_usage(my_decor).run();
    ///     ...
    /// }
    /// ```
    #[must_use]
    pub fn with_usage<F>(mut self, f: F) -> Self
    where
        F: Fn(Doc) -> Doc,
    {
        let mut buf = Doc::default();
        buf.write_meta(&self.inner.meta(), true);
        self.info.usage = Some(f(buf));
        self
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

    /// Customize parser for `--help`
    ///
    /// By default `bpaf` displays help when program is called with either `--help` or `-h`, you
    /// can customize those names and description in the help message
    ///
    /// Note, `--help` is something user expects to work
    #[cfg_attr(not(doctest), doc = include_str!("docs2/custom_help_version.md"))]
    #[must_use]
    pub fn help_parser(mut self, parser: NamedArg) -> Self {
        self.info.help_arg = parser;
        self
    }

    /// Customize parser for `--version`
    ///
    /// By default `bpaf` displays version information when program is called with either `--version`
    /// or `-V` (and version is available), you can customize those names and description in the help message
    ///
    /// Note, `--version` is something user expects to work
    #[cfg_attr(not(doctest), doc = include_str!("docs2/custom_help_version.md"))]
    #[must_use]
    pub fn version_parser(mut self, parser: NamedArg) -> Self {
        self.info.version_arg = parser;
        self
    }

    /// Print help if app was called with no parameters
    ///
    /// By default `bpaf` tries to parse command line options and displays the best possible
    /// error it can come up with. If application requires a subcommand or some argument
    /// and user specified none - it might be a better experience for user to print
    /// the help message.
    ///
    /// ```rust
    /// # use bpaf::*;
    /// # fn options() -> OptionParser<bool> { short('a').switch().to_options() }
    /// // create option parser in a usual way, derive or combinatoric API
    /// let opts = options().fallback_to_usage().run();
    /// ```
    ///
    /// For derive macro you can specify `fallback_to_usage` in top level annotations
    /// for options and for individual commands if fallback to useage is the desired behavior:
    ///
    ///
    /// ```ignore
    /// #[derive(Debug, Clone, Bpaf)]
    /// enum Commands {
    ///     #[bpaf(command, fallback_to_usage)]
    ///     Action {
    ///         ...
    ///     }
    /// }
    /// ```
    ///
    /// Or
    ///
    /// ```ignore
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, fallback_to_usage)]
    /// struct Options {
    ///     ...
    /// }
    ///
    /// fn main() {
    ///     let options = options().run(); // falls back to usage
    /// }
    /// ```
    #[must_use]
    pub fn fallback_to_usage(mut self) -> Self {
        self.info.help_if_no_args = true;
        self
    }

    /// Set the width of the help message printed to the terminal upon failure
    ///
    /// By default, the help message is printed with a width of 100 characters.
    /// This method allows to change where the help message is wrapped.
    ///
    /// Setting the max width too low may negatively affect the readability of the help message.
    /// Also, the alignment padding of broken lines is always applied.
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Self {
        self.info.max_width = width;
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
pub(crate) enum ExtraParams {
    Help(bool),
    Version(Doc),
}
