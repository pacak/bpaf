//! # Pecking order
//!
//! For as long as there's only one task to wake up for the input - it is safe to just
//! wake it up and be done with it, but users are allowed to specify multiple consumers for the
//! same name as well as multiple positional consumers that don't have names at all. This requires
//! deciding which parser gets to run first or gets to run at all.
//!
//! Rules for priority are:
//!
//! - sum branches run in parallel, left most wins if there's multiple successes
//! - parsers inside a product run sequentially, left most wins
//!
//! Therefore we are going to arrange tasks in following order:
//! There's one queue for each branch_id (sum parent id + field), every queue contains
//! items from the same product, so their priority is how far from the left end they are
//!
//! In practice all we need is a single BTreeSet :)
//!
//! "any" parsers get to run for both named and positional input inside their branch
//! accoding to their priority, if at the front. Consider a few queues
//! - `[named, any]` - `any` doesn't run since `named` takes priority
//! - `[any1, named, any2]` - `any1` runs, if it fails to match anything - `named` runs.
//! - `[any1, any2, named]` - `any1` runs, if not - `any2`, if not - `named`
//!
//! "any" are mixed with positional items the same way so we'll have to mix them in dynamically...

use crate::executor::{BranchId, Id};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub(crate) struct Pecking(BTreeSet<(BranchId, Id)>);

pub(crate) struct PeckingIter<'a> {
    order: &'a BTreeSet<(BranchId, Id)>,
    prev_branch: Option<BranchId>,
}

impl Iterator for PeckingIter<'_> {
    type Item = (BranchId, Id);

    fn next(&mut self) -> Option<Self::Item> {
        let (branch, id) = match self.prev_branch {
            Some(branch) => self
                .order
                .range((branch.succ(), Id::ZERO)..)
                .next()
                .copied()?,
            None => self.order.first().copied()?,
        };

        self.prev_branch = Some(branch);
        Some((branch, id))
    }
}

impl Pecking {
    /// Iterate over heads of all the queues
    pub(crate) fn heads(&self) -> PeckingIter<'_> {
        PeckingIter {
            order: &self.0,
            prev_branch: None,
        }
    }

    /// Iterate over all the items in all the queues,
    pub(crate) fn iter(&self) -> std::collections::btree_set::Iter<(BranchId, Id)> {
        self.0.iter()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn remove(&mut self, branch: BranchId, id: Id) {
        self.0.remove(&(branch, id));
    }

    pub(crate) fn insert(&mut self, branch: BranchId, id: Id) {
        self.0.insert((branch, id));
    }
}

#[test]
fn it_works() {
    let b1 = BranchId {
        parent: Id::ZERO,
        field: 0,
    };

    let b2 = BranchId {
        parent: Id::ZERO,
        field: 1,
    };

    let mut p = Pecking::default();

    p.insert(b1, Id::new(1));
    p.insert(b1, Id::new(2));
    p.insert(b1, Id::new(3));
    p.insert(b2, Id::new(4));
    p.insert(b2, Id::new(5));
    p.insert(b2, Id::new(6));

    let xs = p.heads().collect::<Vec<_>>();

    assert_eq!(xs, &[(b1, Id::new(1)), (b2, Id::new(4))]);
}
