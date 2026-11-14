use crate::{info::Info, meta_help::Metavar, parsers::NamedArg, Doc, Meta};

/// A single thing a parser consumes from the command line
///
/// Leaves of the [`Meta`] tree. Everything a renderer needs to describe one flag, argument,
/// positional or command is here: the names it answers to, the metavariable to show for its
/// value, the environment variable it falls back to and the help text attached to it.
#[derive(Clone, Debug)]
pub enum Item {
    /// Free form item created with [`any`](crate::any)
    Any {
        /// Rendered in place of a name, since `any` has none
        metavar: Doc,
        /// used by any, moves it from positionals into arguments
        anywhere: bool,
        /// Help message attached with [`help`](crate::parsers::ParseAny::help)
        help: Option<Doc>,
    },
    /// Positional item, consumed from the the front of the arguments
    /// `<FILE>`
    Positional {
        /// Placeholder name for the value, `FILE` in `<FILE>`
        metavar: Metavar,
        /// Help message attached with [`help`](crate::parsers::ParsePositional::help)
        help: Option<Doc>,
    },
    /// Subcommand with a parser of its own
    Command {
        /// Name the subcommand is invoked by
        name: &'static str,
        /// Single character alias, if any
        short: Option<char>,
        /// Single line summary shown in the parent's help message
        help: Option<Doc>,
        /// Shape of the subcommand's own parser, walk it to document nested commands
        meta: Box<Meta>,
        /// Description, header and footer of the subcommand's help message
        info: Box<Info>,
    },
    /// short or long name, consumed anywhere
    /// -f
    /// --file
    Flag {
        /// Names this flag answers to
        name: ShortLong,
        /// used for disambiguation
        shorts: Vec<char>,
        /// Environment variable consulted when the flag is absent
        env: Option<&'static str>,
        /// Help message attached with [`help`](crate::parsers::NamedArg::help)
        help: Option<Doc>,
    },
    /// Short or long name followed by a value, consumed anywhere
    /// `-f <VAL>`
    /// `--file <VAL>`
    Argument {
        /// Names this argument answers to
        name: ShortLong,
        /// used for disambiguation
        shorts: Vec<char>,
        /// Placeholder name for the value, `VAL` in `--file <VAL>`
        metavar: Metavar,
        /// Environment variable consulted when the argument is absent
        env: Option<&'static str>,
        /// Help message attached with [`help`](crate::parsers::NamedArg::help)
        help: Option<Doc>,
    },
}

impl Item {
    pub(crate) fn is_pos(&self) -> bool {
        match self {
            Item::Any { anywhere, .. } => !anywhere,
            Item::Positional { .. } | Item::Command { .. } => true,
            Item::Flag { .. } | Item::Argument { .. } => false,
        }
    }
    /// Normalize name inside [`ShortLong`] into either short or long
    pub(crate) fn normalize(&mut self, short: bool) {
        match self {
            Item::Positional { .. } | Item::Command { .. } | Item::Any { .. } => {}
            Item::Flag { name, .. } | Item::Argument { name, .. } => name.normalize(short),
        }
    }
}

/// Names a flag or an argument answers to
///
/// Names are stored without their dashes, `Long("file")` is written `--file` on a command line.
#[derive(Copy, Clone, Debug)]
pub enum ShortLong {
    /// Only a short name, `-f`
    Short(char),
    /// Only a long name, `--file`
    Long(&'static str),
    /// Both, with the short name acting as an alias for the long one
    Both(char, &'static str),
}

impl ShortLong {
    pub(crate) fn as_long(&self) -> Option<&'static str> {
        match self {
            ShortLong::Long(l) | ShortLong::Both(_, l) => Some(l),
            ShortLong::Short(_) => None,
        }
    }
    pub(crate) fn as_short(&self) -> Option<char> {
        match self {
            ShortLong::Short(s) | ShortLong::Both(s, _) => Some(*s),
            ShortLong::Long(_) => None,
        }
    }
}

impl PartialEq<&str> for ShortLong {
    fn eq(&self, other: &&str) -> bool {
        fn short_eq(c: char, s: &str) -> bool {
            let mut tmp = [0u8; 4];
            s.strip_prefix('-') == Some(c.encode_utf8(&mut tmp))
        }
        fn long_eq(l: &str, s: &str) -> bool {
            Some(l) == s.strip_prefix("--")
        }
        match self {
            ShortLong::Short(s) => short_eq(*s, other),
            ShortLong::Long(l) => long_eq(l, other),
            ShortLong::Both(s, l) => short_eq(*s, other) || long_eq(l, other),
        }
    }
}

impl ShortLong {
    /// Changes [`ShortLong`](ShortLong::ShortLong) variant into either short or long depending,
    /// leaves both Short and Long untouched
    pub(crate) fn normalize(&mut self, short: bool) {
        match self {
            ShortLong::Short(_) | ShortLong::Long(_) => {}
            ShortLong::Both(s, l) => {
                if short {
                    *self = Self::Short(*s);
                } else {
                    *self = Self::Long(l);
                }
            }
        }
    }
}

impl TryFrom<&NamedArg> for ShortLong {
    type Error = ();

    fn try_from(named: &NamedArg) -> Result<Self, Self::Error> {
        match (named.short.is_empty(), named.long.is_empty()) {
            (true, true) => Err(()),
            (true, false) => Ok(Self::Long(named.long[0])),
            (false, true) => Ok(Self::Short(named.short[0])),
            (false, false) => Ok(Self::Both(named.short[0], named.long[0])),
        }
    }
}

impl Item {
    #[must_use]
    pub(crate) fn required(self, required: bool) -> Meta {
        let boxed = Meta::from(self);
        if required {
            boxed
        } else {
            Meta::Optional(Box::new(boxed))
        }
    }
}
