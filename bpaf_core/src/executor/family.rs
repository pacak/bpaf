//! Relationship between different tasks
//!
//! Tasks are spawned and executed in prod/sum groups.
//!
//! For parallel case it is important to know mutual exclusivity, for sequential case - parser priority

use std::collections::{BTreeMap, HashMap, VecDeque};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Parent {
    pub(crate) kind: NodeKind,
    pub(crate) id: Id,
    pub(crate) field: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
    // I want to know the position of the item in the virtual product
    // for that I need to know parent offset as well as child on the left offset...
    //
    // (a*, b): a < b; [0] < [1]
    // ((a*, b), c): a < b < c: [0, 0] < [0, 1] < [1]
    // ((e, b, a*), c): e < b < a < c; [0, 0] < [0, 1] < [0, 2] < [1]
    // ((a, b), (c, d)): a < b < c < d; [0, 0] < [0, 1] < [1, 0] < [1, 1]
    //
    // Easiest way is to build paths to the each id and sort them in
    // lexicographical order... But this is allocations.
    // We can treat it by sort of linked list
    // by keeping depth and field number. This way
    // we can recover the order by getting right/left item
    // to the same depth, if they match parent - compare field ids.
    // otherwise - keep going up until branch id...
    //
    // Alternatively - try to reuse the same task ids and have queus as ordered sets...
}

// For any node I need to be able to find all sum siblings
// and order prod siblings in a pecking order
#[derive(Debug, Default)]
pub(crate) struct FamilyTree {
    named: BTreeMap<Name<'static>, Pecking>,
    positional: Pecking,
    tasks: HashMap<Id, TaskInfo>,
}

impl FamilyTree {
    pub(crate) fn add_positional(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.positional.insert(id, branch);
    }

    pub(crate) fn remove_positional(&mut self, id: Id) {
        let branch = self.tasks.get(&id).unwrap().branch;
        self.positional.remove(branch, id);
    }

