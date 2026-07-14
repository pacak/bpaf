//! # Pecking order - what parsers get to run
//!
//! Crate permits arbitrary algebraic trees for parsers composed of sums (enums) and
//! products (structs), including multiple parsers matching the same trigger -
//! name or position.
//!
//! At the same time we must be careful about parsing:
//! 1. In the final result each CLI gets consumed by exactly one parser
//! 2. All the CLI items are consumed.
//!
//! During the execution we might consume the same item from multiple parsers - as long as they
//! go into parallel branches of a sum and gets disambiguated later.
//!
//! Rules for running as follows:
//!
//! 1. If a parser from a product runs - no other parser from the same product can consume this
//!    item - all the items from this product go into the final result
//! 2. Parsers from the same sum can run in parallel
//!
//! Some examples:
//!
//! - `A + B` - Both can run
//! - `A * B` - Only `A` can run
//! - `(A + B) + C` - Run all 3
//! - `(A + B) * (C + D)` - Run `A` and `B`
//! - `(A * B) + (C * D)` - Run `A` and `C`

use crate::{Id, Kind, Parent};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Default)]
pub(crate) struct PeckingOrder(Pecking);

#[derive(Debug, Default)]
enum Pecking {
    #[default]
    Empty,
    Single {
        id: Id,
        parent: Parent,
        kind: Kind,
    },
    Set {
        set: BTreeMap<Id, (Parent, Kind)>,
    },
}

impl PeckingOrder {
    pub(crate) fn insert(&mut self, parent: Parent, id: Id, kind: Kind) {
        match &mut self.0 {
            Pecking::Empty => self.0 = Pecking::Single { id, parent, kind },
            Pecking::Single {
                id: pid,
                parent: pparent,
                kind: pkind,
            } => {
                let mut set = BTreeMap::new();
                set.insert(*pid, (*pparent, *pkind));
                set.insert(id, (parent, kind));
                self.0 = Pecking::Set { set };
            }
            Pecking::Set { set } => {
                set.insert(id, (parent, kind));
            }
        }
    }

    pub(crate) fn first(&self) -> Option<Id> {
        match &self.0 {
            Pecking::Empty => None,
            Pecking::Single { id, .. } => Some(*id),
            Pecking::Set { set } => Some(*set.first_key_value()?.0),
        }
    }

    /// Remove an item from the pecking order, returns `true` if the order is now empty
    pub(crate) fn remove(&mut self, id: Id) -> bool {
        let ok = match &mut self.0 {
            Pecking::Empty => false,
            Pecking::Single { id: old, .. } => {
                let ok = id == *old;
                self.0 = Pecking::Empty;
                ok
            }
            Pecking::Set { set } => set.remove(&id).is_some(),
        };
        debug_assert!(ok, "Couldn't remove absent {id:?} from {self:?}!");
        match &self.0 {
            Pecking::Empty => true,
            Pecking::Single { .. } => false,
            Pecking::Set { set } => set.is_empty(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Mixer {
    candidates: VecDeque<Id>,
    seen: BTreeSet<Id>,
    out: Vec<Id>,
}

impl Mixer {
    pub(crate) fn push(&mut self, id: Id) {
        self.candidates.push_back(id);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub(crate) fn push_peck(&mut self, order: &PeckingOrder) {
        match &order.0 {
            Pecking::Empty => {}
            Pecking::Single { id, .. } => self.candidates.push_back(*id),
            Pecking::Set { set } => {
                let mut prev = Parent(0);
                for (id, (parent, kind)) in set.iter() {
                    if *parent != prev || *kind != Kind::Prod {
                        self.candidates.push_back(*id);
                        prev = *parent;
                    }
                }
            }
        }
    }

    /// Goes though all the candidates and returns those that should be executed
    pub(crate) fn for_wake(&mut self, tasks: &crate::Tasks) -> std::vec::Drain<'_, Id> {
        self.candidates.make_contiguous().sort();
        'outer: while let Some(mut id) = self.candidates.pop_front() {
            let candidate = id;
            'inner: while id != Id(0) {
                let task = &tasks[id];
                id = task.info.parent_id.as_id();
                if !self.seen.insert(id) {
                    // false - seen already
                    if task.info.parent_kind == crate::Kind::Prod {
                        continue 'outer;
                    } else {
                        break 'inner;
                    }
                }
            }
            self.out.push(candidate);
        }
        self.candidates.clear();
        self.seen.clear();
        self.out.drain(..)
    }

    pub(crate) fn clear(&mut self) {
        self.candidates.clear();
        self.out.clear();
        self.seen.clear();
    }
}
