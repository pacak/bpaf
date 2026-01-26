//! A context shared between [`Task`] and [`Executor`]. Ideally I want to pass it to tasks when I'm
//! trying to run them, but async monad is very limited in Rust so shared state in [`Rc`] it is.

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, VecDeque},
    rc::Rc,
};

#[allow(unused)] // used by `rustdoc`,
use crate::Executor;

use crate::{
    Conflict, Id, KillReason, Parent, Reason, TChange, TTarget, Task, TaskInfo, args::Args,
};

/// State shared between all the parsers, used via [`Ctx`] alias
pub struct RawCtx<'p> {
    /// Scheduled ops
    ///
    /// Holds other operations that might need access to tasks or other internal structures
    pub(crate) pending_ops: RefCell<VecDeque<Op<'p>>>,

    /// Early exit ranges
    ///
    /// When there's no matching triggers executor will try to terminate anything inside of
    /// the each pair. This allows is necessary for things like `.optional()` and `.many()` to work
    pub(crate) early_exit: RefCell<BTreeSet<Scope>>,

    /// Next free Id
    ///
    /// Tasks can use it to allocate `Id`s for children tasks including overriding
    pub(crate) next_free: Cell<u32>,

    /// Arguments we are parsing as well as a cursor
    pub(crate) args: Args,
    pub(crate) cursor: Cell<usize>,

    /// When task is woken up this contains a reason for it
    pub(crate) wakeup_reason: RefCell<Reason>,

    /// Reference to [`TaskInfo`] for the current task
    ///
    /// Executor sets it to the right value when polling a task
    pub(crate) current_task: RefCell<TaskInfo>,

    /// We keep track of all the early terminated branches, saving each termination along with
    /// the position - from that we'll deduce conflict info
    pub(crate) conflicts: RefCell<Vec<Conflict>>,

    /// Treat the rest of the items as strictly positional - we'll set it if we ever encounter a
    /// bare `--` during parsing
    ///
    /// Needs to live here so it gets shared to nested parsers that get a new executor
    pub(crate) strict_pos: Cell<bool>,

    pub(crate) help_parser: &'p crate::RcParser<crate::Extra>,
}

///
pub type Ctx<'p> = Rc<RawCtx<'p>>;

#[derive(Debug)]
pub(crate) enum Op<'p> {
    Spawn(Task<'p>),
    RegisterSum {
        id: Id,
        scope: Scope,
    },
    DeregisterSum {
        id: Id,
    },
    KillScope {
        cursor: u32,
        scope: Scope,
        reason: KillReason,
    },
    Trigger {
        change: TChange,
        target: TTarget,
        parent: Parent,
        id: Id,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Scope {
    pub(crate) start: Id,
    pub(crate) end: Id,
}

impl Scope {
    pub(crate) const ALL: Scope = Scope {
        start: Id(0),
        end: Id(u32::MAX),
    };
    pub(crate) fn contains(&self, id: Id) -> bool {
        self.start <= id && id < self.end
    }
}
