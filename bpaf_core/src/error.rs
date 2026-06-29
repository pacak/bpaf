#![allow(private_interfaces)] // Name is private at the moment
use crate::{
    Kind, Lit, Metavar, Name,
    arg::Adjacency,
    complete::{CompReply, ShellRender},
    console_writer::Styled,
};

const I: &str = crate::console_writer::Style::Invalid.ansi(); // invalid
const V: &str = crate::console_writer::Style::Valid.ansi(); // valid
const R: &str = crate::console_writer::Style::Text.ansi(); // reset
const M: &str = crate::console_writer::Style::Metavar.ansi(); // metavar
const Q: &str = crate::console_writer::Style::MonoTick.ansi(); // quote

#[derive(Debug)]
pub enum Problem {
    /// Parser inside of a `.parse()` had failed
    /// The original value is available for [`Leaf`] parsers
    /// and not available for composite ones.
    Parse {
        value: Option<String>,
        error: String,
    },
    /// An argument expected a value, got either nothing at all or a named value
    WrongArgument {
        meta: Metavar,
        name: Name<'static>,
        value: Option<String>,
    },
    /// Got a named value that conflicts with a different value
    Conflict {
        accepted: String,
        unexpected: Name<'static>,
    },

    /// Got a positional value that might conflict with a different value
    ///
    /// Unlike named conflict we are not 100% sure about that - positional items
    /// are all alike
    ConflictPos {
        accepted: String,
        unexpected: String,
    },
    /// Got something we don't know how to parse and there's no matching conflict
    /// that can apply
    Unconsumed { value: String },

    /// Managed to parse a value successfully, but check inside of a .guard() failed
    ///
    /// For Leaf parsers we have a value we are trying to parse, for non leaf - there's no value
    GuardFailed {
        message: &'static str,
        range: Option<String>,
    },
    /// We managed to parse a similar named value before, we might be able to parse
    /// it later if something restarts, but
    OnlyOnce { name: Name<'static> },
    /// We are parsing a group of stacked short flags. A char `name` at position `ix` was valid
    /// before we started parsing, but we can't parse it now. Maybe it was used more than once,
    /// maybe it conflicts with something else.
    OnlyOnceInGroup { group: String, name: char, ix: u32 },
    /// Expected a flag, but got it with an attached value.
    ExpectedFlag {
        name: Name<'static>,
        adj: Adjacency,
        value: String,
    },
    /// Got an unknown name, found something very close to it
    DidYouName {
        target: Name<'static>,
        best: Name<'static>,
    },
    /// Got an unknown literal, found something very close to it
    DidYouMeanLit { target: String, best: String },
    /// found an input that looks like a valid long name, but lacks a single dash at the front
    TryDDash { name: String },

    /// A missing item kind of error that can't be caught, should be already rendered
    Dynamic { err: String },
    /// a positional value that was marked as a strict one is located either before `--` or `--` is
    /// not present at all
    NotStrict { metavar: Metavar, string: String },
    /// a named value was marked as adjacent only, but was entered as two separate items
    NotAdjacent { name: String, value: String },
    /// parser failed with a missing item, executor find something it can't consume
    MissingGot { missing: Missing, value: String },
    /// flag `name` is not valid in this context, but can be passed to `cmd`
    TryInCommand {
        cmd: Lit<'static>,
        name: Name<'static>,
    },
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::Parse { value: None, error } => {
                write!(f, "parse error: {error}")
            }
            Problem::Parse {
                value: Some(value),
                error,
            } => {
                write!(f, "couldn't parse {Q}{I}{value}{R}{Q}: {error}")
            }

            Problem::Unconsumed { value } => {
                write!(f, "{Q}{I}{value}{R}{Q} is not expected in this context")
            }
            Problem::Conflict {
                accepted,
                unexpected,
            } => {
                write!(
                    f,
                    "{Q}{I}{unexpected}{R}{Q} cannot be used at the same time as {Q}{V}{accepted}{R}{Q}"
                )
            }
            Problem::ConflictPos {
                accepted,
                unexpected,
            } => {
                write!(
                    f,
                    "can't parse {Q}{I}{unexpected}{R}{Q}, likely conflicts with {Q}{V}{accepted}{R}{Q}"
                )
            }
            Problem::WrongArgument {
                name,
                meta,
                value: None,
            } => {
                write!(f, "{Q}{I}{name}{R}{Q} expects a value {Q}{M}{meta}{R}{Q}")
            }
            Problem::WrongArgument {
                name,
                meta,
                value: Some(value),
            } => {
                write!(
                    f,
                    "{Q}{V}{name}{R}{Q} requires an argument {Q}{M}{meta}{R}{Q}, got a {Q}{I}{value}{R}{Q}, try {Q}{V}{name}={value}{R}{Q} to use it as an argument"
                )
            }
            Problem::GuardFailed { message, range } => match range {
                Some(r) => write!(f, "{Q}{I}{r}{R}{Q}: {message}"),
                None => write!(f, "{message}"),
            },
            Problem::OnlyOnce { name } => {
                write!(
                    f,
                    "argument {Q}{V}{name}{R}{Q} cannot be used multiple times in this context"
                )
            }
            Problem::OnlyOnceInGroup { group, name, ix } => {
                write!(
                    f,
                    "can't parse {Q}{I}{name}{R}{Q} (item {ix}) while parsing {Q}{V}{group}{R}{Q} as a set of stacked short flags"
                )
            }
            Problem::DidYouName { target, best } => {
                write!(
                    f,
                    "no such flag: {Q}{I}{target}{R}{Q}, did you mean {Q}{V}{best}{R}{Q}?"
                )
            }
            Problem::DidYouMeanLit { target, best } => {
                write!(
                    f,
                    "no such command: {Q}{I}{target}{R}{Q}, did you mean {Q}{V}{best}{R}{Q}?"
                )
            }
            Problem::ExpectedFlag { name, adj, value } => {
                write!(
                    f,
                    "the app can accept {Q}{V}{name}{R}{Q} as a flag, but got {Q}{I}{name}{adj}{value}{I}{Q}"
                )
            }
            Problem::TryDDash { name } => {
                write!(
                    f,
                    "no such flag: {Q}{I}-{name}{R}{Q} (with one dash), did you mean {Q}{V}--{name}{R}{Q}?"
                )
            }
            Problem::Dynamic { err } => write!(f, "{err}"),
            Problem::NotStrict { metavar, string } => {
                write!(
                    f,
                    "expected {Q}{I}{string}{R}{Q} ({M}{metavar}{R}) to follow {Q}{V}--{R}{Q}"
                )
            }
            Problem::NotAdjacent { name, value } => {
                write!(
                    f,
                    "expected value to be adjacent to {V}{name}{R}, try {V}{name}={value}{R}"
                )
            }
            Problem::MissingGot {
                missing:
                    missing @ Missing {
                        item: MissingItem::Some { .. },
                        ..
                    },
                ..
            } => {
                // keep the error from `.some("error")` separately even if there's unexpected input
                writeln!(f, "{missing}")
            }
            Problem::MissingGot { missing, value } => {
                write!(f, "{missing}, got {Q}{value}{Q}")
            }
            Problem::TryInCommand { cmd, name } => {
                write!(
                    f,
                    "flag {Q}{name}{Q} is not valid in this context, did you mean to pass it to command {Q}{cmd}{Q}?"
                )
            }
        }
    }
}

