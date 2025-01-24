//! This visitor tries to explain unparsed items passed by user but not consumed by the parser
//!
//! Visitor makes best effort to explain

use crate::{
    error::{Error, Message, MissingItem},
    mini_ansi::{Emphasis, Invalid},
    named::Name,
    split::{split_param, Arg, OsOrStr},
    visitor::{Group, Item, Mode, Visitor},
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

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
    // Can't use HashMap to keep results deterministic
    all_names: BTreeMap<Name<'a>, NameEntry>,
    branch_id: u32,
    in_many: u32,

    current_command: Option<Name<'a>>,
    good_command: Option<Name<'a>>,

    stack: Vec<Group>,
    unparsed_raw: Option<&'a str>,
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
        unparsed: Arg<'a>,
        unparsed_raw: Option<&'a str>,
        parsed: &'a [OsOrStr<'a>],
    ) -> Self {
        Self {
            unparsed,
            unparsed_raw,
            parsed,
            missing,
            in_many: 0,
            stack: Default::default(),
            branch_id: 0,
            all_names: Default::default(),

            current_command: None,
            good_command: None,
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
            // two named items can't be used at once
            if let Some(err) = self.is_in_conflict(&parsed, name.as_ref()) {
                return err;
            }
            // a single named items can be used only once
            if let Some(err) = self.is_redundant(&parsed, name.as_ref()) {
                return err;
            }
        }

        // not supported directly, but supported by a subcommand
        if let Some(good) = self.good_command {
            return Error::try_subcommand(self.unparsed.to_owned(), good.to_owned());
        }

        if let Arg::Named { name, value } = &self.unparsed {
            if let Some(err) = self.is_typo(name, value.as_ref()) {
                return err;
            }
        }

        // expect positional item or items, got named one
        if let Some([MissingItem::Positional { meta }, rest @ ..]) = self.missing.as_deref() {
            if matches!(self.unparsed, Arg::Named { .. })
                && rest
                    .iter()
                    .all(|i| matches!(i, MissingItem::Positional { .. }))
            {
                return Error::try_positional(self.unparsed.to_owned(), *meta);
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
                let unparsed = self.unparsed.to_owned();
                Error::unexpected(unparsed)
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
            let name = unparsed.to_owned();
            return Some(Error::new(Message::OnlyOnce { name }));
        }

        None
    }

    fn is_typo(&self, name: &Name, value: Option<&OsOrStr>) -> Option<Error> {
        match name {
            Name::Short(s) => {
                if let Some(input) = self.unparsed_raw {
                    if let Some(long) = input.strip_prefix('-') {
                        let long = Name::Long(Cow::Borrowed(long));
                        if self.all_names.contains_key(&long) {
                            return Some(Error::new(Message::TryDoubleDash {
                                input: Invalid(input.to_owned()),
                                long: Emphasis(long.to_owned()),
                            }));
                        }
                    }
                }

                self.is_typo_in_short(*s)
            }

            Name::Long(cow) => self.is_typo_in_long(cow),
        }
    }

    fn is_typo_in_short(&self, name: char) -> Option<Error> {
        None
    }

    fn is_typo_in_long(&self, name: &str) -> Option<Error> {
        let mut best = None;
        let mut best_distance = usize::MAX;
        let name_len = name.chars().count();
        if name_len == 1 {
            let short = Name::Short(name.chars().next()?); // we checked - this is one char long name
            for candidate in self.all_names.keys() {
                if short == candidate.as_ref() {
                    return Some(Error::new(Message::TrySingleDash {
                        input: Invalid(Name::long(name).to_owned()),
                        short: Emphasis(candidate.to_owned()),
                    }));
                }
            }
        }

        for candidate in self.all_names.keys() {
            if let Name::Long(cow) = candidate {
                let this = damerau_levenshtein(name, cow);
                if this < best_distance {
                    best_distance = this;
                    best = Some(candidate);
                }
            }
        }

        if name_len / 2 > best_distance {
            return Some(Error::new(Message::TryTypo {
                input: Invalid(Name::long(name).to_owned()),
                long: Emphasis(best?.to_owned()),
            }));
        }

        None
    }
}

pub(crate) fn first_good_name<'a>(names: &'a [Name<'a>]) -> Option<Name<'a>> {
    names.first().map(|n| n.as_ref())
}

impl<'a> ExplainUnparsed<'a> {
    fn unparsed_arg_name(&'a self) -> Option<Name<'a>> {
        match &self.unparsed {
            Arg::Named { name, .. } => Some(name.as_ref()),
            _ => None,
        }
    }
}

impl<'a> Visitor<'a> for ExplainUnparsed<'a> {
    fn item(&mut self, item: Item<'a>) {
        self.advance_branch_id();

        match item {
            Item::Flag { names, .. } | Item::Arg { names, .. } => {
                for name in names {
                    if self.current_command.is_some()
                        && Some(name.as_ref()) == self.unparsed_arg_name()
                    {
                        self.good_command = self.current_command.clone();
                    }

                    let e = self.all_names.entry(name.as_ref()).or_default();
                    e.in_many |= self.in_many > 0;
                    e.count += 1;
                    e.branch = self.branch_id;
                }
            }
            Item::Positional { .. } => {}
        }
    }

    fn command(&mut self, x: &'a [Name<'a>]) -> bool {
        self.advance_branch_id();
        if self.unparsed_arg_name().is_some() && self.current_command.is_none() {
            self.current_command = first_good_name(x);
            self.current_command.is_some()
        } else {
            false
        }
    }

    fn push_group(&mut self, group: Group) {
        self.advance_branch_id();
        if group == Group::Many {
            self.in_many += 1
        }
        self.stack.push(group);
    }

    fn pop_group(&mut self) {
        let last = self.stack.pop();
        if last == Some(Group::Many) {
            self.in_many -= 1;
        } else if last == Some(Group::Subparser) {
            self.current_command = None;
        }
    }

    fn mode(&self) -> Mode {
        Mode::Info
    }
}

/// Damerau-Levenshtein distance function
#[inline(never)]
fn damerau_levenshtein(a: &str, b: &str) -> usize {
    #![allow(clippy::many_single_char_names)]
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    let buf_size = (a_len + 1) * (b_len + 1);
    if buf_size > 1_000_000 {
        // don't DoS on inputs that are too big
        return usize::MAX;
    }
    let mut d = vec![0; buf_size];

    let ix = |ib, ia| a_len * ia + ib;

    for i in 0..=a_len {
        d[ix(i, 0)] = i;
    }

    for j in 0..=b_len {
        d[ix(0, j)] = j;
    }

    let mut pa = '\0';
    let mut pb = '\0';
    for (i, ca) in a.chars().enumerate() {
        let i = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let j = j + 1;
            let cost = usize::from(ca != cb);
            d[ix(i, j)] = (d[ix(i - 1, j)] + 1)
                .min(d[ix(i, j - 1)] + 1)
                .min(d[ix(i - 1, j - 1)] + cost);
            if i > 1 && j > 1 && ca == pb && cb == pa {
                d[ix(i, j)] = d[ix(i, j)].min(d[ix(i - 2, j - 2)] + 1);
            }
            pb = cb;
        }
        pa = ca;
    }

    d[ix(a_len, b_len)]
}
