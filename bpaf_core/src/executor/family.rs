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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct BranchId {
    parent: Id,
    field: u32,
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
    parent: Parent,
    branch: BranchId,
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
        println!("need to remove positional {id:?}");
        //        self.positional.remove(id);
    }

    pub(crate) fn add_named(&mut self, id: Id, names: &[Name<'static>]) {
        let branch = self.tasks.get(&id).unwrap().branch;

        for name in names.iter() {
            self.named.entry(*name).or_default().insert(id, branch);
        }
        println!("Added {names:?}, now it is {self:?}");
    }

    pub(crate) fn remove_named(&mut self, id: Id, names: &[Name<'static>]) {
        for name in names {
            self.named.remove(name);
        }
        println!("remove named listener for {names:?} {id:?}");
    }

    pub(crate) fn pick_parsers_for(
        &mut self,
        input: &str,
        out: &mut VecDeque<(Id, usize)>,
    ) -> Result<(), Error> {
        // Populate ids with tasks that subscribed for the next token
        println!("Picking parser to deal with {input:?}");

        // first we need to decide what parsers to run
        match split_param(input)? {
            Arg::Named { name, val: _ } => {
                let Some(q) = self.named.get_mut(name.as_bytes()) else {
                    return Err(Error::Invalid);
                };

                println!("looking in {q:?}");
                q.pop_front(out);
                println!("got {out:?}");
            }
            Arg::ShortSet { names } => todo!(),
            Arg::Positional { value } => {
                self.positional.pop_front(out);
            }
        }
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

//

// [[X, x], x] => [[X, _], _]
// [[x, x], X] => [_, X]

// [[X, (x, [x, x])], x] => [[X, _], _]
// [[x, (X, [x, x])], x] => [[_, (X, [x, x])]
// [[x, (x, [x, x])], x]

// #[test]
// fn alt_parent_1() {
//     let mut f = FamilyTree::default();
//     f.insert(Id(0).sum(0), Id(1));
//     f.insert(Id(1).sum(0), Id(2));
//     f.insert(Id(1).sum(1), Id(3));
//
//     assert_eq!(Id(0), f.top_sum_parent(Id(1)).unwrap().id);
//     assert_eq!(Id(0), f.top_sum_parent(Id(2)).unwrap().id);
//     assert_eq!(Id(0), f.top_sum_parent(Id(3)).unwrap().id);
//
//     f.remove(Id(3));
//     f.remove(Id(2));
//     f.remove(Id(1));
//
//     assert_eq!(f.tasks.len(), 0);
// }

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
///
///
/// # Operations needed
///
/// - `Pecking::insert`
/// - `Pecking::select`
/// - `Pecking::remove`?
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

impl Pecking {
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

    fn pop_front(&mut self, ids: &mut VecDeque<(Id, usize)>) -> usize {
        match self {
            Pecking::Empty => 0,
            Pecking::Single(branch_id, id) => {
                ids.push_back((*id, 0));
                *self = Pecking::Empty;
                1
            }
            Pecking::Queue(branch_id, vec_deque) => {
                if let Some(f) = vec_deque.pop_front() {
                    ids.push_back((f, 0));
                    1
                } else {
                    0
                }
            }
            Pecking::Forest(hash_map) => {
                let mut cnt = 0;
                for m in hash_map.values_mut() {
                    if let Some(f) = m.pop_front() {
                        ids.push_back((f, 0));
                        cnt += 1;
                    }
                }
                cnt
            }
        }
    }

    fn drain_to(&mut self, ids: &mut VecDeque<Id>) {
        match self {
            Pecking::Empty => {}
            Pecking::Single(branch_id, id) => {
                ids.push_back(*id);
            }
            Pecking::Queue(branch_id, vec_deque) => ids.extend(vec_deque.drain(..)),
            Pecking::Forest(hash_map) => {
                for mut queue in std::mem::take(hash_map).into_values() {
                    ids.extend(queue.drain(..));
                }
            }
        }
        *self = Pecking::Empty;
    }
}
