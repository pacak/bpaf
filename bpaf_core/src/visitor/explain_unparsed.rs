//! This visitor tries to explain unparsed items passed by user but not consumed by the parser
//!
//! Visitor makes best effort to explain

use crate::{
    error::{Error, MissingItem},
    mini_ansi::{Emphasis, Invalid},
    named::Name,
    split::{split_param, Arg, OsOrStr},
    visitor::{Group, Item, Mode, Visitor},
};
use std::collections::{BTreeMap, HashMap};

/// Visitor that tries to explain why we couldn't parse a name
#[derive(Debug)]
pub(crate) struct ExplainUnparsed<'a> {
    // ============== this things are passed from the parser
    /// An item we couldn't parse
    unparsed: Arg<'a>,
    /// All the items we managed to parse succesfully
    parsed: &'a [OsOrStr<'a>],
    /// If parser failed with "there are missing items" - those are missing items
    missing: Option<Vec<MissingItem>>,

    // ============== inner state
    all_names: HashMap<Name<'a>, NameEntry>,
    branch_id: u32,
    in_many: u32,
    stack: Vec<Group>,
}

#[derive(Debug, Default)]
struct NameEntry {
    in_many: bool,
    branch: u32,
    count: u32,
}

impl<'a> ExplainUnparsed<'a> {
    pub(crate) fn new(
        missing: Option<Vec<MissingItem>>,
        arg: Arg<'a>,
        parsed: &'a [OsOrStr<'a>],
    ) -> Self {
        Self {
            unparsed: arg,
            parsed,
            missing,
            in_many: 0,
            stack: Default::default(),
            branch_id: 0,
            all_names: Default::default(),
        }
    }

    pub(crate) fn explain(self) -> Error {
        let m = HashMap::new();

        let parsed = self
            .parsed
            .iter()
            .flat_map(|sos| match split_param(sos, &m, &m).ok()? {
                Arg::Named { name, .. } => Some(name),
                Arg::ShortSet { .. } | Arg::Positional { .. } => None,
            })
            .collect::<Vec<_>>();

        if let Arg::Named { name, .. } = &self.unparsed {
            if let Some(err) = self.is_in_conflict(&parsed, name.as_ref()) {
                return err;
            }
            if let Some(err) = self.is_redundant(&parsed, name.as_ref()) {
                return err;
            }
        }

        // Suggestions I'd like to make

        // 1. two names cannot be used at once
        //    implemented in is_in_conflict
        //
        // 2. name can only be used once
        //    - name must be in parsed once and must not be wraped inside of `many
        //
        // 3. there's a typo in the name
        //    - user typed --f instead of -f
        //    - user typed -foo instead of foo (we'll see it as named arg Foo with name f and val "oo"
        //    - track all the names, look for shortest damerau levenshtein (copy/paste it from
        //      src/meta_youmean.rs)
        //
        // 4. not available in the main parser, but present in a sub parser in some way
        //    - go one level deep into subparsers (visitor for command returns bool asking
        //      if it should go in or skip it)
        //    - look for exact names only
        println!("Try improve error message with this {self:?}");
        match self.missing {
            Some(m) => Error {
                message: crate::error::Message::Missing(m),
            },
            None => {
                let unparsed = self.unparsed;
                Error::parse_fail(format!("Unexpected item {unparsed:?}"))
            }
        }
    }

    fn advance_branch_id(&mut self) {
        if self.stack.last().copied() == Some(Group::Sum) {
            self.branch_id += 1;
        }
    }

    fn is_in_conflict(&self, parsed: &[Name], unparsed: Name) -> Option<Error> {
        let unparsed_info = self.all_names.get(&unparsed)?;
        if unparsed_info.count > 1 || unparsed_info.in_many {
            return None;
        }
        for p in parsed {
            let Some(parsed) = self.all_names.get(p) else {
                continue;
            };
            if parsed.branch != unparsed_info.branch {
                return Some(Error {
                    message: crate::error::Message::Conflicts {
                        winner: Emphasis(p.to_owned()),
                        loser: Invalid(unparsed.to_owned()),
                    },
                });
            }
        }
        None
    }

    fn is_redundant(&self, parsed: &[Name], unparsed: Name) -> Option<Error> {
        let unparsed_info = self.all_names.get(&unparsed)?;
        if unparsed_info.in_many {
            return None;
        }

        if parsed.contains(&unparsed) {
            return Some(Error {
                message: crate::error::Message::OnlyOnce {
                    name: unparsed.to_owned(),
                },
            });
        }

        None
    }
}

impl<'a> Visitor<'a> for ExplainUnparsed<'a> {
    fn item(&mut self, item: Item<'a>) {
        self.advance_branch_id();

        match item {
            Item::Flag { names, .. } | Item::Arg { names, .. } => {
                for name in names {
                    let e = self.all_names.entry(name.as_ref()).or_default();
                    e.in_many |= self.in_many > 0;
                    e.count += 1;
                    e.branch = self.branch_id;
                }
            }
            Item::Positional { .. } => {}
        }
    }

    fn command(&mut self, _: &[Name]) -> bool {
        self.advance_branch_id();
        false
    }

    fn push_group(&mut self, group: Group) {
        self.advance_branch_id();
        if group == Group::Many {
            self.in_many += 1
        }
        self.stack.push(group);
    }

    fn pop_group(&mut self) {
        if self.stack.last().copied() == Some(Group::Many) {
            self.in_many -= 1;
        }
        self.stack.pop();
    }

    fn mode(&self) -> Mode {
        Mode::Info
    }
}
