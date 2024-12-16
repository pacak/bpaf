//! Relationship between different tasks
//!
//! Tasks are spawned and executed in prod/sum groups.
//!
//! For parallel case it is important to know mutual exclusivity, for sequential case - parser priority

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use crate::{
    executor::{split_param, Arg},
    named::Name,
};

use super::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct Id(u32);
impl Id {
    const ROOT: Self = Self(0);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum NodeKind {
    Sum,
    Prod,
}

impl Id {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
    pub(crate) fn sum(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Sum,
            id: self,
            field,
        }
    }

    pub(crate) fn prod(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Prod,
            id: self,
            field,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct Parent {
    pub(crate) kind: NodeKind,
    pub(crate) id: Id,
    pub(crate) field: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct BranchId {
    parent: Id,
    field: u32,
}

impl std::fmt::Debug for BranchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B({}:{})", self.parent.0, self.field)
    }
}

impl BranchId {
    pub(crate) const ROOT: Self = Self {
        parent: Id::ROOT,
        field: 0,
    };
    pub(crate) const DUMMY: Self = Self {
        parent: Id::ROOT,
        field: 0,
    };
}

#[derive(Debug)]
struct TaskInfo {
    /// Pointer to the parent item plus some info about it:
    /// - parent id
    /// - field number,
    /// - parent type (sum/prod)
    parent: Parent,

    /// Id field number of the nearest sum parent
    /// When consuming things in the same category (same name or
    /// several positional items) we want to know if those things
    /// are being consumed in parallel. `branch` holds id and field
    /// of the nearest sum parent.
    ///
    /// If they are the same - items consumed belong to the same prod
    /// type so must be consumed sequentially. If they are different -
    /// they belong to different branches and can be consumed concurrently
    ///
    /// Longest match wins though.
    branch: BranchId,
    // Problem: .many() will repeatedly spawn the inner parser until interrupted
    // by something. This interacts badly with parsers that consume the same object type in the
    // same parser... When parsers are positional and values are "a b c d", first call
    // for .many() consumes "a" and a new parser gets spawned, then call to the second parser
    // consumes "b" and only then first parser deals with "c" and "d", resulting in ([a, c, d], b).

    // I want to know the position of the item in the virtual product
    // for that I need to know parent offset as well as child on the left offset...
    //
    // (a*, b): a < b; [0] < [1]
    // ((a*, b), c): a < b < c: [0, 0] < [0, 1] < [1]
    // ((e, b, a*), c): e < b < a < c; [0, 0] < [0, 1] < [0, 2] < [1]
    // ((a, b), (c, d)): a < b < c < d; [0, 0] < [0, 1] < [1, 0] < [1, 1]
    //
    // 1. Easiest way is to build paths to the each id and sort them in
    // lexicographical order... But this is allocations.
    // We can treat it by sort of linked list
    // by keeping depth and field number. This way
    // we can recover the order by getting right/left item
    // to the same depth, if they match parent - compare field ids.
    // otherwise - keep going up until branch id...
    //
    // 2. Alternatively - try to reuse the same task ids and have queus as ordered sets...
    // easiest way to achieve this is by adding a boolean flag to spawn operation
    // asking to retain and reuse current next_task_id seem to work fine - requires some
    // shuffling though.
    //
    // 3. Finally - declare this a feature. By definition parsers are greedy and there's
    // no fallback so .positional().many() followed by .positional() can never match
    // anything, not until we start adding .take() or .range()...
    //
    //
}

// For any node I need to be able to find all sum siblings
// and order prod siblings in a pecking order
#[derive(Debug, Default)]
pub(crate) struct FamilyTree<'ctx> {
    flags: BTreeMap<Name<'ctx>, Pecking>,
    args: BTreeMap<Name<'ctx>, Pecking>,
    positional: Pecking,
    tasks: HashMap<Id, TaskInfo>,
}

impl<'ctx> FamilyTree<'ctx> {
    pub(crate) fn add_positional(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.positional.insert(id, branch);
    }

    pub(crate) fn remove_positional(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.positional.remove(branch, id);
    }

