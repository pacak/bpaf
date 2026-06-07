//! A context shared between [`Task`] and [`Executor`]. Ideally I want to pass it to tasks when I'm
//! trying to run them, but async monad is very limited in Rust so shared state in [`Rc`] it is.

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::OsStr,
    hash::Hash,
    rc::Rc,
};

#[allow(unused)] // used by `rustdoc`,
use crate::Executor;

use crate::{
    Conflict, Id, KillReason, Lit, Name, PeckingOrder, Reason, TChange, TTarget, Task, TaskInfo,
    args::Args, info::Custom,
};

type DynamicOsStrCheck = Rc<dyn Fn(&OsStr) -> bool>;

/// A collection of mappings from values encountered in arguments to
/// tasks scheduled to consume them, in some priority order
#[derive(Default)]
pub(crate) struct Triggers {
    // `-f`, `--foo`
    pub(crate) flags: HashMap<Name<'static>, PeckingOrder>,
    // `-f=bar` `--foo=bar`, `-fbar`
    pub(crate) args: HashMap<Name<'static>, PeckingOrder>,
    // `foo`
    pub(crate) pos: PeckingOrder,
    // `-1`
    pub(crate) checks: BTreeMap<Id, DynamicOsStrCheck>,
    // `a`, `alpha`
    pub(crate) literal: HashMap<Lit<'static>, PeckingOrder>,
}

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
    pub(crate) args: &'p Args,
    pub(crate) path: String,
    pub(crate) cursor: Cell<u32>,

    /// Value we are parsing - positional value itself or argument part of option_argument
    pub(crate) current_value: RefCell<Option<&'p OsStr>>,

    /// When task is woken up this contains a reason for it
    pub(crate) wakeup_reason: RefCell<Reason<'p>>,

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

    /// Trigger registry - maps argument patterns to parser tasks
    pub(crate) triggers: RefCell<Triggers>,

    pub(crate) custom: &'p Custom,
}

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

impl<'p> RawCtx<'p> {
    pub(crate) fn add_named_trigger<K: Eq + Hash + Clone>(
        &self,
        names: &[K],
        getmap: impl FnOnce(&mut Triggers) -> &mut HashMap<K, PeckingOrder>,
    ) {
        let cur = self.current_task.borrow();
        let mut t = self.triggers.borrow_mut();
        let map = getmap(&mut t);
        for name in names.iter().cloned() {
            map.entry(name)
                .or_default()
                .insert(cur.parent_id, cur.id, cur.parent_kind);
        }
    }

    pub(crate) fn remove_named_trigger<K: Eq + Hash + Clone>(
        &self,
        names: &[K],

        getmap: impl FnOnce(&mut Triggers) -> &mut HashMap<K, PeckingOrder>,
    ) {
        use std::collections::hash_map::Entry;

        let cur = self.current_task.borrow();
        let mut t = self.triggers.borrow_mut();
        let map = getmap(&mut t);
        for name in names.iter().cloned() {
            if let Entry::Occupied(mut e) = map.entry(name)
                && e.get_mut().remove(cur.id)
            {
                e.remove();
            }
        }
    }

 }