/// In flight completion reply
/// Completion generates two types of replies:
/// - items (name/meta/help)
/// - values (value + help)
///
/// For the items I want to be able to replace help message - help might contain a very detailed
/// description that shouldn't go into completion.
/// For values - I might be able to expand potential suffixes, so `--name a<TAB>` can expand
/// into all potential names that start with `a`.
#[derive(Debug)]
pub(crate) struct CV {
    /// can contain following items, in order:
    /// 1. name - for completions with adjacent name but also for items
    /// 2. adjacency (`=` or ``) - for completions only
    /// 3. value being completed - can be empty
    pub(crate) prefix_value: String,
    /// should value completion be called when seen
    pub(crate) has_value: bool,
    /// which portion of the prefix_value is prefix we keep. completion function gets
    /// `&prefix_value[prefix_len..]`
    pub(crate) prefix_len: u32,
    /// `prefix_value` holds a metavar placeholder instead of real user input.
    ///
    /// When the user hasn't typed anything yet (e.g. `command <TAB>` for a positional),
    /// there's no real input to show or complete. Instead, the metavar (e.g. `"X"`) is
    /// stored in `prefix_value` so the fallback `From<CV> for CompReply` path can display
    /// it as a hint: `X\tpos help`. It's not a real value so it shouldn't be passed to
    /// the completer.
    pub(crate) meta_only: bool,
    pub(crate) help: Option<&'static str>,
    /// We'll need this to convert CV into CompReply when merging several errors together
    pub(crate) shell: ShellRender,
}

/// Quantity indicator for missing items
///
/// We are not collecting info about all the missing items, but printing that
/// there's just one item missing where it is actually more can be confusing
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MissingCount {
    /// There's actually just one item missing
    /// ""
    One,
    /// There's a product of two or more missing items
    /// OR
    /// There's a sum of two or more branches and each branch requires at least two missing items
    /// " (and more)"
    AndMore,
    /// There's a sum and one of the branches is just one item
    /// " (or more)"
    OrMore,
}

