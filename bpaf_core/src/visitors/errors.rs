use crate::{Flag, Nest, visitors::VKind};

use super::*;
/// Check if parser accepts a certain named item only once
#[derive(Debug)]
pub(crate) struct IsAcceptedOnce<'a> {
    name: &'a Name<'a>,
    known: KnownName,
    stack: Vec<VisitGroup>,
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
        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Flag { named }
            | Item::Arg { named, .. }
            | Item::Nested {
                outer: Nest::Named(Flag { named, .. }),
                ..
            } => {
                if named.names.contains(self.name) {
                    let known = if self.in_many > 0 {
                        KnownName::Many
                    } else {
                        KnownName::Single
                    };
                    self.known = self.known.max(known);
                }
            }
            Item::Nested { .. }
            | Item::Positional { .. }
            | Item::Command { .. }
            | Item::Section { .. }
            | Item::Rendered { .. } => {}
        }
    }

    fn push_group(&mut self, group: VisitGroup) {
        self.stack.push(group);
        if matches!(group, VisitGroup::Many) {
            self.in_many += 1;
        }
    }

    fn pop_group(&mut self) {
        let group = self.stack.pop().unwrap();
        if matches!(group, VisitGroup::Many) {
            self.in_many -= 1;
        }
    }

    fn identify(&self) -> crate::VKind {
        VKind::Error
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
        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Flag { named }
            | Item::Arg { named, .. }
            | Item::Nested {
                outer: Nest::Named(Flag { named, .. }),
                ..
            } => {
                for name in &named.names {
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
            Item::Nested { .. }
            | Item::Section { .. }
            | Item::Rendered { .. }
            | Item::Positional { .. }
            | Item::Command { .. } => {}
        }
    }

    fn push_group(&mut self, _group: VisitGroup) {}
    fn pop_group(&mut self) {}

    fn identify(&self) -> crate::VKind {
        VKind::Error
    }
}

pub(crate) struct ValidCommand<'a> {
    target: &'a str,
    distance: f32,
    best: &'a str,
}

impl<'a> Visitor<'a> for ValidCommand<'a> {
    fn item(&mut self, item: Item<'a>) {
        match item {
            Item::OptionParser { inner, .. } => inner.vi(self),
            Item::Command { names, .. } => {
                for name in names {
                    let Name::Long(name) = &name.0 else {
                        continue;
                    };
                    let dist = damerau_levenshtein(self.target, name);
                    if self.distance > dist {
                        self.distance = dist;
                        self.best = name;
                    }
                }
            }
            Item::Flag { .. }
            | Item::Arg { .. }
            | Item::Positional { .. }
            | Item::Nested { .. }
            | Item::Section { .. }
            | Item::Rendered { .. } => {}
        }
    }

    fn push_group(&mut self, _group: VisitGroup) {}
    fn pop_group(&mut self) {}

    fn identify(&self) -> crate::VKind {
        VKind::Error
    }
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
        value: Option<&(Adjacency, &OsStr)>,
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

        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Flag { named }
            | Item::Arg { named, .. }
            | Item::Nested {
                outer: Nest::Named(Flag { named, .. }),
                ..
            } => {
                for name in &named.names {
                    if let Name::Long(actual) = name
                        && actual == &self.name
                    {
                        self.exists = true;
                    }
                }
            }
            Item::Nested { .. }
            | Item::Positional { .. }
            | Item::Command { .. }
            | Item::Section { .. }
            | Item::Rendered { .. } => {}
        }
    }

    fn push_group(&mut self, _: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> crate::VKind {
        VKind::Error
    }
}
