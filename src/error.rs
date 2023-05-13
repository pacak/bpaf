use std::ops::Range;

use crate::{item::Item, meta_help::Metavar};

/// Unsuccessful command line parsing outcome, internal representation
#[derive(Debug)]
pub enum Error {
    /// Parsing failed, it is still possible to improve the error message
    /// Some messages can be caught with .fallback or variants
    Message(Message),
    /// Parsing failed and this is the final output
    ParseFailure(ParseFailure),
    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<MissingItem>),
}

#[derive(Debug)]
pub enum Message {
    /// Tried to consume an env variable with no fallback, variable was not set
    NoEnv(&'static str),
    /// Tried to consume a strict positional argument, value was present but was not strictly
    /// positional
    StrictPos(Metavar),
    /// User specified an error message on some
    ParseSome(&'static str),
    /// User specified guard failed
    Guard(&'static str),
    /// User asked for parser to fail explicitly
    ParseFail(&'static str),

    /// Parser provided by user failed to parse a value
    ParseFailed(Option<usize>, String),

    /// Parser provided by user failed to validate a value
    ValidateFailed(Option<usize>, String),

    /// pure_with failed to parse a value
    PureFailed(String),

    /// Argument requres a value but something else was passed,
    /// required: --foo <BAR>
    /// given: --foo --bar
    ///        --foo -- bar
    ///        --foo
    NoArgument(usize),
}
impl Message {
    pub(crate) fn can_catch(&self) -> bool {
        match self {
            Message::NoEnv(_) => true,
            Message::StrictPos(_) => false,
            Message::ParseSome(_) => true,
            Message::Guard(_) => false,
            Message::ParseFail(_) => true,
            Message::ParseFailed(_, _) => false,
            Message::ValidateFailed(_, _) => false,
            Message::PureFailed(_) => true,
            Message::NoArgument(_) => false,
        }
    }
}

/// Missing item in a context
#[derive(Debug, Clone)]
pub struct MissingItem {
    /// Item that is missing
    pub(crate) item: Item,
    /// Position it is missing from - exact for positionals, earliest possible for flags
    pub(crate) position: usize,
    /// Range where search was performed, important for combinators that narrow the search scope
    /// such as adjacent
    pub(crate) scope: Range<usize>,
}

impl Error {
    #[must_use]
    pub(crate) fn combine_with(self, other: Self) -> Self {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // help output takes priority
            (a @ Error::ParseFailure(_), _) => a,
            (_, b @ Error::ParseFailure(_)) => b,

            // unconditional parsing failure takes priority
            (a @ Error::Message(_), _) => a,
            (_, b @ Error::Message(_)) => b,

            // combine missing elements
            (Error::Missing(mut a), Error::Missing(mut b)) => {
                a.append(&mut b);
                Error::Missing(a)
            }
        }
    }
    pub(crate) fn can_catch(&self) -> bool {
        match self {
            Error::Message(msg) => msg.can_catch(),
            Error::ParseFailure(_) => false,
            Error::Missing(_) => true,
        }
    }
}

/// Unsuccessful command line parsing outcome, use it for unit tests
///
/// Useful for unit testing for user parsers, consume it with
/// [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`]
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Print this to stdout and exit with success code
    Stdout(String),
    /// Print this to stderr and exit with failure code
    Stderr(String),
}

impl ParseFailure {
    /// Returns the contained `stderr` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stdout`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Self::Stderr(err) => err,
            Self::Stdout(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    /// Returns the contained `stdout` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stderr`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Self::Stdout(err) => err,
            Self::Stderr(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }

    /// Run an action appropriate to the failure and produce the exit code
    ///
    /// Prints a message to `stdout` or `stderr` and returns the exit code
    #[allow(clippy::must_use_candidate)]
    pub fn exit_code(self) -> i32 {
        match self {
            ParseFailure::Stdout(msg) => {
                print!("{}", msg); // completions are sad otherwise
                0
            }
            ParseFailure::Stderr(msg) => {
                eprintln!("{}", msg);
                1
            }
        }
    }
}
