use crate::{Lit, Problem, Visited as _, Visitor, traits::VKind};

use super::*;

pub(crate) trait ProblemVisitor<'a>: Visitor<'a> {
    fn see_problem(&self) -> Option<Problem>;
}

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
}

impl<'a> ProblemVisitor<'a> for IsAcceptedOnce<'a> {
    fn see_problem(&self) -> Option<Problem> {
        match self.known {
            KnownName::No | KnownName::Many => None,
            KnownName::Single => Some(Problem::OnlyOnce {
                name: self.name.clone().into_owned(),
            }),
        }
    }
}

impl<'a> Visitor<'a> for IsAcceptedOnce<'a> {
    fn item(&mut self, item: Item<'a, '_>) {
        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Flag { named } | Item::Arg { named, .. } => {
                if named.names.contains(self.name) {
                    let known = if self.in_many > 0 {
                        KnownName::Many
                    } else {
                        KnownName::Single
                    };
                    self.known = self.known.max(known);
                }
            }
            Item::Nested { outer, .. } => outer.vi(self),
            Item::Positional { .. }
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
}

impl<'a> ProblemVisitor<'a> for BetterName<'a> {
    fn see_problem(&self) -> Option<Problem> {
        if self.distance < 0.4 {
            Some(Problem::DidYouName {
                target: Name::Long(std::borrow::Cow::Owned(self.target.to_string())),
                best: Name::Long(std::borrow::Cow::Owned(self.best.to_string())),
            })
        } else {
            None
        }
    }
}

impl<'a> Visitor<'a> for BetterName<'a> {
    fn item(&mut self, item: Item<'a, '_>) {
        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Flag { named } | Item::Arg { named, .. } => {
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
            Item::Nested { outer, .. } => outer.vi(self),
            Item::Section { .. }
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
    fn item(&mut self, item: Item<'a, '_>) {
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
}

impl<'a> ProblemVisitor<'a> for ValidCommand<'a> {
    fn see_problem(&self) -> Option<Problem> {
        if self.distance < 0.4 {
            Some(Problem::DidYouMeanLit {
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
}

impl<'a> ProblemVisitor<'a> for IsDDash {
    fn see_problem(&self) -> Option<Problem> {
        self.exists.then_some(Problem::TryDDash {
            name: self.name.clone(),
        })
    }
}

impl Visitor<'_> for IsDDash {
    fn item(&mut self, item: Item<'_, '_>) {
        if self.exists {
            return;
        }
        match item {
            Item::OptionParser { info: _, inner } => inner.vi(self),
            Item::Nested { outer, .. } => outer.vi(self),
            Item::Flag { named } | Item::Arg { named, .. } => {
                for name in &named.names {
                    if let Name::Long(actual) = name
                        && actual == &self.name
                    {
                        self.exists = true;
                    }
                }
            }
            Item::Positional { .. }
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

/// Given a named item, check if there's a command or nested parser at the
/// current level that accepts it, and remember the name.
pub(crate) struct IsInCommand<'a> {
    /// name we are looking for
    target: &'a Name<'a>,
    /// A candidate, if we found it
    candidate: Option<Candidate<'a>>,
    current_ctx: Option<Candidate<'a>>,
}

#[derive(Clone, Copy)]
enum Candidate<'a> {
    Command(&'a Lit<'static>),
    NamedFlag(&'a Name<'static>),
    Keyword(&'a Lit<'static>),
}

impl<'a> IsInCommand<'a> {
    pub(crate) fn new(target: &'a Name<'a>) -> Self {
        Self {
            target,
            candidate: None,
            current_ctx: None,
        }
    }
}

impl<'a> ProblemVisitor<'a> for IsInCommand<'a> {
    fn see_problem(&self) -> Option<Problem> {
        let candidate = self.candidate?;
        let name = self.target.clone().into_owned();
        Some(match candidate {
            Candidate::Command(cmd) => Problem::TryInCommand {
                cmd: cmd.clone(),
                name,
            },
            Candidate::NamedFlag(n) => Problem::TryInNested {
                outer: n.to_string(),
                name,
            },
            Candidate::Keyword(l) => Problem::TryInNested {
                outer: l.to_string(),
                name,
            },
        })
    }
}

/// Collect information about nested label for "is in command" visitor
struct NestedCandidate<'a> {
    ctx: Option<Candidate<'a>>,
}

impl<'a> Visitor<'a> for NestedCandidate<'a> {
    fn item<'t>(&mut self, item: Item<'a, 't>) {
        self.ctx = match item {
            Item::Flag { named } => named.names.first().map(Candidate::NamedFlag),
            Item::Command { names, .. } => names.first().map(Candidate::Keyword),
            _ => return,
        }
    }

    fn identify(&self) -> VKind {
        VKind::Error
    }

    fn push_group(&mut self, _: VisitGroup) {}

    fn pop_group(&mut self) {}
}

impl<'a> Visitor<'a> for IsInCommand<'a> {
    fn item(&mut self, item: Item<'a, '_>) {
        if self.candidate.is_some() {
            return;
        }
        match item {
            Item::OptionParser { inner, .. } => inner.vi(self),
            Item::Command { names, inner, .. } => {
                if self.current_ctx.is_none() {
                    self.current_ctx = names.first().map(Candidate::Command);
                    inner.vi(self);
                    self.current_ctx = None;
                }
            }
            Item::Nested { outer, inner } => {
                if self.current_ctx.is_none() {
                    let mut v = NestedCandidate { ctx: None };
                    outer.vi(&mut v);
                    if v.ctx.is_some() {
                        self.current_ctx = v.ctx;
                        inner.vi(self);
                        self.current_ctx = None;
                    }
                } else {
                    inner.vi(self);
                }
            }
            Item::Flag { named } | Item::Arg { named, .. } => {
                if !named.names.contains(self.target) {
                    return;
                }
                if let Some(ctx) = self.current_ctx {
                    self.candidate = Some(ctx);
                }
            }
            Item::Positional { .. } | Item::Section { .. } | Item::Rendered { .. } => {}
        }
    }

    fn push_group(&mut self, _: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> crate::VKind {
        VKind::Error
    }
}

impl<'a> Visitor<'a> for Vec<&mut dyn ProblemVisitor<'a>> {
    fn item(&mut self, item: Item<'a, '_>) {
        for v in self.iter_mut() {
            v.item(item);
        }
    }
    fn push_group(&mut self, group: VisitGroup) {
        for v in self.iter_mut() {
            v.push_group(group);
        }
    }
    fn pop_group(&mut self) {
        for v in self.iter_mut() {
            v.pop_group();
        }
    }
    fn identify(&self) -> VKind {
        VKind::Error
    }
}
