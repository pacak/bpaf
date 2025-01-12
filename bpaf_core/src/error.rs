use crate::{
    mini_ansi::{Emphasis, Invalid},
    named::Name,
    split::{Arg, OsOrStr},
};

#[derive(Debug, Clone)]
pub enum ParseFailure {
    Stdout(String),
    Stderr(String),
    Completion(String), // TODO - use OsString?
}

impl ParseFailure {
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stdout(self) -> String {
        match self {
            ParseFailure::Stdout(o) => o,
            ParseFailure::Stderr(_) | ParseFailure::Completion(_) => {
                panic!("not an stdout: {self:?}")
            }
        }
    }

    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stderr(self) -> String {
        match self {
            ParseFailure::Stderr(o) => o,
            ParseFailure::Stdout(_) | ParseFailure::Completion(_) => {
                panic!("not an stderr: {self:?}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub(crate) message: Message,
}

impl Error {
    pub(crate) fn combine_with(self, other: Self) -> Self {
        Self {
            message: self.message.combine_with(other.message),
        }
    }

    pub(crate) fn parse_fail(message: String) -> Error {
        Self {
            message: Message::ParseFailed(None, message),
        }
    }
    pub(crate) fn try_subcommand(unparsed: Arg<'static>, command: Name<'static>) -> Self {
        Self {
            message: Message::TrySubcommand {
                value: Invalid(unparsed),
                command: Emphasis(command),
            },
        }
    }

    pub(crate) fn missing(item: MissingItem) -> Error {
        Self {
            message: Message::Missing(vec![item]),
        }
    }

    pub(crate) fn not_positional(arg: OsOrStr<'static>) -> Self {
        Self {
            message: Message::NotPositional {
                arg: Invalid(arg.to_owned()),
            },
        }
    }
    pub(crate) fn try_positional(arg: Arg<'static>, meta: Metavar) -> Self {
        Self {
            message: Message::TryPositional {
                value: Invalid(arg),
                expected: Emphasis(meta),
            },
        }
    }
    pub(crate) fn new(message: Message) -> Self {
        Self { message }
    }

    pub(crate) fn unexpected(arg: Arg<'static>) -> Self {
        Self {
            message: Message::Unexpected { arg: Invalid(arg) },
        }
    }

    pub(crate) fn fail(message: &'static str) -> Error {
        Error {
            message: Message::Fail(message),
        }
    }
    pub(crate) fn from_str_fail(value: OsOrStr<'static>, message: String) -> Self {
        Self {
            message: Message::FromStrFailed {
                value: Invalid(value),
                message,
            },
        }
    }

    pub(crate) fn empty<T>() -> Result<T, Error> {
        Err(Self {
            message: Message::Killed,
        })
    }

    pub(crate) fn handle_with_fallback(&self) -> bool {
        self.message.handle_with_fallback()
    }

    pub(crate) fn render(self) -> ParseFailure {
        self.message.render()
    }

    pub(crate) fn get_missing(self) -> Option<Vec<MissingItem>> {
        if let Message::Missing(vec) = self.message {
            Some(vec)
        } else {
            None
        }
    }
}

impl Message {
    fn combine_with(self, e2: Self) -> Self {
        match (self, e2) {
            (Self::Missing(mut m1), Self::Missing(m2)) => {
                m1.extend(m2);
                Self::Missing(m1)
            }
            (a, b) if a.handle_with_fallback() => b,
            (a, _) => a,
        }
    }
    fn handle_with_fallback(&self) -> bool {
        match self {
            // Missing is by definition can be handled with fallback
            Self::Missing(_) => true,
            Self::Fail(_) => true,
            // this should never reach the consumer - it gets generated
            // when handle gets thrown out
            Self::Killed => true,
            Self::Conflicts { .. } => false,
            Self::OnlyOnce { .. } => false,
            Self::Unexpected { .. } => false,
            Self::ParseFailed(..) => false,
            Self::NotPositional { .. } => false,
            Self::ArgNeedsValue { .. } => false,
            Self::ArgNeedsValueGotNamed { .. } => false,
            Self::FromStrFailed { .. } => false,
            Self::ParseFailure(_) => false,
            Self::TrySubcommand { .. } => false,
            Self::TryPositional { .. } => false,
            Self::TrySingleDash { .. } => false,
            Self::TryDoubleDash { .. } => false,
            Self::GuardFailed { .. } => false,
            Self::TryTypo { .. } => false,
        }
    }
}

impl From<Message> for Error {
    fn from(message: Message) -> Self {
        Error { message }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    /// Pure text message that comes from `.some()` or `fail()
    ///
    /// Problem caused by combination of multiple items and can't
    /// be addressed to one specific command line item
    Fail(&'static str),
    Missing(Vec<MissingItem>),

    /// Passed something that wasn't expected, such as `-` for an argument name, etc.
    Unexpected {
        arg: Invalid<Arg<'static>>,
    },

    /// Generated as a dummy error message...
    /// Do I need it? Not used anymore
    Killed,

    /// expected --foo BAR, found --foo
    ArgNeedsValue {
        name: Name<'static>,
        meta: Metavar,
    },

    /// expected --foo BAR, got --foo --bar, try --foo=--bar
    ArgNeedsValueGotNamed {
        name: Name<'static>,
        meta: Metavar,
        val: OsOrStr<'static>,
    },

    Conflicts {
        winner: Emphasis<Name<'static>>,
        loser: Invalid<Name<'static>>,
    },

    NotPositional {
        arg: Invalid<OsOrStr<'static>>,
    },
    GuardFailed {
        message: &'static str,
    },
    // // those can be caught ---------------------------------------------------------------
    // /// Tried to consume an env variable with no fallback, variable was not set
    // NoEnv(&'static str),
    //
    // /// User specified an error message on some
    // ParseSome(&'static str),
    //
    // /// User asked for parser to fail explicitly
    // ParseFail(&'static str),
    //
    // /// pure_with failed to parse a value
    // PureFailed(String),
    //
    // /// Expected one of those values
    // ///
    // /// Used internally to generate better error messages
    // Missing(Vec<MissingItem>),
    //
    // // those cannot be caught-------------------------------------------------------------
    // /// Parsing failed and this is the final output
    // ParseFailure(ParseFailure),
    FromStrFailed {
        value: Invalid<OsOrStr<'static>>,
        message: String,
    },
    // /// Tried to consume a strict positional argument, value was present but was not strictly
    // /// positional
    // StrictPos(usize, Metavar),
    //
    // /// Tried to consume a non-strict positional argument, but the value was strict
    // NonStrictPos(usize, Metavar),
    //
    /// Parser provided by user failed to parse a value
    ParseFailed(Option<usize>, String),
    TrySubcommand {
        value: Invalid<Arg<'static>>,
        command: Emphasis<Name<'static>>,
    },
    TryPositional {
        value: Invalid<Arg<'static>>,
        expected: Emphasis<Metavar>,
    },

    TrySingleDash {
        input: Invalid<Name<'static>>,
        short: Emphasis<Name<'static>>,
    },
    TryDoubleDash {
        input: Invalid<String>,
        long: Emphasis<Name<'static>>,
    },
    TryTypo {
        input: Invalid<Name<'static>>,
        long: Emphasis<Name<'static>>,
    },
    //
    // /// Parser provided by user failed to validate a value
    // GuardFailed(Option<usize>, &'static str),
    //
    // /// Argument requres a value but something else was passed,
    // /// required: --foo <BAR>
    // /// given: --foo --bar
    // ///        --foo -- bar
    // ///        --foo
    // NoArgument(usize, Metavar),
    //
    // /// Parser is expected to consume all the things from the command line
    // /// this item will contain an index of the unconsumed value
    // Unconsumed(/* TODO - unused? */ usize),
    //
    // /// argument is ambigoups - parser can accept it as both a set of flags and a short flag with no =
    // Ambiguity(usize, String),
    //
    // /// Suggested fixes for typos or missing input
    // Suggestion(usize, Suggestion),
    //
    // /// Two arguments are mutually exclusive
    // /// --release --dev
    // Conflict(/* winner */ usize, usize),
    //
    // /// Expected one or more items in the scope, got someting else if any
    // Expected(Vec<Item>, Option<usize>),
    //
    /// Parameter is accepted but only once
    OnlyOnce {
        name: Name<'static>,
    },

    ParseFailure(ParseFailure),
}

impl Message {
    pub(crate) fn render(self) -> ParseFailure {
        self.write_message()
            .expect("write to string shouldn't fail")
    }
    fn write_message(self) -> Result<ParseFailure, std::fmt::Error> {
        use std::fmt::Write;
        let mut res = String::new();
        match self {
            Message::Fail(msg) => write!(res, "failed: {msg}")?,
            Message::Missing(xs) => {
                write!(res, "expected ")?;
                write_missing(&mut res, &xs)?; // TODO - a newtype wrapper for a slice?
                                               // Something that picks the best item and returns it
                                               // as `impl Display`?
                // TODO - can we use real --help command? Or can we drop this part if custom help
                // is in use?
                write!(res, ", pass {} for usage information", Emphasis(Name::long("help")))?;
            }
            Message::Conflicts { winner, loser } =>
                write!(res, "{loser} cannot be used at the same time as {winner}")?,


            Message::Unexpected { arg } => write!(res, "{arg} is not expected in this context")?,
            Message::NotPositional { arg } => write!(res, "{arg} not positional TODO")?,
            Message::Killed => todo!(),
            Message::ParseFailed(_, x) => write!(res, "{x}")?,
            Message::ArgNeedsValue { name, meta } => write!(res, "{name} wants a value {meta}")?,
            Message::ArgNeedsValueGotNamed { name, meta, val } => write!(
                res,
                "{arg_parser} wants a value {meta}, got {bad_val}, try using {name}={val}",
                arg_parser = Emphasis(&name),
                bad_val = Invalid(&val)
            )?,
            Message::OnlyOnce { name } => write!(
                res,
                "argument `{name}` cannot be used multiple times in this context"
            )?,
            Message::ParseFailure(parse_failure) => return Ok(parse_failure),
            Message::FromStrFailed { value, message } => {
                write!(res, "couldn't parse {value}: {message}")?
            }
            Message::TrySubcommand { value, command } => write!(res,
                "{value} is not valid in this context, did you mean to pass it to command {command:#}?")?,

            Message::TryPositional { value, expected } => write!(res,
                "Parser expects a positional {expected}, got a named {value}. If you meant to use it as {expected} - try inserting {ddash} in front of it", ddash = Emphasis("--"))?,
            Message::TrySingleDash { input, short } => write!(res, "no such flag: {input} (with two dashes), did you mean {short}?")?,
            Message::TryDoubleDash{ input, long } => write!(res, "no such flag: {input} (with one dash), did you mean {long}?")?,
            Message::TryTypo { input, long } => write!(res, "no such flag: {input}, did you mean {long}?")?,
            Message::GuardFailed { message } => res.push_str(message),

        }
        res = crate::mini_ansi::mono(res);
        Ok(ParseFailure::Stderr(res))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Metavar(pub(crate) &'static str);
impl std::fmt::Display for Metavar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_angled() {
            write!(f, "<{}>", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl Metavar {
    pub(crate) fn is_angled(&self) -> bool {
        !self
            .0
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '-' || c == '_')
    }
}

#[test]
fn metavar_display_and_with() {
    let a = Metavar("A");
    assert_eq!(a.to_string(), "A");
    assert_eq!(a.width(), 1);

    let a = Metavar("a|b");
    assert_eq!(a.to_string(), "<a|b>");
    assert_eq!(a.width(), 5);
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum MissingItem {
    Named {
        name: Vec<Name<'static>>,
        meta: Option<Metavar>,
    },
    Positional {
        meta: Metavar,
    },
    Command {
        name: Vec<Name<'static>>,
    },
    Any {
        meta: Metavar,
    },
    Env {
        name: &'static str,
    },
}

/// Pick a missing item that best describes the situation
fn write_missing(res: &mut String, items: &[MissingItem]) -> std::fmt::Result {
    use std::fmt::Write;
    if let Some(meta) = items.iter().find_map(|i| match i {
        MissingItem::Positional { meta } => Some(meta),
        MissingItem::Any { meta } => Some(meta),
        _ => None,
    }) {
        write!(res, "{meta}")?;
    } else if let Some((name, meta)) = items.iter().find_map(|i| match i {
        MissingItem::Named { name, meta } => Some((first_good_name(name)?, meta)),
        _ => None,
    }) {
        // TODO - figure emphasis
        match meta {
            Some(meta) => write!(res, "{name}={meta}")?,
            None => write!(res, "{name}")?,
        }
    } else if items
        .iter()
        .any(|i| matches!(i, MissingItem::Command { .. }))
    {
        write!(res, "{}", Emphasis("COMMAND"))?;
    } else {
        write!(res, "some hidden item")?;
    }
    Ok(())
}

pub(crate) fn first_good_name<'a>(names: &'a [Name<'a>]) -> Option<Name<'a>> {
    names.first().map(|n| n.as_ref())
}