    pub(crate) fn add_named(&mut self, flag: bool, id: Id, names: &[Name<'static>]) {
        let branch = self.tasks.get(&id).unwrap().branch;
        let map = if flag {
            &mut self.flags
        } else {
            &mut self.args
        };
        for name in names.iter() {
            map.entry(*name).or_default().insert(id, branch);
        }
        // println!("Added {names:?}, now it is {self:?}");
    }

    pub(crate) fn remove_named(&mut self, flag: bool, id: Id, names: &[Name<'static>]) {
        let branch = self.tasks.get(&id).unwrap().branch;
        let map = if flag {
            &mut self.flags
        } else {
            &mut self.args
        };
        for name in names {
            let std::collections::btree_map::Entry::Occupied(mut entry) = map.entry(*name) else {
                continue;
            };
            entry.get_mut().remove(branch, id);
            if entry.get().0.is_empty() {
                entry.remove();
            }
        }
        self.tasks.remove(&id);
    }

    pub(crate) fn pick_parsers_for(
        &mut self,
        front: &Arg<'ctx>,
        out: &mut VecDeque<(Id, usize)>,
    ) -> Result<(), Error> {
        out.clear();
        // Populate ids with tasks that subscribed for the next token
        println!("Picking parser to deal with {front:?}");

        // first we need to decide what parsers to run
        match front {
            Arg::Named {
                name,
                value: Some(arg),
            } => {
                if let Some(q) = self.args.get_mut(name) {
                    q.peek_front(out);
                } else {
                    todo!("not found {name:?}")
                }
            }
            Arg::Named { name, value: None } => {
                if let Some(q) = self.flags.get_mut(name) {
                    q.peek_front(out);
                };
                if let Some(q) = self.args.get_mut(name) {
                    q.peek_front(out);
                };
                if out.is_empty() {
                    todo!("no such {name:?}");
                }
            }
            Arg::ShortSet { names, current } => todo!(),
            Arg::Positional { value: _ } => {
                self.positional.peek_front(out);
            }
        }
        println!("Got {out:?}");
        Ok(())
    }

    pub(crate) fn insert(&mut self, parent: Parent, id: Id) {
        let branch = match parent.kind {
            NodeKind::Sum => BranchId {
                parent: parent.id,
                field: parent.field,
            },
            NodeKind::Prod => self
                .tasks
                .get(&parent.id)
                .map_or(BranchId::ROOT, |t| t.branch),
        };

        let info = TaskInfo { parent, branch };
        self.tasks.insert(id, info);
    }

    pub(crate) fn remove(&mut self, id: Id) {
        self.tasks.remove(&id);
    }
}

/// # Pecking order
///
/// For as long as there's only one task to wake up for the input - it is safe to just
/// wake it up and be done with it, but users are allowed to specify multiple consumers for the
/// same name as well as multiple positional consumers that don't have names at all. This requires
/// deciding which parser gets to run first or gets to run at all.
///
/// Rules for priority are:
///
/// - sum branches run in parallel, left most wins if there's multiple successes
/// - parsers inside a product run sequentially, left most wins
///
/// Therefore we are going to arrange tasks in following order:
/// There's one queue for each branch_id (sum parent id + field), every queue contains
/// items from the same product, so their priority is how far from the left end they are
///
/// In practice all we need is a single BTreeSet :)
///
/// "any" parsers get to run for both named and positional input inside their branch
/// accoding to their priority, if at the front. Consider a few queues
/// - `[named, any]` - `any` doesn't run since `named` takes priority
/// - `[any1, named, any2]` - `any1` runs, if it fails to match anything - `named` runs.
/// - `[any1, any2, named]` - `any1` runs, if not - `any2`, if not - `named`
///
/// "any" are mixed with positional items the same way so we'll have to mix them in dynamically...

#[derive(Debug, Clone, Default)]
pub(crate) struct Pecking(BTreeSet<(Id, BranchId)>);

impl Pecking {
    /// removes an item from a pecking order,
    fn remove(&mut self, branch: BranchId, id: Id) {
        self.0.remove(&(id, branch));
    }

    fn insert(&mut self, id: Id, branch: BranchId) {
        self.0.insert((id, branch));
    }

    fn peek_front(&self, ids: &mut VecDeque<(Id, usize)>) {
        let mut prev_branch = None;
        for (id, branch) in self.0.iter() {
            if let Some(prev) = prev_branch {
                if prev >= branch {
                    break;
                }
            }
            ids.push_back((*id, 0));
            prev_branch = Some(branch);
        }
    }
}