    pub(crate) fn add_named(&mut self, id: Id, names: &[Name<'static>]) {
        let branch = self.tasks.get(&id).unwrap().branch;

        for name in names.iter() {
            self.named.entry(*name).or_default().insert(id, branch);
        }
        println!("Added {names:?}, now it is {self:?}");
    }

    pub(crate) fn remove_named(&mut self, id: Id, names: &[Name<'static>]) {
        let branch = self.tasks.get(&id).unwrap().branch;
        for name in names {
            let std::collections::btree_map::Entry::Occupied(mut entry) = self.named.entry(*name)
            else {
                continue;
            };
            entry.get_mut().remove(branch, id);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
        self.tasks.remove(&id);
    }

    pub(crate) fn pick_parsers_for(
        &mut self,
        input: &str,
        out: &mut VecDeque<(Id, usize)>,
    ) -> Result<(), Error> {
        out.clear();
        // Populate ids with tasks that subscribed for the next token
        println!("Picking parser to deal with {input:?}");

        // first we need to decide what parsers to run
        match split_param(input)? {
            Arg::Named { name, value: _ } => {
                let Some(q) = self.named.get_mut(name.as_bytes()) else {
                    return Err(Error::Invalid);
                };
                q.peek_front(out);
            }
            Arg::ShortSet { names } => todo!(),
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

    // fn top_sum_parent(&self, mut id: Id) -> Option<Parent> {
    //     let mut best = None;
    //     while let Some(parent) = self.parents.get(&id) {
    //         if parent.kind == NodeKind::Sum {
    //             best = Some(*parent);
    //         }
    //         id = parent.id;
    //     }
    //     best
    // }
    //
    // fn branch_for(&self, id: Id) -> BranchId {
    //     match self.top_sum_parent(id) {
    //         Some(p) => BranchId {
    //             parent: p.id,
    //             field: p.field,
    //         },
    //         None => BranchId::ROOT,
    //     }
    // }
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
/// "any" parsers get to run for both named and positional input inside their branch
/// accoding to their priority, if at the front. Consider a few queues
/// - `[named, any]` - `any` doesn't run since `named` takes priority
/// - `[any1, named, any2]` - `any1` runs, if it fails to match anything - `named` runs.
/// - `[any1, any2, named]` - `any1` runs, if not - `any2`, if not - `named`
///
/// "any" are mixed with positional items the same way so we'll have to mix them in dynamically...
#[derive(Debug, Default)]
enum Pecking {
    /// No parsers at all, this makes sense for `positional` and `any` items, with
    /// named might as well drop the parser
    #[default]
    Empty,

    /// A single parser
    ///
    /// Usually a unique named argument or a single positional item to the parser
    Single(BranchId, Id),
    /// There's multiple parsers, but they all belong to the same queue
    ///
    /// Several positional items
    Queue(BranchId, VecDeque<Id>),

    /// Multiple alternative branches, VecDeque contains at least one item
    Forest(HashMap<BranchId, VecDeque<Id>>),
}

fn remove_first(q: &mut VecDeque<Id>, id: Id) {
    let Some(pos) = q.iter().position(|i| *i == id) else {
        panic!("Trying to remove {id:?} that's not present in {q:?}");
    };
    q.remove(pos);
}

impl Pecking {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
    /// removes an item from a pecking order,
    fn remove(&mut self, branch: BranchId, id: Id) {
        match self {
            Self::Empty => panic!("trying to remove a listener from an empty"),
            Self::Single(branch_id, cur_id) => {
                assert_eq!(*branch_id, branch);
                assert_eq!(*cur_id, id);
                *self = Self::Empty;
            }
            Self::Queue(branch_id, vec_deque) => {
                assert_eq!(*branch_id, branch);
                remove_first(vec_deque, id);
                if vec_deque.is_empty() {
                    *self = Self::Empty;
                }
            }
            Self::Forest(hash_map) => {
                let std::collections::hash_map::Entry::Occupied(mut entry) = hash_map.entry(branch)
                else {
                    panic!("Trying to remove branch {branch:?} from {self:?}");
                };

                remove_first(entry.get_mut(), id);
                if entry.get().is_empty() {
                    entry.remove();
                }
                if hash_map.is_empty() {
                    *self = Self::Empty;
                }
            }
        }
    }

    fn insert(&mut self, id: Id, branch: BranchId) {
        match self {
            Pecking::Empty => *self = Pecking::Single(branch, id),
            Pecking::Single(prev_bi, prev_id) => {
                if *prev_bi == branch {
                    let mut queue = VecDeque::new();
                    queue.push_back(*prev_id);
                    queue.push_back(id);
                    *self = Pecking::Queue(branch, queue)
                } else {
                    let mut forest = HashMap::new();
                    let mut queue = VecDeque::new();
                    queue.push_back(*prev_id);
                    forest.insert(*prev_bi, queue);

                    let mut queue = VecDeque::new();
                    queue.push_back(id);
                    forest.insert(branch, queue);
                    *self = Pecking::Forest(forest)
                }
            }
            Pecking::Queue(prev_bi, vec_deque) => {
                if *prev_bi == branch {
                    vec_deque.push_back(id);
                } else {
                    let mut forest = HashMap::new();
                    forest.insert(*prev_bi, std::mem::take(vec_deque));
                    let mut queue = VecDeque::new();
                    queue.push_back(id);
                    forest.insert(branch, queue);
                    *self = Pecking::Forest(forest);
                }
            }
            Pecking::Forest(forest) => {
                forest.entry(branch).or_default().push_back(id);
            }
        }
    }

    fn peek_front(&self, ids: &mut VecDeque<(Id, usize)>) {
        match self {
            Pecking::Empty => panic!("empty_queue"),
            Pecking::Single(_branch_id, id) => {
                ids.push_back((*id, 0));
            }
            Pecking::Queue(_branch_id, vec_deque) => {
                if let Some(f) = vec_deque.front() {
                    ids.push_back((*f, 0));
                }
            }
            Pecking::Forest(hash_map) => {
                for m in hash_map.values() {
                    if let Some(f) = m.front() {
                        ids.push_back((*f, 0));
                    }
                }
            }
        }
    }

    // fn drain_to(&mut self, ids: &mut VecDeque<Id>) {
    //     match self {
    //         Pecking::Empty => {}
    //         Pecking::Single(branch_id, id) => {
    //             ids.push_back(*id);
    //         }
    //         Pecking::Queue(branch_id, vec_deque) => ids.extend(vec_deque.drain(..)),
    //         Pecking::Forest(hash_map) => {
    //             for mut queue in std::mem::take(hash_map).into_values() {
    //                 ids.extend(queue.drain(..));
    //             }
    //         }
    //     }
    //     *self = Pecking::Empty;
    // }
}
