//! Help message generation and rendering

#![allow(clippy::write_with_newline)]
use std::rc::Rc;

use crate::{args::Args, params::short, DynParse, Parser};

/// Internal parse error, used
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
    #[cfg(test)]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Error::Stderr(err) => err,
            Error::Stdout(_) | Error::Missing(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    #[cfg(test)]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Error::Stdout(err) => err,
            Error::Stderr(_) | Error::Missing(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }

    #[doc(hidden)]
    pub fn combine_with(self, other: Self) -> Self {
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

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemKind {
    Flag,
    Command,
    Decor,
    Positional,
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Item {
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub metavar: Option<&'static str>,
    pub help: Option<String>,
    pub kind: ItemKind,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ItemKind::Flag => match (self.short, self.long, self.metavar) {
                (None, None, _) => {
                    unreachable!("Item should have either short or long variant by construction")
                }
                (None, Some(l), None) => write!(f, "--{}", l),
                (Some(s), _, None) => write!(f, "-{}", s),
                (None, Some(l), Some(v)) => write!(f, "--{} {}", l, v),
                (Some(s), _, Some(v)) => write!(f, "-{} {}", s, v),
            },

            ItemKind::Command => write!(f, "COMMAND"),
            ItemKind::Positional => match self.metavar {
                Some(m) => write!(f, "<{}>", m),
                None => write!(f, "<FILE>"),
            },
            ItemKind::Decor => Ok(()),
        }
    }
}

impl Item {
    #[doc(hidden)]
    pub fn required(self, required: bool) -> Meta {
        if required {
            Meta::Required(Box::new(Meta::Item(self)))
        } else {
            Meta::Optional(Box::new(Meta::Item(self)))
        }
    }

    #[doc(hidden)]
    pub fn name_len(&self) -> usize {
        let mut res = 0;
        res += match self.long {
            Some(s) => s.len() + 3,
            None => 0,
        };
        res += match self.metavar {
            Some(s) => s.len() + 2,
            None => 0,
        };
        res
    }

    #[doc(hidden)]
    pub fn decoration<M>(help: Option<M>) -> Self
    where
        M: Into<String>,
    {
        Self {
            short: None,
            long: None,
            metavar: None,
            help: help.map(|h| h.into()),
            kind: ItemKind::Decor,
        }
    }

    #[doc(hidden)]
    pub fn is_command(&self) -> bool {
        match self.kind {
            ItemKind::Command => true,
            ItemKind::Flag | ItemKind::Decor | ItemKind::Positional => false,
        }
    }

    #[doc(hidden)]
    pub fn is_flag(&self) -> bool {
        match self.kind {
            ItemKind::Flag | ItemKind::Decor => true,
            ItemKind::Command | ItemKind::Positional => false,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Meta {
    /// always fails
    Empty,

    ///
    And(Vec<Meta>),
    Or(Vec<Meta>),
    Required(Box<Meta>),
    Optional(Box<Meta>),
    Item(Item),
    Many(Box<Meta>),
    Decorated(Box<Meta>, String),
    Id,
}

#[doc(hidden)]
impl Meta {
    pub fn is_required(&self) -> bool {
        match self {
            Meta::Empty => false,
            Meta::And(xs) => xs.iter().any(|x| x.is_required()),
            Meta::Or(xs) => xs.iter().all(|x| x.is_required()),
            Meta::Required(_) => true,
            Meta::Optional(_) => false,
            Meta::Item(i) => match i.kind {
                ItemKind::Flag => unreachable!(),
                ItemKind::Command => true,
                ItemKind::Decor => false,
                ItemKind::Positional => unreachable!(),
            },
            Meta::Many(_) => false,
            Meta::Id => false,
            Meta::Decorated(x, _) => x.is_required(),
        }
    }
    pub fn or(self, other: Meta) -> Self {
        match (self, other) {
            (Meta::Id, b) => b,
            (Meta::Empty, b) => b,
            (a, Meta::Empty) => a,
            (a, Meta::Id) => a,
            (Meta::Or(mut a), Meta::Or(mut b)) => {
                a.append(&mut b);
                Meta::Or(a)
            }
            (Meta::Or(mut a), b) => {
                a.push(b);
                Meta::Or(a)
            }
            (a, Meta::Or(mut b)) => {
                b.push(a);
                Meta::Or(b)
            }
            (a, b) => Meta::Or(vec![a, b]),
        }
    }
    pub fn and(self, other: Meta) -> Self {
        match (self, other) {
            (Meta::Id, a) => a,
            (b, Meta::Id) => b,
            (Meta::And(mut xs), Meta::And(mut ys)) => {
                xs.append(&mut ys);
                Meta::And(xs)
            }
            (Meta::And(mut xs), b) => {
                xs.push(b);
                Meta::And(xs)
            }
            (a, Meta::And(mut ys)) => {
                ys.push(a);
                Meta::And(ys)
            }
            (a, b) => Meta::And(vec![a, b]),
        }
    }
    pub fn optional(self) -> Self {
        match self {
            Meta::Required(m) => Meta::Optional(m),
            m @ Meta::Optional(_) => m,
            m => Meta::Optional(Box::new(m)),
        }
    }
    pub fn required(self) -> Self {
        Meta::Required(Box::new(self))
    }
    pub fn many(self) -> Self {
        Meta::Many(Box::new(self))
    }

    pub fn commands(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res, |i| i.is_command());
        res
    }

    pub fn flags(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res, |i| i.is_flag());
        res
    }

    pub fn decorate<M>(self, msg: M) -> Self
    where
        M: Into<String>,
    {
        Meta::Decorated(Box::new(self), msg.into())
    }

    fn collect_items<F>(&self, res: &mut Vec<Item>, pred: F)
    where
        F: Fn(&Item) -> bool + Copy,
    {
        match self {
            Meta::Empty => {}
            Meta::And(xs) => {
                for x in xs {
                    x.collect_items(res, pred);
                }
            }
            Meta::Or(xs) => {
                for x in xs {
                    x.collect_items(res, pred);
                }
            }
            Meta::Required(a) => a.collect_items(res, pred),
            Meta::Many(a) => a.collect_items(res, pred),
            Meta::Optional(a) => a.collect_items(res, pred),
            Meta::Item(i) => {
                if pred(i) {
                    res.push(i.clone())
                }
            }
            Meta::Id => {}
            Meta::Decorated(x, msg) => {
                res.push(Item::decoration(Some(msg)));
                let prev_len = res.len();
                x.collect_items(res, pred);
                if res.len() == prev_len {
                    res.pop();
                } else {
                    res.push(Item::decoration(None::<String>));
                }
            }
        }
    }

    fn is_simple(&self) -> bool {
        match self {
            Meta::Empty => true,
            Meta::And(_) => false,
            Meta::Or(_) => false,
            Meta::Required(m) => m.is_simple(),
            Meta::Optional(m) => m.is_simple(),
            Meta::Item(_) => true,
            Meta::Many(m) => m.is_simple(),
            Meta::Decorated(m, _) => m.is_simple(),
            Meta::Id => true,
        }
    }
}

fn dedup(prev: &mut Option<Item>, cur: &Meta) -> bool {
    let item = if let Meta::Item(item) = cur {
        item
    } else {
        return true;
    };
    match prev {
        Some(p) => {
            if p.kind == ItemKind::Command && item.kind == ItemKind::Command {
                false
            } else {
                *prev = Some(item.clone());
                true
            }
        }
        None => {
            *prev = Some(item.clone());
            true
        }
    }
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Meta::Empty => Ok(()),
            Meta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    write!(f, "{}", x)?;
                    if ix + 1 < xs.len() {
                        write!(f, " ")?;
                    }
                }
                Ok(())
            }
            Meta::Or(xs) => {
                let mut prev = None;
                let xs = xs
                    .iter()
                    .filter(|i| dedup(&mut prev, i))
                    .collect::<Vec<_>>();
                let required = self.is_required();
                if !required {
                    write!(f, "[")?;
                } else if xs.len() > 1 {
                    write!(f, "(")?;
                }
                for (ix, x) in xs.iter().enumerate() {
                    write!(f, "{}", x)?;
                    if ix + 1 < xs.len() {
                        write!(f, " | ")?;
                    }
                }
                if !required {
                    write!(f, "]")?
                } else if xs.len() > 1 {
                    write!(f, ")")?
                }
                Ok(())
            }
            Meta::Required(m) if m.is_simple() => write!(f, "{}", m),
            Meta::Required(m) => write!(f, "({})", m),
            Meta::Optional(m) => write!(f, "[{}]", m),
            Meta::Many(m) => write!(f, "{}...", m),
            Meta::Item(i) => write!(f, "{}", i),
            Meta::Id => Ok(()),
            Meta::Decorated(x, _) => write!(f, "{}", x),
        }
    }
}

