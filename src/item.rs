use crate::{Meta, Named};

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item {
    Positional {
        metavar: &'static str,
        help: Option<String>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<String>,
    },
    Flag {
        name: ShortLong,
        help: Option<String>,
    },
    Argument {
        name: ShortLong,
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

impl From<&Named> for ShortLong {
    fn from(named: &Named) -> Self {
        match (named.short.is_empty(), named.long.is_empty()) {
            (true, true) => unreachable!("Named should have either short or long name"),
            (true, false) => Self::Long(named.long[0]),
            (false, true) => Self::Short(named.short[0]),
            (false, false) => Self::ShortLong(named.short[0], named.long[0]),
        }
    }
}

impl ShortLong {
    pub(crate) fn full_width(&self) -> usize {
        match self {
            ShortLong::Short(_) => 2,
            ShortLong::Long(l) | ShortLong::ShortLong(_, l) => 6 + l.len(),
        }
    }
}

impl std::fmt::Display for ShortLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                ShortLong::Short(short) => write!(f, "-{}", short),
                ShortLong::Long(long) => write!(f, "    --{}", long),
                ShortLong::ShortLong(short, long) => write!(f, "-{}, --{}", short, long),
            }
        } else {
            match self {
                ShortLong::Short(short) | ShortLong::ShortLong(short, _) => write!(f, "-{}", short),
                ShortLong::Long(long) => write!(f, "--{}", long),
            }
        }
    }
}

/// {} renders a version for short usage string
/// {:#} renders a full width version for --help body and complete, this version
/// supports padding of the help by some max width
impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Positional { metavar, help: _ } => write!(f, "<{}>", metavar),
            Item::Command { .. } => write!(f, "COMMAND ..."),
            Item::Flag { name, help: _ } => write!(f, "{}", name),
            Item::Argument {
                name,
                metavar,
                help: _,
                env: _,
            } => write!(f, "{} {}", name, metavar),
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

    #[must_use]
    pub(crate) fn is_command(&self) -> bool {
        match self {
            Item::Command { .. } => true,
            Item::Positional { .. } | Item::Flag { .. } | Item::Argument { .. } => false,
        }
    }

    #[must_use]
    pub(crate) fn is_flag(&self) -> bool {
        match self {
            Item::Positional { .. } | Item::Command { .. } => false,
            Item::Flag { .. } | Item::Argument { .. } => true,
        }
    }

    #[must_use]
    pub(crate) fn is_positional(&self) -> bool {
        match self {
            Item::Positional { help, .. } => help.is_some(),
            Item::Command { .. } | Item::Flag { .. } | Item::Argument { .. } => false,
        }
    }
}
