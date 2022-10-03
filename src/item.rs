use crate::{parsers::NamedArg, Meta};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Item {
    Positional {
        metavar: &'static str,
        strict: bool,
        help: Option<String>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<String>,
        meta: Box<Meta>,
    },
    Flag {
        name: ShortLong,
        shorts: Vec<char>,
        help: Option<String>,
    },
    Argument {
        name: ShortLong,
        shorts: Vec<char>,
        metavar: &'static str,
        env: Option<&'static str>,
        help: Option<String>,
    },
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ShortLong {
    Short(char),
    Long(&'static str),
    ShortLong(char, &'static str),
}

impl From<&NamedArg> for ShortLong {
    fn from(named: &NamedArg) -> Self {
        match (named.short.is_empty(), named.long.is_empty()) {
            (true, true) => unreachable!("Named should have either short or long name"),
            (true, false) => Self::Long(named.long[0]),
            (false, true) => Self::Short(named.short[0]),
            (false, false) => Self::ShortLong(named.short[0], named.long[0]),
        }
    }
}

/// {} renders a version for short usage string
/// supports padding of the help by some max width
impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Positional {
                metavar,
                help: _,
                strict: _,
            } => write!(f, "<{}>", metavar),
            Item::Command { .. } => write!(f, "COMMAND ..."),
            Item::Flag {
                name,
                help: _,
                shorts: _,
            } => write!(f, "{}", name),
            Item::Argument {
                name,
                metavar,
                help: _,
                env: _,
                shorts: _,
            } => write!(f, "{} {}", name, metavar),
        }
    }
}

impl std::fmt::Display for ShortLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortLong::Short(short) | ShortLong::ShortLong(short, _) => write!(f, "-{}", short),
            ShortLong::Long(long) => write!(f, "--{}", long),
        }
    }
}

impl Item {
    #[must_use]
    pub(crate) fn required(self, required: bool) -> Meta {
        if required {
            Meta::Item(self)
        } else {
            Meta::Optional(Box::new(Meta::Item(self)))
        }
    }
}