#[derive(Debug)]
pub struct Missing {
    count: MissingCount,
    item: MissingItem,
}

impl Missing {
    fn combine(self, other: Self, kind: Kind) -> Self {
        match (self.count, other.count, kind) {
            (MissingCount::One, MissingCount::One, Kind::Sum) => Self {
                count: MissingCount::OrMore,
                item: self.item.pick(other.item),
            },
            (MissingCount::One, _, Kind::Sum) => Missing {
                count: MissingCount::OrMore,
                item: self.item,
            },
            (_, MissingCount::One, Kind::Sum) => Missing {
                count: MissingCount::OrMore,
                item: other.item,
            },
            (_, _, Kind::Prod) => Self {
                count: MissingCount::AndMore,
                item: self.item.pick(other.item),
            },
            (MissingCount::AndMore, MissingCount::AndMore, Kind::Sum) => Self {
                count: MissingCount::AndMore,
                item: self.item.pick(other.item),
            },
            (MissingCount::OrMore, MissingCount::OrMore, Kind::Sum) => Self {
                count: MissingCount::OrMore,
                item: self.item.pick(other.item),
            },
            (MissingCount::OrMore, MissingCount::AndMore, Kind::Sum) => self,
            (MissingCount::AndMore, MissingCount::OrMore, Kind::Sum) => other,
        }
    }
}

impl MissingItem {
    fn pick(self, other: Self) -> Self {
        use MissingItem as M;
        match (self, other) {
            (c @ M::Some { .. }, _) | (_, c @ M::Some { .. }) => c,
            (cmd @ M::Cmd { .. }, _) | (_, cmd @ M::Cmd { .. }) => cmd,
            (pos @ M::Pos { .. }, _) | (_, pos @ M::Pos { .. }) => pos,
            (lit @ M::Lit { .. }, _) | (_, lit @ M::Lit { .. }) => lit,
            (n @ M::Named { .. }, _) | (_, n @ M::Named { .. }) => n,
            (first, _) => first,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Missing(Missing),
    CompReply(CompReply),
    /// Generated by the executor and hopefully handled by the user with `.complete()`
    CompValue(CV),
    /// `u32` describes the index where problem occurs - we want to try to report
    /// earliest possible issue
    Problem(u32, Problem),
    Final(ParseFailure),
    Silent(&'static str),
}

impl Error {
    pub(crate) const OUTCONSUMED: Self =
        Error::Silent("Task was terminated because alternative branch consumed more");
}

#[derive(Debug, Clone)]
pub enum MissingItem {
    Named {
        name: Name<'static>,
        meta: Option<Metavar>,
    },
    Pos {
        meta: Metavar,
    },
    /// Keyword parameter
    Lit {
        value: Lit<'static>,
    },
    /// A command
    Cmd {
        _value: Lit<'static>,
    },
    EnvVar {
        var_name: &'static str,
    },
    /// Not an actually missing item, but an error message from `.some("msg")` that pretends to be one
    ///
    /// Obeys mostly the same rules as missing item (can be caught, lower priority), but renders
    /// a bit differently to avoid confusing wording
    Some {
        item: String,
    },
}

impl std::fmt::Display for Missing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(
            self.item,
            MissingItem::Some { .. } | MissingItem::Cmd { .. }
        ) {
            write!(f, "{}", self.item)
        } else {
            write!(f, "{}{}", self.item, self.count)
        }
    }
}

impl std::fmt::Display for MissingCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingCount::One => Ok(()),
            MissingCount::AndMore => write!(f, ", and more"),
            MissingCount::OrMore => write!(f, ", or more"),
        }
    }
}

impl std::fmt::Display for MissingItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingItem::Named { name, meta: None } => write!(f, "expected {Q}{V}{name}{R}{Q}"),
            MissingItem::Named {
                name,
                meta: Some(meta),
            } => write!(f, "expected {Q}{V}{name}={meta}{R}{Q}"),
            MissingItem::Pos { meta } => write!(f, "expected {Q}{M}{meta}{R}{Q}"),
            MissingItem::Lit { value } => write!(f, "expected {Q}{V}{value}{R}{Q}"),
            MissingItem::Cmd { _value: _ } => write!(f, "expected {Q}{V}COMMAND ...{R}{Q}"),

            MissingItem::EnvVar { var_name } => {
                write!(f, "env variable {Q}{V}{var_name}{R}{Q} is not set")
            }
            MissingItem::Some { item } => write!(f, "{item}"),
        }
    }
}

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseFailure::Stdout(m) | ParseFailure::Stderr(m) => f.write_str(&m.mono()),
            ParseFailure::Console(c) => write!(f, "{c:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParseFailure {
    Stdout(Styled),
    Stderr(Styled),
    Console(String),
}

