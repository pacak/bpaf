use crate::{named::Name, split::OsOrStr};

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

    pub(crate) fn unexpected<T>() -> Result<T, Error> {
        Err(Self {
            message: Message::Unexpected,
        })
    }

    pub(crate) fn parse_fail(message: String) -> Error {
        Self {
            message: Message::ParseFailed(None, message),
        }
    }

    pub(crate) fn missing(item: MissingItem) -> Error {
        Self {
            message: Message::Missing(vec![item]),
        }
    }

    pub(crate) fn fail(message: &'static str) -> Error {
        Error {
            message: Message::Fail(message),
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

    pub(crate) fn render(self) -> String {
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
            Self::Unexpected => false,
            Self::ParseFailed(..) => false,
            Self::ArgNeedsValue { .. } => false,
            Self::ArgNeedsValueGotNamed { .. } => false,
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
    Unexpected,

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
        winner: Name<'static>,
        loser: Name<'static>,
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
    //
    // /// Tried to consume a strict positional argument, value was present but was not strictly
    // /// positional
    // StrictPos(usize, Metavar),
    //
    // /// Tried to consume a non-strict positional argument, but the value was strict
    // NonStrictPos(usize, Metavar),
    //
    /// Parser provided by user failed to parse a value
    ParseFailed(Option<usize>, String),
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
}

impl Message {
    pub(crate) fn render(&self) -> String {
        self.write_message()
            .expect("write to string shouldn't fail")
    }
    fn write_message(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut res = String::new();
        match self {
            Message::Fail(msg) => write!(res, "failed: {msg}")?,
            Message::Missing(xs) => {
                write!(res, "Expected ")?;
                for (ix, item) in xs.iter().take(4).enumerate() {
                    if ix > 0 {
                        write!(res, ", ")?;
                    }
                    match item {
                        MissingItem::Named { name, meta } => match (&name[0], meta) {
                            (name, None) => write!(res, "{name}")?,
                            (name, Some(m)) => write!(res, "{name}={m}")?,
                        },
                        MissingItem::Positional { meta } => write!(res, "{}", meta)?,
                        MissingItem::Command { name: _ } => write!(res, "COMMAND")?,
                        MissingItem::Any { metavar } => todo!(),
                        MissingItem::Env { name } => todo!(),
                    }
                }
            }
            Message::Conflicts { winner, loser } => {
                write!(res, "{loser} cannot be used at the same time as {winner}")?
            }
            Message::Unexpected => write!(res, "unexpected item!")?, // <- TODO
            Message::Killed => todo!(),
            Message::ParseFailed(_, x) => write!(res, "{x}")?,
            Message::ArgNeedsValue { name, meta } => write!(res, "{name} wants a value {meta}")?,
            Message::ArgNeedsValueGotNamed { name, meta, val } => write!(
                res,
                "{name} wants a value {meta}, got {val}, try using {name}={val}"
            )?,
            Message::OnlyOnce { name } => write!(
                res,
                "argument `{name}` cannot be used multiple times in this context"
            )?,
        }
        Ok(res)
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
        metavar: &'static str,
    },
    Env {
        name: &'static str,
    },
}
