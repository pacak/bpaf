pub(crate) mod errors;
pub(crate) mod help;
pub(crate) mod usage;

use crate::{
    Item, Name, Named, Visitor, arg::Adjacency, traits::VisitGroup, utils::damerau_levenshtein,
};
use std::ffi::OsStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ShortLong<'a> {
    Short(char),
    Long(&'a str),
    Both(char, &'a str),
}

impl ShortLong<'_> {
    pub(crate) fn col_width(&self) -> usize {
        4 + match self {
            // `-f`
            ShortLong::Short(_) => 2,
            // `-f, --foo` or `    --foo`
            ShortLong::Both(_, l) | ShortLong::Long(l) => 6 + crate::console_writer::char_width(l),
        }
    }
}

impl Named {
    fn get_shortlong<'a>(&'a self) -> Option<ShortLong<'a>> {
        match self.get_short_and_long() {
            (None, None) => None,
            (None, Some(l)) => Some(ShortLong::Long(l)),
            (Some(s), None) => Some(ShortLong::Short(s)),
            (Some(s), Some(l)) => Some(ShortLong::Both(s, l)),
        }
    }
}