impl From<Item> for Meta {
    fn from(item: Item) -> Self {
        Meta::Item(item)
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
///                .version("3.1415")
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
    /// let info = Info::default().version("3.1415");
    /// # drop(info);
    /// ```
    pub fn version(mut self, version: &'static str) -> Self {
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
    pub fn descr(mut self, descr: &'static str) -> Self {
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
    pub fn header(mut self, header: &'static str) -> Self {
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
    pub fn footer(mut self, footer: &'static str) -> Self {
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
    pub fn usage(mut self, usage: &'static str) -> Self {
        self.usage = Some(usage);
        self
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum ExtraParams {
    Help,
    Version,
}

impl Info {
    fn help_parser(&self) -> Parser<ExtraParams> {
        let help = short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(ExtraParams::Help);

        let ver = short('v')
            .long("version")
            .help("Prints version information")
            .req_flag(ExtraParams::Version);

        if self.version.is_some() {
            help.or_else(ver)
        } else {
            help
        }
    }

    fn render_help(self, parser_meta: Meta, help_meta: Meta) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut res = String::new();
        if let Some(t) = self.descr {
            write!(res, "{}\n\n", t)?;
        }
        match self.usage {
            Some(u) => write!(res, "{}\n\n", u)?,
            None => write!(res, "Usage: {}\n", parser_meta)?,
        }
        if let Some(t) = self.header {
            write!(res, "\n{}\n", t)?;
        }
        let meta = Meta::and(parser_meta, help_meta);
        let flags = &meta.flags();

        let max_name_width = flags.iter().map(|i| i.name_len()).max().unwrap_or(0);
        if !flags.is_empty() {
            write!(res, "\nAvailable options:\n")?;
        }
        for i in flags {
            match i.short {
                Some(c) => write!(res, "    -{}", c)?,
                None => write!(res, "      ")?,
            }
            if i.short.is_some() && i.long.is_some() {
                write!(res, ", ")?
            } else {
                write!(res, "  ")?
            }
            match (i.long, i.metavar) {
                (None, None) => write!(res, "{:ident$}", "", ident = max_name_width + 2)?,
                (None, Some(m)) => write!(
                    res,
                    "<{}>{:ident$}",
                    m,
                    "",
                    ident = max_name_width - m.len()
                )?,
                (Some(l), None) => write!(res, "--{:ident$}", l, ident = max_name_width)?,
                (Some(l), Some(m)) => write!(
                    res,
                    "--{:ident$}",
                    format!("{} <{}>", l, m),
                    ident = max_name_width
                )?,
            }
            match &i.help {
                Some(h) => {
                    write!(res, "{}\n", h)?;
                }
                None => {
                    // strip unnecessary spaces inserted by previous writes
                    res.truncate(res.trim_end_matches(' ').len());
                    write!(res, "\n")?;
                }
            }
        }

        let commands = &meta.commands();
        if !commands.is_empty() {
            write!(res, "\nAvailable commands:\n")?;
        }
        let max_command_width = commands
            .iter()
            .map(|i| i.long.map(|l| l.len()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        for c in commands {
            if let Some(l) = c.long {
                write!(res, "    {:indent$}", l, indent = max_command_width)?;
            } else {
                write!(res, "    {:indent$}", "", indent = max_command_width)?;
            }
            match &c.help {
                Some(help) => {
                    write!(res, "  {}\n", help)?;
                }
                None => {
                    // strip unnecessary spaces inserted by previous writes
                    res.truncate(res.trim_end_matches(' ').len());
                    writeln!(res)?;
                }
            }
        }

        if let Some(t) = self.footer {
            write!(res, "\n{}\n", t)?;
        }
        Ok(res)
    }

    /// Attach additional information to the parser
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
                        .unwrap();
                    return Err(Error::Stdout(msg));
                }
                Ok((ExtraParams::Version, _)) => {
                    if let Some(v) = self.version {
                        return Err(Error::Stdout(format!("Version: {}", v)));
                    } else {
                        unreachable!()
                    }
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

fn check_unexpected<T>((t, args): (T, Args)) -> Result<(T, Args), Error> {
    match args.peek() {
        None => Ok((t, args)),
        Some(item) => Err(Error::Stderr(format!(
            "{} is not expected in this context",
            item
        ))),
    }
}
