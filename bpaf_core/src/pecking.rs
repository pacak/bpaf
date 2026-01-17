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
//! Rules for running as follow:
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

use crate::{Id, Parent};
use std::collections::BTreeSet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Item {
    pub(crate) parent: Parent,
    pub(crate) id: Id,
}

impl Item {
    pub(crate) fn next_item(&self) -> Item {
        Item {
            id: Id(self.id.0 + 1),
            ..*self
        }
    }
    pub(crate) fn next_family_item(&self) -> Item {
        Item {
            parent: Parent(self.parent.0 + 1),
            id: Id(0),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PeckingOrder(Pecking);
impl PeckingOrder {
    pub(crate) fn is_empty(&self) -> bool {
        match &self.0 {
            Pecking::Empty => true,
            Pecking::Single { item: _ } => false,
            Pecking::Set { set } => set.is_empty(),
        }
    }
    pub(crate) fn peek(&self) -> Option<Item> {
        match &self.0 {
            Pecking::Empty => None,
            Pecking::Single { item } => Some(*item),
            Pecking::Set { set } => set.first().copied(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.0 = Pecking::Empty
    }
}

#[derive(Debug, Default)]
enum Pecking {
    #[default]
    Empty,
    Single {
        item: Item,
    },
    Set {
        set: BTreeSet<Item>,
    },
}

impl PeckingOrder {
    pub(crate) fn insert(&mut self, parent: Parent, id: Id) {
        let new_item = Item { parent, id };
        match &mut self.0 {
            Pecking::Empty => self.0 = Pecking::Single { item: new_item },
            Pecking::Single { item } => {
                let mut set = BTreeSet::new();
                set.insert(*item);
                set.insert(new_item);
                self.0 = Pecking::Set { set };
            }
            Pecking::Set { set } => {
                set.insert(new_item);
            }
        }
    }

    /// Remove an item from the pecking order, returns `false` if PO is now empty
    pub(crate) fn remove(&mut self, parent: Parent, id: Id) -> bool {
        let to_remove = Item { parent, id };
        let ok = match &mut self.0 {
            Pecking::Empty => false,
            Pecking::Single { item } => {
                let ok = *item == to_remove;
                self.0 = Pecking::Empty;
                ok
            }
            Pecking::Set { set } => set.remove(&to_remove),
        };
        debug_assert!(ok, "Couldn't remove absent {to_remove:?} from {self:?}!");
        match &self.0 {
            Pecking::Empty => true,
            Pecking::Single { .. } => false,
            Pecking::Set { set } => set.is_empty(),
        }
    }

    /// Get the next item that might or might not belong to this family
    pub(crate) fn item_after(&self, prev: Item) -> Option<&Item> {
        match &self.0 {
            Pecking::Empty => None,
            Pecking::Single { item } => (prev < *item).then_some(item),
            Pecking::Set { set } => set.range(prev.next_item()..).next(),
        }
    }
}
