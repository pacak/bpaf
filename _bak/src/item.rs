use crate::{info::Info, meta_help::Metavar, parsers::NamedArg, Doc, Meta};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Item {
    Any {
        metavar: Doc,
        /// used by any, moves it from positionals into arguments
        anywhere: bool,
        help: Option<Doc>,
    },
    /// Positional item, consumed from the the front of the arguments
    /// <FILE>
    Positional { metavar: Metavar, help: Option<Doc> },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<Doc>,
        meta: Box<Meta>,
        info: Box<Info>,
    },
    /// short or long name, consumed anywhere
    /// -f
    /// --file
    Flag {
        name: ShortLong,
        /// used for disambiguation
        shorts: Vec<char>,
        env: Option<&'static str>,
        help: Option<Doc>,
    },
    /// Short or long name followed by a value, consumed anywhere
    /// -f <VAL>
    /// --file <VAL>
    Argument {
        name: ShortLong,
        /// used for disambiguation
        shorts: Vec<char>,
        metavar: Metavar,
        env: Option<&'static str>,
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

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub enum ShortLong {
    Short(char),
    Long(&'static str),
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
