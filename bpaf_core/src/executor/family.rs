//! Relationship between different tasks
//!
//! Tasks are spawned and executed in prod/sum groups.
//!
//! For parallel case it is important to know mutual exclusivity, for sequential case - parser priority

use crate::{error::Error, executor::Arg, named::Name, pecking::Pecking};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(u32);
impl Id {
    pub(crate) const ZERO: Self = Self(0);
    const ROOT: Self = Self(1);
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

    pub fn prod(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Prod,
            id: self,
            field,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Parent {
    pub(crate) kind: NodeKind,
    pub(crate) id: Id,
    pub(crate) field: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct BranchId {
    pub(crate) parent: Id,
    pub(crate) field: u32,
}

impl std::fmt::Debug for BranchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B({}:{})", self.parent.0, self.field)
    }
}

impl BranchId {
    pub(crate) const ZERO: Self = Self {
        parent: Id(0),
        field: 0,
    };
    pub(crate) const ROOT: Self = Self {
        parent: Id::ROOT,
        field: 0,
    };
    pub(crate) fn succ(&self) -> Self {
        Self {
            parent: self.parent,
            field: self.field + 1,
        }
    }
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
    // TODO - use HashMap?
    pub(crate) flags: BTreeMap<Name<'ctx>, Pecking>,
    pub(crate) args: BTreeMap<Name<'ctx>, Pecking>,
    fallback: Pecking,
    positional: Pecking,
    tasks: HashMap<Id, TaskInfo>,
    conflicts: BTreeMap<Name<'ctx>, usize>,
}

impl<'ctx> FamilyTree<'ctx> {
    pub(crate) fn add_positional(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.positional.insert(branch, id);
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
            self.conflicts.remove(name);
            map.entry(name.clone()).or_default().insert(branch, id);
        }
        // println!("Added {names:?}, now it is {self:?}");
    }

    pub(crate) fn add_fallback(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.fallback.insert(branch, id);
    }
    pub(crate) fn remove_fallback(&mut self, id: Id) {
        println!("removing fallback {id:?}");
        let branch = self.tasks.get(&id).unwrap().branch;
        self.fallback.remove(branch, id);
    }

    pub(crate) fn remove_named(
        &mut self,
        flag: bool,
        id: Id,
        names: &[Name<'static>],
        conflict: Option<usize>,
    ) {
        if let Some(conflict) = conflict {
            for name in names {
                self.conflicts.insert(name.clone(), conflict);
            }
        }
        let branch = self.tasks.get(&id).unwrap().branch;
        let map = if flag {
            &mut self.flags
        } else {
            &mut self.args
        };
        for name in names {
            let std::collections::btree_map::Entry::Occupied(mut entry) = map.entry(name.clone())
            else {
                continue;
            };
            entry.get_mut().remove(branch, id);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
        //        self.tasks.remove(&id);
    }

    pub(crate) fn pick_fallback(&mut self, out: &mut Vec<(BranchId, Id)>) {
        out.clear();
        out.extend(self.fallback.heads());
    }

    pub(crate) fn pick_parsers_for(
        &mut self,
        front: &Arg<'ctx>,
        out: &mut Vec<(BranchId, Id)>,
    ) -> Result<(), Error> {
        out.clear();
        // Populate ids with tasks that subscribed for the next token
        println!("Picking parser to deal with {front:?}");

        // first we need to decide what parsers to run
        match front {
            Arg::Named {
                name,
                value: Some(_),
            } => {
                if let Some(q) = self.args.get_mut(name) {
                    out.extend(q.iter());
                //                    q.queue_all(out);
                } else {
                    todo!("not found {name:?}")
                }
            }
            Arg::Named { name, value: None } => {
                if let Some(q) = self.flags.get_mut(name) {
                    out.extend(q.heads());
                };
                if let Some(q) = self.args.get_mut(name) {
                    out.extend(q.heads());
                };
                if out.is_empty() {
                    if let Some(x) = self.conflicts.get(name) {
                        return Err(Error::parse_fail(format!("{name} conflicts with {x}")));
                    }
                }
            }
            Arg::ShortSet { names, current } => todo!(),
            Arg::Positional { value: _ } => {
                out.extend(self.positional.heads());
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
