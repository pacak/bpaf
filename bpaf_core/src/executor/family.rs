use std::collections::{BTreeMap, HashMap};

use super::{Action, BranchId, Id, NodeKind, Parent};

#[derive(Debug)]
struct Node {
    ty: NodeKind,
    children: BTreeMap<u32, Id>,
}

// For any node I need to be able to find all sum siblings
// and order prod siblings in a pecking order
#[derive(Debug, Default)]
pub(crate) struct FamilyTree {
    children: HashMap<Id, Node>,  // parent -> children
    parents: HashMap<Id, Parent>, // child -> parent
}

impl FamilyTree {
    pub(crate) fn insert(&mut self, parent: Parent, id: Id) {
        self.parents.insert(id, parent);
        let entry = self.children.entry(parent.id).or_insert(Node {
            ty: parent.kind,
            children: BTreeMap::new(),
        });
        entry.children.insert(parent.field, id);
    }

    pub(crate) fn remove(&mut self, id: Id) {
        use std::collections::hash_map::Entry;
        let Some(parent) = self.parents.remove(&id) else {
            return;
        };
        let Entry::Occupied(mut e) = self.children.entry(parent.id) else {
            return;
        };
        e.get_mut().children.remove(&parent.field);
        self.children.remove(&id);
    }
    //        fn missing_siblings(&self) {}

    fn top_sum_parent(&self, mut id: Id) -> Option<Parent> {
        let mut best = None;
        while let Some(parent) = self.parents.get(&id) {
            if parent.kind == NodeKind::Sum {
                best = Some(*parent);
            }
            id = parent.id;
        }
        best
    }

    pub(crate) fn branch_for(&self, id: Id) -> BranchId {
        match self.top_sum_parent(id) {
            Some(p) => BranchId {
                parent: p.id,
                field: p.field,
            },
            None => BranchId::ROOT,
        }
    }
}

//

// [[X, x], x] => [[X, _], _]
// [[x, x], X] => [_, X]

// [[X, (x, [x, x])], x] => [[X, _], _]
// [[x, (X, [x, x])], x] => [[_, (X, [x, x])]
// [[x, (x, [x, x])], x]

#[test]
fn alt_parent_1() {
    let mut f = FamilyTree::default();
    f.insert(Id(0).sum(0), Id(1));
    f.insert(Id(1).sum(0), Id(2));
    f.insert(Id(1).sum(1), Id(3));

    assert_eq!(Id(0), f.top_sum_parent(Id(1)).unwrap().id);
    assert_eq!(Id(0), f.top_sum_parent(Id(2)).unwrap().id);
    assert_eq!(Id(0), f.top_sum_parent(Id(3)).unwrap().id);

    f.remove(Id(3));
    f.remove(Id(2));
    f.remove(Id(1));

    assert_eq!(f.children.len(), 1, "{f:?}");
    assert_eq!(f.parents.len(), 0);
}