impl ParseFailure {
    pub(crate) fn stderr(raw: String) -> Self {
        ParseFailure::Stderr(Styled { raw, tab: 0 })
    }

    pub(crate) fn stdout(raw: String) -> Self {
        ParseFailure::Stdout(Styled { raw, tab: 0 })
    }
}

impl From<ParseFailure> for Error {
    fn from(value: ParseFailure) -> Self {
        Error::Final(value)
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &str) -> ! {
    panic!("{msg}: {error:?}");
}
impl ParseFailure {
    #[track_caller]
    pub fn unwrap_stdout(self) -> String {
        match self {
            ParseFailure::Stdout(s) => s.mono(),
            ParseFailure::Console(c) => c,
            ParseFailure::Stderr(e) => unwrap_failed(
                "called `ParseFailure::unwrap_stdout()` on Stderr",
                &e.mono(),
            ),
        }
    }

    #[track_caller]
    pub fn unwrap_stderr(self) -> String {
        match self {
            ParseFailure::Stderr(s) => s.mono(),
            ParseFailure::Stdout(e) => unwrap_failed(
                "called `ParseFailure::unwrap_stderr()` on Stdout",
                &e.mono(),
            ),
            ParseFailure::Console(_) => {
                unwrap_failed("called `ParseFailure::unwrap_stderr()` on Console", "")
            }
        }
    }
}

impl From<Error> for ParseFailure {
    fn from(value: Error) -> Self {
        match value {
            Error::Missing(m) => ParseFailure::stderr(m.to_string()),
            Error::CompReply(CompReply(reply)) => ParseFailure::Console(reply),
            Error::Problem(_, problem) => ParseFailure::stderr(problem.to_string()),
            Error::Final(parse_failure) => parse_failure,
            Error::Silent(reason) => {
                ParseFailure::stderr(format!("internal error, got unexpected silent {reason}"))
            }
            Error::CompValue(cv) => ParseFailure::Console(CompReply::from(cv).0),
        }
    }
}

impl Error {
    pub(crate) fn with_executor(self, e: Error) -> Self {
        match (self, e) {
            (Error::Missing(missing), Error::Problem(offset, Problem::Unconsumed { value })) => {
                Error::Problem(offset, Problem::MissingGot { missing, value })
            }
            (e1, e2) => e1.combine(e2, Kind::Prod),
        }
    }
    pub(crate) fn combine(self, e2: Error, kind: Kind) -> Error {
        match (self, e2) {
            // If we failed to expand `CompValue` right away
            (Error::CompValue(v), e) => Error::CompReply(CompReply::from(v)).combine(e, kind),
            (e, Error::CompValue(v)) => e.combine(Error::CompReply(CompReply::from(v)), kind),

            (e @ Error::Final(_), _) | (_, e @ Error::Final(_)) => e,
            (Error::Silent(_), e) | (e, Error::Silent(_)) => e,
            (Error::CompReply(c1), Error::CompReply(c2)) => Error::CompReply(c1 + c2),
            (e @ Error::CompReply(_), _) | (_, e @ Error::CompReply(_)) => e,
            (e1 @ Error::Problem(o1, _), e2 @ Error::Problem(o2, _)) => {
                if o1 > o2 {
                    e2
                } else {
                    e1
                }
            }
            (e @ Error::Problem(..), _) | (_, e @ Error::Problem(..)) => e,
            (Error::Missing(m1), Error::Missing(m2)) => Error::Missing(m1.combine(m2, kind)),
        }
    }

    pub(crate) fn missing(item: MissingItem) -> Self {
        Self::Missing(Missing {
            count: MissingCount::One,
            item,
        })
    }

    /// Consume current error and append it to a growing collection in `dst`
    ///
    /// It exists to collect errors from multiple handles and designed to work with
    /// [`Result::map_err`]. We aggregate the best possible error inside an Option
    /// and fail with that if it is present
    pub fn append_to(self, dst: &mut Option<Error>) -> Self {
        *dst = Some(match dst.take() {
            Some(e) => e.combine(self, Kind::Prod),
            None => self,
        });
        Error::Silent("swapped by append_to")
    }

    /// Convert the error into a final version
    ///
    /// Used to convert errors from sub parsers (command parser, etc) that must take priority
    pub(crate) fn finalize_problems(self) -> Error {
        Error::Final(ParseFailure::from(self))
    }
}
