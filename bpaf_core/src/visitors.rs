pub(crate) mod errors;
pub(crate) mod help;
pub(crate) mod usage;

use crate::{
    Item, Name, Named, Problem, Visitor, arg::Adjacency, traits::VKind, traits::VisitGroup,
    utils::damerau_levenshtein,
};
use std::ffi::OsStr;

macro_rules! visit_tuple  {
    ($( $class:ident $field:tt );+) => {
        impl <'a, $( $class: Visitor<'a>),+ > Visitor<'a> for ($($class),+) {
            fn item(&mut self, item: Item<'a>) { $( self.$field.item(item.clone()); )+ }
            fn push_group(&mut self, group: VisitGroup) { $( self.$field.push_group(group); )+ }
            fn pop_group(&mut self) { $( self.$field.pop_group(); )+}
            fn identify(&self) -> VKind { VKind::Error }
        }
    }
}

visit_tuple!(A 0 ; B 1 ; C 2 );

impl<'a, A: Visitor<'a>> Visitor<'a> for Option<A> {
    fn item(&mut self, item: Item<'a>) {
        let Some(inner) = self else {
            return;
        };
        inner.item(item);
    }

    fn push_group(&mut self, group: VisitGroup) {
        let Some(inner) = self else {
            return;
        };
        inner.push_group(group);
    }

    fn pop_group(&mut self) {
        let Some(inner) = self else {
            return;
        };
        inner.pop_group();
    }

    fn identify(&self) -> VKind {
        match self {
            Some(inner) => inner.identify(),
            None => VKind::Error,
        }
    }
}

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
