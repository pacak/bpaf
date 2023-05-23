use std::ops::Range;

use crate::{item::Item, meta_help::Metavar, meta_youmean::Suggestion};

/// Unsuccessful command line parsing outcome, internal representation
#[derive(Debug)]
pub struct Error(pub(crate) Message);

impl Error {
    pub(crate) fn combine_with(self, other: Self) -> Self {
        Error(self.0.combine_with(other.0))
    }
}

#[derive(Debug)]
pub(crate) enum Message {
    // those can be caught ---------------------------------------------------------------
    /// Tried to consume an env variable with no fallback, variable was not set
    NoEnv(&'static str),

    /// User specified an error message on some
    ParseSome(&'static str),

    /// User asked for parser to fail explicitly
    ParseFail(&'static str),

    /// pure_with failed to parse a value
    PureFailed(String),

    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<MissingItem>),

    // those cannot be caught-------------------------------------------------------------
    /// Parsing failed and this is the final output
    ParseFailure(ParseFailure),
    /// Tried to consume a strict positional argument, value was present but was not strictly
    /// positional
    StrictPos(usize, Metavar),

    /// Parser provided by user failed to parse a value
    ParseFailed(Option<usize>, String),

    /// Parser provided by user failed to validate a value
    GuardFailed(Option<usize>, &'static str),

    /// Argument requres a value but something else was passed,
    /// required: --foo <BAR>
    /// given: --foo --bar
    ///        --foo -- bar
    ///        --foo
    NoArgument(usize, Metavar),

    /// Parser is expected to consume all the things from the command line
    /// this item will contain an index of the unconsumed value
    Unconsumed(/* TODO - unused? */ usize),

    /// argument is ambigoups - parser can accept it as both a set of flags and a short flag with no =
    Ambiguity(usize, String),

    /// Suggested fixes for typos or missing input
    Suggestion(usize, Suggestion),

    /// Two arguments are mutually exclusive
    /// --release --dev
    Conflict(/* winner */ usize, usize),

    /// Expected one or more items in the scope, got someting else if any
    Expected(Vec<Item>, Option<usize>),
}

impl Message {
    pub(crate) fn can_catch(&self) -> bool {
        match self {
            Message::NoEnv(_)
            | Message::ParseSome(_)
            | Message::ParseFail(_)
            | Message::Missing(_)
            | Message::PureFailed(_) => true,
            Message::StrictPos(_, _)
            | Message::ParseFailed(_, _)
            | Message::GuardFailed(_, _)
            | Message::Unconsumed(_)
            | Message::Ambiguity(_, _)
            | Message::Suggestion(_, _)
            | Message::Conflict(_, _)
            | Message::ParseFailure(_)
            | Message::Expected(_, _)
            | Message::NoArgument(_, _) => false,
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

impl Message {
    #[must_use]
    pub(crate) fn combine_with(self, other: Self) -> Self {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // help output takes priority
            (a @ Message::ParseFailure(_), _) => a,
            (_, b @ Message::ParseFailure(_)) => b,

            // combine missing elements
            (Message::Missing(mut a), Message::Missing(mut b)) => {
                a.append(&mut b);
                Message::Missing(a)
            }

            // otherwise earliest wins
            (a, b) => {
                if a.can_catch() {
                    b
                } else {
                    a
                }
            }
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
