use std::ops::{Index, IndexMut};

use crate::{Id, Scope, Task};

fn to_ix(id: Id) -> usize {
    let id = id.0 as usize;
    id.wrapping_sub(1)
}

/// A collection of tasks indexed by [`Id`]
#[derive(Default, Debug)]
pub(crate) struct Tasks<'a>(Vec<Option<Task<'a>>>);

impl<'a> Tasks<'a> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn insert(&mut self, id: Id, task: Task<'a>) -> Option<Task<'a>> {
        let ix = to_ix(id);
        if ix >= self.0.len() {
            self.0.resize_with(ix + 1, || None);
        }
        self.0[ix].replace(task)
    }

    pub(crate) fn remove(&mut self, id: Id) -> Option<Task<'a>> {
        self.0.get_mut(to_ix(id)).and_then(|slot| slot.take())
    }
}

impl<'a> Index<Id> for Tasks<'a> {
    type Output = Task<'a>;

    fn index(&self, id: Id) -> &Task<'a> {
        self.0[to_ix(id)]
            .as_ref()
            .unwrap_or_else(|| panic!("Task {id:?} not found"))
    }
}

impl<'a> IndexMut<Id> for Tasks<'a> {
    fn index_mut(&mut self, id: Id) -> &mut Task<'a> {
        self.0[to_ix(id)]
            .as_mut()
            .unwrap_or_else(|| panic!("Task {id:?} not found"))
    }
}

/// Cursor state for [`Tasks::drain_next`], initialized by [`Tasks::drain_tasks_rev`]
#[derive(Debug)]
pub(crate) struct DrainRevState {
    start: usize,
    next: usize,
}

impl<'a> Tasks<'a> {
    /// Start draining tasks in a [`Scope`] in reverse order
    pub(crate) fn drain_tasks_rev(&self, scope: Scope) -> DrainRevState {
        let start = scope.start.0.saturating_sub(1) as usize;
        let end = (scope.end.0.saturating_sub(1) as usize).min(self.0.len());
        DrainRevState { start, next: end }
    }

    /// Extract the next task using the cursor from [`Tasks::drain_tasks_rev`]
    pub(crate) fn drain_next(&mut self, state: &mut DrainRevState) -> Option<Task<'a>> {
        while state.next > state.start {
            state.next -= 1;
            if let Some(task) = self.0[state.next].take() {
                return Some(task);
            }
        }
        None
    }

    /// Assert that there's no leftover tasks past the end
    ///
    /// And truncate the inner vector to make iterations faster
    pub(crate) fn assert_no_tasks_past_end(&mut self, end: u32) {
        let start = end.saturating_sub(1) as usize;
        assert!(self.0[start..].iter().all(|t| t.is_none()));
        self.0.truncate(start);
    }
}
