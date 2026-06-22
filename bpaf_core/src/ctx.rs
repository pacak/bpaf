//! A context shared between [`Task`] and [`Executor`]. Ideally I want to pass it to tasks when I'm
//! trying to run them, but async monad is very limited in Rust so shared state in [`Rc`] it is.

use std::{
    cell::{Cell, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::OsStr,
    hash::Hash,
    rc::Rc,
};

#[allow(unused)] // used by `rustdoc`,
use crate::Executor;

use crate::{
    Conflict, Id, KillReason, Lit, Name, PeckingOrder, Reason, TaskInfo,
    args::Args,
    info::{Custom, Extra},
    tasks::Tasks,
    traits::BoxParser,
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

/// State shared between forked contexts
pub(crate) struct SharedCtx<'p> {
    /// Arguments we are parsing
    pub(crate) args: &'p Args,

    /// Current position in the argument list
    pub(crate) cursor: Cell<u32>,

    /// customizations: style, --help and --version parsers
    pub(crate) custom: &'p Custom,

    /// instantiated --help/--version parser
    pub(crate) help_and_version: &'p BoxParser<Extra>,

    /// Global task pool shared across contexts from fork
    pub(crate) tasks: RefCell<Tasks<'p>>,

    /// Next free Id
    ///
    /// Tasks can use it to allocate `Id`s for children tasks including overriding it temporary
    pub(crate) next_free: Cell<u32>,

    /// When task is woken up this contains a reason for it
    pub(crate) wakeup_reason: RefCell<Reason<'p>>,

    /// Global trigger set, shared across all executor scopes.
    ///
    /// Tasks marked with [`global`](crate::Parser::global) register their triggers here.
    pub(crate) global_triggers: Rc<RefCell<Triggers>>,

    /// Reference to [`TaskInfo`] for the current task
    ///
    /// Executor sets it to the right value when polling a task.
    pub(crate) current_task: RefCell<TaskInfo>,

    /// Active sum parsers and their scopes
    pub(crate) sums: RefCell<BTreeMap<Id, Scope>>,

    /// Scheduled ops
    ///
    /// Holds other operations that might need access to tasks or other internal structures
    pub(crate) pending_ops: RefCell<VecDeque<Op>>,

    /// Conflict records - early terminated branches saved for error reporting
    pub(crate) conflicts: RefCell<Vec<Conflict>>,
}

/// State shared between all the parsers, used via [`Ctx`] alias
pub struct RawCtx<'p> {
    /// Early exit ranges
    ///
    /// When there's no matching triggers executor will try to terminate anything inside of
    /// the each pair. This allows is necessary for things like `.optional()` and `.many()` to work
    pub(crate) early_exit: RefCell<BTreeSet<Scope>>,

    pub(crate) shared: Rc<SharedCtx<'p>>,
    pub(crate) path: String,

    /// Value we are parsing - positional value itself or argument part of option_argument
    pub(crate) current_value: RefCell<Option<&'p OsStr>>,

    /// Treat the rest of the items as strictly positional - we'll set it if we ever encounter a
    /// bare `--` during parsing
    ///
    /// Needs to live here so it gets shared to nested parsers that get a new executor
    pub(crate) strict_pos: Cell<bool>,

    /// Trigger registry - maps argument patterns to parser tasks
    ///
    /// Per-executor (local) trigger set. Replaced with
    /// [`SharedCtx::global_triggers`] when polling a global task.
    pub(crate) triggers: RefCell<Triggers>,
}

pub type Ctx<'p> = Rc<RawCtx<'p>>;

#[derive(Debug)]
pub(crate) enum Op {
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
    pub(crate) fn contains(&self, id: Id) -> bool {
        self.start <= id && id < self.end
    }
}

impl<'p> RawCtx<'p> {
    /// Returns a mutable borrow of the trigger set appropriate for the current task.
    ///
    /// Global tasks borrow [`SharedCtx::global_triggers`], others
    /// borrow the per-executor local [`triggers`](RawCtx::triggers).
    pub(crate) fn task_triggers(&self) -> RefMut<'_, Triggers> {
        if self.shared.current_task.borrow().global {
            self.shared.global_triggers.borrow_mut()
        } else {
            self.triggers.borrow_mut()
        }
    }

    pub(crate) fn add_named_trigger<K: Eq + Hash + Clone>(
        &self,
        names: &[K],
        getmap: impl FnOnce(&mut Triggers) -> &mut HashMap<K, PeckingOrder>,
    ) {
        let cur = self.shared.current_task.borrow();
        let mut t = self.task_triggers();
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

        let cur = self.shared.current_task.borrow();
        let mut t = self.task_triggers();
        let map = getmap(&mut t);
        for name in names.iter().cloned() {
            if let Entry::Occupied(mut e) = map.entry(name)
                && e.get_mut().remove(cur.id)
            {
                e.remove();
            }
        }
    }

    pub(crate) fn cursor(&self) -> &Cell<u32> {
        &self.shared.cursor
    }
}
