use crate::{
    named::Name,
    split::{split_param, Arg, OsOrStr},
    visitor::{Group, Item, Visitor},
    Error,
};
use std::collections::BTreeMap;

/// Visitor that tries to explain why we couldn't parse a name
#[derive(Debug)]
pub(crate) struct UnparsedName<'a> {
    name: Name<'a>,
    parsed: &'a [OsOrStr<'a>],

    /// Each name is annotated with it's branch id and if it is contained in `many` in some way
    all_names: BTreeMap<Name<'a>, NameEntry>, // <(Name<'a>, usize, bool)>,
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

impl<'a> UnparsedName<'a> {
    pub(crate) fn new(name: Name<'a>, parsed: &'a [OsOrStr<'a>]) -> Self {
        Self {
            name,
            parsed,
            in_many: 0,
            stack: Default::default(),
            branch_id: 0,
            all_names: Default::default(),
        }
    }

    pub(crate) fn explain(&self) -> Error {
        let m = BTreeMap::new();

        let parsed = self
            .parsed
            .iter()
            .flat_map(|sos| match split_param(sos, &m, &m).ok()? {
                Arg::Named { name, .. } => Some(name),
                Arg::ShortSet { .. } | Arg::Positional { .. } => None,
            })
            .collect::<Vec<_>>();

        if let Some(err) = self.is_in_conflict(&parsed, self.name.as_ref()) {
            return err;
        }

        println!("{parsed:?}");
        // DO I care about

        // 1. two names cannot be used at once
        //
        //
        // 2. name can only be used once
        //    track if we are in `many`
        //    - if we are not in many and see the name
        //    - if we see the name only once
        //
        // 3. there's a typo in the name
        //    - user typed --f instead of -f
        //    - user typed -foo instead of foo
        //    - track all the names, look for shortest lev distance
        //
        // 4. not available in the main parser, but present in a sub parser in some way
        //    - go one level deep into subparsers
        //    - look for exact names
        todo!("{self:?}");
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
                        winner: p.to_owned(),
                        loser: unparsed.to_owned(),
                    },
                });
                //                println!("{unparsed:?} conflicts with {p:?}");
            }
        }
        None
    }
}

impl<'a> Visitor<'a> for UnparsedName<'a> {
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
}
