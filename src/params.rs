//! This module exposes type tags to represent different modes of operation of [`SimpleParser`]

use crate::{
    args::{Arg, State},
    error::{Message, MissingItem},
    from_os_str::parse_os_str,
    item::{Item, ShortLong},
    meta_help::Metavar,
    Doc, Error, Meta, OptionParser, Parser, SimpleParser,
};
use std::{ffi::OsString, marker::PhantomData, str::FromStr};

#[cfg(doc)]
use crate::{any, env, long, positional, short};

/// A type of a [`SimpleParser`] that is used to create a parser with a name
///
/// # Combinatoric usage
///
/// Named items ([`argument`](SimpleParser::argument), [`flag`](SimpleParser::<Named>::flag),
/// [`switch`](SimpleParser::switch) or [`req_flag`](SimpleParser::req_flag]) can have up to 2
/// visible names (one short and one long) and as many hidden short and long aliases as needed.
/// It's also possible to consume items from environment variables using [`env`](SimpleParser::env()).
/// You usually start with [`short`] or [`long`] function, then add extra names with
/// [`short`](SimpleParser::short) / [`long`](SimpleParser::long) / [`env`](SimpleParser::env())
/// repeatedly to build a desired set of names then transform it into a parser using `flag`,
/// `switch` or `positional`.
///
/// You can run all the examples by importing `bpaf` with `use bpaf::*;` and running generated
/// options with something like `println!("{:?}", options().run())` as part of a `main` function.
///
/// 1. In most cases you don't keep `Named` `SimpleParser` around long enough to give it a name:
///
///    ```rust
///    # use bpaf::*;
///    fn options() -> OptionParser<usize> {
///        short('s')
///            .long("size")
///            .help("Maximum size to process")
///            .argument("SIZE")
///            .to_options() // <- skip this if you want to use this parser
///                          //    to make other parsers for example
///    }
///    ```
///
/// 2. But in some cases it might be useful to assign parser to a variable and clone it to use
///    in several places. This example starts by making a `Named` `SimpleParser` first and uses
///    it to try the output first with extra parameter and if that fails - once again, without a
///    parameter:
///
///    ```rust
///    # use bpaf::*;
///    # use std::path::PathBuf;
///    #[derive(Debug, Clone)]
///    pub enum Output {
///        ToFile(PathBuf),
///        ToConsole,
///    }
///
///    fn options() -> OptionParser<Output> {
///        let output = short('o').long("output");
///
///        let to_file = output
///            .clone()
///            .help("Save output to file")
///            .argument("PATH")
///            .map(Output::ToFile);
///
///        let to_console = output
///            .help("Print output to console")
///            .req_flag(Output::ToConsole);
///
///        // when combining multiple parsers that can conflict with each other
///        // it's a good idea to put more general first:
///        construct!([to_file, to_console]).to_options()
///    }
///    ```
///
/// 3. Apart from that `Named` `SimpleParser` follows approach similar to a builder. Methods set
///    different fields, methods like [`SimpleParser::argument`] consume the builder and give
///    something that implements a [`Parser`] back.
///
///
/// # Derive usage
///
/// One `Named` `SimpleParser` gets automatically created and automatically consumed for each field
/// inside a structure. You can omit some or all the details
///
/// 1. If no naming information is present at all - `bpaf` would use field name as a long name (or
///    a short name if field name consists of a single character)
///
///    ```
///    # use bpaf::*;
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    struct Options {
///        name: String,
///    }
///    ```
///
/// 2. If `short` or `long` annotation is present without an argument - `bpaf` would use first
///    character or a full name as long and short name respectively. It won't try to add implicit
///    long or short name from the previous item.
///
///    ```
///    # use bpaf::*;
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    struct Options {
///        #[bpaf(short, long)]
///        name: String,
///    }
///    ```
///
/// 3. If `short` or `long` annotation is present with an argument - those are values `bpaf` would
///    use instead of the original field name
///
///    ```
///    # use bpaf::*;
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    struct Options {
///        #[bpaf(short('N'), long("name"))]
///        name: String,
///    }
///    ```
///
/// 4. You can specify many `short` and `long` names, any past the first one of each type will
///    become hidden aliases
///
///    ```
///    # use bpaf::*;
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    struct Options {
///        #[bpaf(short('N'), short('n'), long("name"), long("app-name"))]
///        name: String,
///    }
///    ```
///
///
/// 5. If `env(arg)` annotation is present - in addition to long/short names derived according to
///    rules 1..3 `bpaf` would also parse environment variable `arg` which can be a string literal
///    or an expression.
///
///    ```
///    # use bpaf::*;
///    const DB: &str = "USERS";
///    #[derive(Debug, Clone, Bpaf)]
///    #[bpaf(options)]
///    struct Options {
///        #[bpaf(short, env("APP_NAME"))]
///        name: String,
///
///        #[bpaf(env(DB))]
///        database: String,
///    }
///    ```
///
#[derive(Clone, Debug)]
pub struct Named {
    pub(crate) short: Vec<char>,
    pub(crate) long: Vec<&'static str>,
    pub(crate) env: Vec<&'static str>,
    pub(crate) help: Option<Doc>,
}

impl Named {
    pub(crate) fn flag_item(&self) -> Option<Item> {
        Some(Item::Flag {
            name: ShortLong::try_from(self).ok()?,
            help: self.help.clone(),
            env: self.env.first().copied(),
            shorts: self.short.clone(),
        })
    }
}

impl Named {
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
    /// using [`short`](SimpleParser::short) and [`long`](SimpleParser::long) methods.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/command.md"))]
    ///
    /// To represent multiple possible commands it is convenient to use enums
    #[cfg_attr(not(doctest), doc = include_str!("docs2/command_enum.md"))]
    #[must_use]
    pub fn command(self, name: &'static str) -> SimpleParser<Command<T>>
    where
        T: 'static,
    {
        SimpleParser(Command {
            longs: vec![name],
            shorts: Vec::new(),
            help: self.short_descr().map(Into::into),
            subparser: self,
            adjacent: false,
        })
    }
}

/// Builder structure for subcommands
///
/// Created with [`OptionParser::command`], implements parser for the inner structure, gives access to [`help`](SimpleParser::help).
pub struct Command<T> {
    pub(crate) longs: Vec<&'static str>,
    pub(crate) shorts: Vec<char>,
    // short help!
    pub(crate) help: Option<Doc>,
    pub(crate) subparser: OptionParser<T>,
    pub(crate) adjacent: bool,
}

impl<T> SimpleParser<Command<T>> {
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
        self.0.help = Some(help.into());
        self
    }

    /// Add a custom short alias for a command
    ///
    /// Behavior is similar to [`short`](SimpleParser::short), only first short name is visible.
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.0.shorts.push(short);
        self
    }

    /// Add a custom hidden long alias for a command
    ///
    /// Behavior is similar to [`long`](SimpleParser::long), but since you had to specify the first long
    /// name when making the command - this one becomes a hidden alias.
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.0.longs.push(long);
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
        self.0.adjacent = true;
        self
    }
}

impl<T> Command<T> {
    pub(crate) fn eval(&self, args: &mut State) -> Result<T, Error> {
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

    pub(crate) fn meta(&self) -> Meta {
        Meta::from(self.item())
    }
}

impl<T> Command<T> {
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

pub(crate) fn build_flag_parser<T>(present: T, absent: Option<T>, named: Named) -> Flag<T>
where
    T: Clone + 'static,
{
    Flag {
        present,
        absent,
        named,
    }
}

#[derive(Clone)]
/// Parser for a named switch, created with [`SimpleParser::flag`] or [`SimpleParser::switch`]
pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    pub(crate) named: Named,
}

impl<T: Clone + 'static> Flag<T> {
    pub(crate) fn eval(&self, args: &mut State) -> Result<T, Error> {
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

    pub(crate) fn meta(&self) -> Meta {
        if let Some(item) = self.named.flag_item() {
            item.required(self.absent.is_none())
        } else {
            Meta::Skip
        }
    }
}

pub(crate) fn build_argument<T>(named: Named, metavar: &'static str) -> Argument<T> {
    Argument {
        named,
        metavar,
        ty: PhantomData,
        adjacent: false,
    }
}

/// Parser for a named argument, created with [`argument`](SimpleParser::argument).
#[derive(Clone)]
pub struct Argument<T> {
    ty: PhantomData<T>,
    pub(crate) named: Named,
    metavar: &'static str,
    pub(crate) adjacent: bool,
}

impl<T> Argument<T> {
    pub(crate) fn item(&self) -> Option<Item> {
        Some(Item::Argument {
            name: ShortLong::try_from(&self.named).ok()?,
            metavar: Metavar(self.metavar),
            env: self.named.env.first().copied(),
            help: self.named.help.clone(),
            shorts: self.named.short.clone(),
        })
    }

    pub(crate) fn take_argument(&self, args: &mut State) -> Result<OsString, Error> {
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

impl<T> Parser<T> for Argument<T>
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

/// Parse a positional item, created with [`positional`]
///
/// You can add extra information to positional parsers with [`help`](Self::help)
/// and [`strict`](Self::strict) on this struct.
#[derive(Clone)]
pub struct Positional<T> {
    pub(crate) metavar: &'static str,
    pub(crate) help: Option<Doc>,
    pub(crate) result_type: PhantomData<T>,
    pub(crate) strict: bool,
}

impl<T> Positional<T> {
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

    pub(crate) fn meta(&self) -> Meta {
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
    match args.take_positional_word(metavar) {
        Ok((ix, is_strict, word)) => {
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

impl<T> Positional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    pub(crate) fn eval(&self, args: &mut State) -> Result<T, Error> {
        let os = parse_pos_word(args, self.strict, self.metavar, &self.help)?;
        match parse_os_str::<T>(os) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error(Message::ParseFailed(args.current, err))),
        }
    }
}

/// Consume an arbitrary value that satisfies a condition, created with [`any`], implements
/// [`anywhere`](SimpleParser::anywhere).
pub struct Anything<T> {
    pub(crate) metavar: Doc,
    pub(crate) help: Option<Doc>,
    pub(crate) check: Box<dyn Fn(OsString) -> Option<T>>,
    pub(crate) anywhere: bool,
}

impl<T> Anything<T> {
    pub(crate) fn item(&self) -> Item {
        Item::Any {
            metavar: self.metavar.clone(),
            help: self.help.clone(),
            anywhere: self.anywhere,
        }
    }
}

impl<T> Parser<T> for Anything<T> {
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
