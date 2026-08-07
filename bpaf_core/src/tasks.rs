use std::{
    ops::{Index, IndexMut},
    pin::Pin,
};

use crate::Scope;

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

/// A parsing thread tracked by the executor
///
/// Doesn't produce a value directly, instead uses a pair of [`ExitHandle`]/[`JoinHandle`] to
/// pass the value to the parent thread.
///
/// Tasks form a tree structure
pub(crate) struct Task<'a> {
    pub(crate) act: Pin<Box<dyn Future<Output = ()> + 'a>>,
    pub(crate) info: TaskInfo,
}

impl std::fmt::Debug for Task<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Task { act: _, info } = self;
        f.debug_struct("Task").field("info", info).finish()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Sum,
    Prod,
}

/// Shared state and meta info about tasks
///
/// Used by the executor but also shared with the task itself as it runs in
/// [`SharedCtx::current_task`]
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskInfo {
    /// Current task Id
    pub(crate) id: Id,
    pub(crate) parent_kind: Kind,
    pub(crate) parent_id: Parent,
    /// For trigger tasks having it nonzero means the task is done
    /// and won't be consuming anything
    pub(crate) consumed: u32,
    /// number of children this task is currently waiting for, we are going to wake them up only if
    // there's no pending children - if we can return immediately
    pub(crate) pending: u32,
    pub(crate) state: TaskState,
    /// If true, this task (and all its children) use the global trigger set
    pub(crate) global: bool,
}

/// Tracks tasks state
///
/// Updates once, right before storing result in the output handle
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum TaskState {
    Pending,
    Success,
    Failure,
}

impl From<bool> for TaskState {
    fn from(success: bool) -> Self {
        if success {
            Self::Success
        } else {
            Self::Failure
        }
    }
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            id: Id(0),
            parent_kind: Kind::Sum,
            parent_id: Parent(0),
            consumed: 0,
            pending: 0,
            state: TaskState::Pending,
            global: false,
        }
    }
}

/// A newtyped [`Id`] for parser tracking relationship
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Parent(pub(crate) u32);
impl Parent {
    pub(crate) fn as_id(&self) -> Id {
        Id(self.0)
    }

    pub(crate) fn is_root(&self) -> bool {
        *self == Parent(0)
    }
}

/// An `Id` of a parser instance
///
/// Used to track parsers in the Executor as well as relationship between parsers in the
/// [`PeckingOrder`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Id(pub(crate) u32);
