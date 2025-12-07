use core::f32;
use std::{borrow::Cow, ffi::OsStr};

use crate::{
    Item, Name, Problem, Visitor, arg::Adjacency, traits::Group, utils::damerau_levenshtein,
};

macro_rules! visit_tuple  {
    ($( $class:ident $field:tt );+) => {
        impl <'a, $( $class: Visitor<'a>),+ > Visitor<'a> for ($($class),+) {
            fn item(&mut self, item: Item<'a>) { $( self.$field.item(item); )+ }
            fn push_group(&mut self, group: Group) { $( self.$field.push_group(group); )+ }
            fn pop_group(&mut self) { $( self.$field .pop_group(); )+}
        }
    }
}

visit_tuple!(A 0 ;  B 1 ; C 2 );

impl<'a, A: Visitor<'a>> Visitor<'a> for Option<A> {
    fn item(&mut self, item: Item<'a>) {
        let Some(inner) = self else {
            return;
        };
        inner.item(item);
    }

    fn push_group(&mut self, group: Group) {
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
}

/// Check if parser accepts a certain named item only once
#[derive(Debug)]
pub(crate) struct IsAcceptedOnce<'a> {
    name: &'a Name<'a>,
    known: KnownName,
    stack: Vec<Group>,
    in_many: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum KnownName {
    No,
    Single,
    Many,
}

impl<'a> IsAcceptedOnce<'a> {
    pub(crate) fn new(name: &'a Name<'a>) -> Self {
        Self {
            name,
            known: KnownName::No,
            stack: Vec::new(),
            in_many: 0,
        }
    }
    pub(crate) fn into_problem(self) -> Option<Problem> {
        match self.known {
            KnownName::No | KnownName::Many => None,
            KnownName::Single => Some(Problem::OnlyOnce {
                name: self.name.clone().into_owned(),
            }),
        }
    }
}

impl<'a> Visitor<'a> for IsAcceptedOnce<'a> {
    fn item(&mut self, item: Item<'a>) {
        let names = match item {
            Item::Flag { names, .. } | Item::Arg { names, .. } | Item::Nested { names, .. } => {
                names
            }
            Item::Positional { .. } | Item::Command { .. } => return,
        };
        if names.contains(self.name) {
            let known = if self.in_many > 0 {
                KnownName::Many
            } else {
                KnownName::Single
            };
            self.known = self.known.max(known);
        }
    }

    fn push_group(&mut self, group: Group) {
        self.stack.push(group);
        if matches!(group, Group::Many) {
            self.in_many += 1;
        }
    }

    fn pop_group(&mut self) {
        let group = self.stack.pop().unwrap();
        if matches!(group, Group::Many) {
            self.in_many -= 1;
        }
    }
}

pub(crate) struct BetterName<'a> {
    target: &'a str,
    best: &'a str,
    distance: f32,
}

impl<'a> BetterName<'a> {
    pub(crate) fn new(target: &'a str) -> Self {
        Self {
            target,
            distance: f32::INFINITY,
            best: "",
        }
    }
    pub(crate) fn into_problem(self) -> Option<Problem> {
        if self.distance < 0.4 {
            Some(Problem::DidYouMean {
                target: Name::from(self.target).into_owned(),
                best: Name::from(self.best).into_owned(),
            })
        } else {
            None
        }
    }
}

impl<'a> Visitor<'a> for BetterName<'a> {
    fn item(&mut self, item: Item<'a>) {
        let (Item::Flag { names, .. } | Item::Arg { names, .. } | Item::Nested { names, .. }) =
            item
        else {
            return;
        };
        for name in names {
            let Name::Long(long_possible) = name else {
                continue;
            };
            let dist = damerau_levenshtein(self.target, long_possible);
            if self.distance > dist {
                self.distance = dist;
                self.best = long_possible;
            }
        }
    }

    fn push_group(&mut self, _group: Group) {}
    fn pop_group(&mut self) {}
}

pub(crate) struct ValidCommand<'a> {
    target: &'a str,
    distance: f32,
    best: &'a str,
}

impl<'a> Visitor<'a> for ValidCommand<'a> {
    fn item(&mut self, item: Item<'a>) {
        let Item::Command { names, .. } = item else {
            return;
        };
        for name in names {
            let dist = damerau_levenshtein(self.target, name);
            if self.distance > dist {
                self.distance = dist;
                self.best = name;
            }
        }
    }

    fn push_group(&mut self, _group: Group) {}
    fn pop_group(&mut self) {}
}

impl<'a> ValidCommand<'a> {
    pub(crate) fn new(target: &'a str) -> Self {
        Self {
            target,
            distance: f32::INFINITY,
            best: "",
        }
    }
    pub(crate) fn into_problem(self) -> Option<Problem> {
        if self.distance < 0.4 {
            Some(Problem::DidYouMeanCmd {
                target: self.target.to_owned(),
                best: self.best.to_owned(),
            })
        } else {
            None
        }
    }
}

pub(crate) struct IsDDash {
    name: String,
    exists: bool,
}

impl IsDDash {
    pub(crate) fn attempt(
        unexpected: &Name<'_>,
        value: Option<&(Adjacency, Cow<'_, OsStr>)>,
    ) -> Option<Self> {
        let (adj, value) = value?;
        let (Name::Short(prefix), Adjacency::Immediate) = (unexpected, adj) else {
            return None;
        };
        let suffix = value.to_str()?;
        let mut name = String::with_capacity(suffix.len() + 4);
        name.push(*prefix);
        name.push_str(suffix);

        Some(Self {
            name,
            exists: false,
        })
    }
    pub(crate) fn into_problem(self) -> Option<Problem> {
        self.exists.then_some(Problem::TryDDash { name: self.name })
    }
}

impl Visitor<'_> for IsDDash {
    fn item(&mut self, item: Item<'_>) {
        if self.exists {
            return;
        }
        let (Item::Flag { names, .. } | Item::Arg { names, .. } | Item::Nested { names, .. }) =
            item
        else {
            return;
        };

        for name in names {
            if let Name::Long(actual) = name {
                if actual == &self.name {
                    self.exists = true;
                }
            }
        }
    }

    fn push_group(&mut self, group: Group) {}

    fn pop_group(&mut self) {}
}
