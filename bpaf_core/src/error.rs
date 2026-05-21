#![allow(private_interfaces)] // Name is private at the moment
use std::{borrow::Cow, ffi::OsString};

use crate::{
    Lit, Metavar, Name,
    arg::Adjacency,
    complete::{CompReply, ShellRender},
    console_writer::Styled,
};

#[derive(Debug)]
pub enum Problem {
    Parse {
        value: Option<String>,
        error: String,
    },
    // TODO - pass Metavar?
    WrongArgument {
        name: Name<'static>,
        value: Option<OsString>,
    },
    Unconsumed {
        value: OsString,
    },
    Conflict {
        accepted: OsString,
        unexpected: Name<'static>,
    },

    ConflictPos {
        accepted: OsString,
        unexpected: OsString,
    },
    GuardFailed {
        message: &'static str,
        range: Option<OsString>,
    },
    OnlyOnce {
        name: Name<'static>,
    },
    OnlyOnceInGroup {
        group: String,
        name: char,
        ix: u32,
    },
    DidYouMean {
        target: Name<'static>,
        best: Name<'static>,
    },
    DidYouMeanCmd {
        target: String,
        best: String,
    },
    ExpectedFlag {
        name: Name<'static>,
        adj: Adjacency,
        value: OsString,
    },
    Static(&'static str),
    TryDDash {
        name: String,
    },
    Dynamic {
        err: String,
    },
    NotStrict {
        metavar: Metavar,
    },
    NotAdjacent {
        name: OsString,
        value: OsString,
    },
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::Parse { value: None, error } => {
                write!(f, "couldn't parse: {error}")
            }
            Problem::Parse {
                value: Some(value),
                error,
            } => {
                write!(f, "couldn't parse `{value}`: {error}")
            }
            Problem::Unconsumed { value } => {
                write!(
                    f,
                    "`{}` is not expected in this context",
                    value.to_string_lossy()
                )
            }
            Problem::Conflict {
                accepted,
                unexpected,
            } => {
                write!(
                    f,
                    "`{unexpected}` cannot be used at the same time as `{}`",
                    accepted.to_string_lossy(),
                )
            }
            Problem::ConflictPos {
                accepted,
                unexpected,
            } => {
                write!(
                    f,
                    "`{}` cannot be used at the same time as `{}`",
                    unexpected.to_string_lossy(),
                    accepted.to_string_lossy(),
                )
            }
            Problem::WrongArgument { name, value: None } => {
                write!(f, "`{name}` expects a value")
            }
            Problem::WrongArgument {
                name,
                value: Some(value),
            } => {
                let s = value.to_string_lossy();
                write!(
                    f,
                    "`{name}` requires an argument TODO, got a flag {s}, try {name}={s}"
                )
            }
            Problem::GuardFailed { message, range } => match range {
                Some(r) => write!(f, "`{}`: {message}", r.to_string_lossy()),
                None => f.write_str(message),
            },
            Problem::OnlyOnce { name } => {
                write!(
                    f,
                    "argument `{name}` cannot be used multiple times in this context"
                )
            }
            Problem::OnlyOnceInGroup { group, name, ix } => {
                write!(
                    f,
                    "can't parse `{name}` (item {ix}) while parsing `{group}` as a set of short flags"
                )
            }
            Problem::DidYouMean { target, best } => {
                write!(f, "no such flag: `{target}`, did you mean `{best}`?")
            }
            Problem::DidYouMeanCmd { target, best } => {
                write!(f, "no such command: `{target}`, did you mean `{best}`?")
            }
            Problem::ExpectedFlag { name, adj, value } => {
                write!(
                    f,
                    "the app can accept `{name}` as a flag, but got `{name}{adj}{}`",
                    value.to_string_lossy()
                )
            }
            Problem::Static(msg) => write!(f, "{msg}"),
            Problem::TryDDash { name } => {
                write!(
                    f,
                    "no such flag: `-{name}` (with one dash), did you mean `--{name}`?"
                )
            }
            Problem::Dynamic { err } => write!(f, "{err}"),
            Problem::NotStrict { metavar } => {
                write!(f, "expected `{metavar}` to be on the right side of `--`")
            }
            Problem::NotAdjacent { name, value } => {
                write!(
                    f,
                    "Expected value to be adjacent to {name}, try {name}={value}",
                    name = name.to_string_lossy(),
                    value = value.to_string_lossy(),
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

#[derive(Debug)]
pub enum Error {
    Missing(MissingItem),
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
    Lit {
        value: Lit<'static>,
    },
    Custom {
        item: Cow<'static, str>,
    },
}

impl std::fmt::Display for MissingItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingItem::Named { name, meta: None } => write!(f, "missing `{name}`"),
            MissingItem::Named {
                name,
                meta: Some(meta),
            } => write!(f, "missing `{name} {meta}`"),
            MissingItem::Pos { meta } => write!(f, "missing `{meta}`"),
            MissingItem::Lit { value } => write!(f, "missing `{value}`"),
            MissingItem::Custom { item: rendered } => f.write_str(rendered),
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
    pub(crate) fn combine(self, e2: Error) -> Error {
        match (self, e2) {
            // If we failed to expand `CompValue` right away
            (Error::CompValue(v), e) => Error::CompReply(CompReply::from(v)).combine(e),
            (e, Error::CompValue(v)) => e.combine(Error::CompReply(CompReply::from(v))),

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
            (e @ Error::Missing(_), _) => e,
        }
    }

    pub(crate) fn missing(item: MissingItem) -> Self {
        Self::Missing(item)
    }

    /// Consume current error and append it to a growing collection in `dst`
    ///
    /// It exists to collect errors from multiple handles and designed to work with
    /// [`Result::map_err`]. We aggregate the best possible error inside an Option
    /// and fail with that if it is present
    pub fn append_to(self, dst: &mut Option<Error>) -> Self {
        *dst = Some(match dst.take() {
            Some(e) => e.combine(self),
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
