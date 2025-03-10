use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize},
};

use crate::{
    error::Error,
    executor::{futures::JoinHandle, Action, BranchId, Id, Op, Parent, RawAction},
    named::Name,
    parsers::HelpWrap,
    split::OsOrStr,
    Parser,
};

pub struct RawCtx<'a> {
    /// Gets populated with current taskid when it is running
    pub(crate) current_task: RefCell<Option<(BranchId, Id)>>,
    /// All the arguments passed to the app including the app name in 0th
    pub(crate) args: &'a [OsOrStr<'a>],
    /// Current cursor position
    cur: AtomicUsize,
    /// For option arguments with immediately adjacent values this
    /// will be set to the value:
    ///
    /// `--foo bar` - None,
    /// `--foo=bar` or `-fbar` - Some("bar")
    pub(crate) front_value: RefCell<Option<OsOrStr<'a>>>,
    // pub(crate) front: RefCell<Option<Arg<'a>>>,
    /// through this tasks can request event scheduling, etc
    pub(crate) shared: RefCell<VecDeque<Op<'a>>>,

    /// Used to pass information about children exit
    pub(crate) child_exit: Cell<Option<Error>>,

    /// number of items consumed by children tasks
    pub(crate) items_consumed: Cell<u32>,

    /// By the end we are trying to kill all the children, when set - children
    /// should fail when encounter unexpected input, otherwise - keep running
    term: AtomicBool,

    /// Start of the current parsing context
    /// For the top level option parser this excludes things like the app name and
    /// cargo invocation if present, for subcommands this exludes the path to get to it
    pub(crate) ctx_start: Cell<u32>,

    /// TODO
    pub(crate) help_and_version: &'a dyn Parser<HelpWrap>,
}

#[derive(Clone)]
#[repr(transparent)]
// this is a newtype instead of struct since things like RawCtx::spawn
// need to pass it by ownership
pub struct Ctx<'a>(Rc<RawCtx<'a>>);

impl<'a> Ctx<'a> {
    pub(crate) fn new(
        args: &'a [OsOrStr<'a>],
        ctx_start: u32,
        help_and_version: &'a dyn Parser<HelpWrap>,
    ) -> Self {
        Ctx(Rc::new(RawCtx {
            args,
            current_task: Default::default(),
            items_consumed: Default::default(),
            shared: Default::default(),
            cur: AtomicUsize::from(0),
            front_value: Default::default(),
            child_exit: Default::default(),
            term: Default::default(),
            ctx_start: Cell::new(ctx_start),
            help_and_version,
        }))
    }
}

impl<'a> std::ops::Deref for Ctx<'a> {
    type Target = RawCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RawCtx<'_> {
    pub fn current_id(&self) -> (BranchId, Id) {
        self.current_task.borrow().expect("not in a task")
    }
    pub(crate) fn current_ctx(&self) -> &[OsOrStr] {
        &self.args[self.ctx_start.get() as usize..]
    }
    pub(crate) fn cur(&self) -> usize {
        self.cur.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub(crate) fn set_term(&self, val: bool) {
        self.term.store(val, std::sync::atomic::Ordering::Relaxed);
    }
    pub(crate) fn is_term(&self) -> bool {
        self.term.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn set_cur(&self, new: usize) {
        self.cur.store(new, std::sync::atomic::Ordering::Relaxed);
    }
    pub(crate) fn advance(&self, inc: usize) {
        self.cur
            .fetch_add(inc, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<'a> Ctx<'a> {
    pub fn spawn<T, P>(&self, parent: Parent, parser: &'a P, keep_id: bool) -> JoinHandle<'a, T>
    where
        P: Parser<T> + ?Sized,
        T: 'static,
    {
        let ctx = self.clone();
        let (exit, join) = self.fork();
        let act = Box::pin(async move {
            exit.id.set(Some(ctx.current_task().1));
            let r = parser.run(ctx).await;

            if let Ok(exit) = Rc::try_unwrap(exit) {
                // This is used to handle erors in Con or similar in execution order
                // instead of definition order with `EarlyExitFut`
                exit.exit_task(r)
            } else {
                unreachable!("We have more than one copy of the exit handle!")
            }
        });
        self.start_task(parent, act, keep_id);
        join
    }

    fn queue(&self, op: Op<'a>) {
        self.shared.borrow_mut().push_back(op);
    }

    pub(crate) fn add_literal_wake(&self, values: &'a [Name<'static>], branch: BranchId, id: Id) {
        self.queue(Op::AddLiteral { branch, id, values })
    }

    pub(crate) fn remove_literal(&self, values: &'a [Name<'static>], branch: BranchId, id: Id) {
        self.queue(Op::RemoveLiteral { branch, id, values })
    }

    pub(crate) fn add_any_wake(&self, branch: BranchId, id: Id) {
        self.queue(Op::AddAny { branch, id });
    }
    pub(crate) fn remove_any(&self, branch: BranchId, id: Id) {
        self.queue(Op::RemoveAny { branch, id });
    }

    pub(crate) fn add_named_wake(
        &self,
        flag: bool,
        names: &'a [Name<'static>],
        branch: BranchId,
        id: Id,
    ) {
        self.queue(Op::AddNamedListener {
            flag,
            names,
            branch,
            id,
        });
    }

    pub(crate) fn add_fallback(&self, branch: BranchId, id: Id) {
        self.queue(Op::AddFallback { branch, id });
    }

    pub(crate) fn remove_fallback(&self, branch: BranchId, id: Id) {
        self.queue(Op::RemoveFallback { branch, id });
    }

    pub(crate) fn remove_named_listener(
        &self,
        flag: bool,
        branch: BranchId,
        id: Id,
        names: &'a [Name<'static>],
    ) {
        self.queue(Op::RemoveNamedListener {
            flag,
            names,
            branch,
            id,
        });
    }

    pub(crate) fn positional_wake(&self, branch: BranchId, id: Id) {
        self.queue(Op::AddPositionalListener { branch, id })
    }

    pub(crate) fn start_task(&self, parent: Parent, action: RawAction<'a>, keep_id: bool) {
        self.queue(Op::SpawnTask {
            parent,
            action: Action::Raw(action),
            keep_id,
        });
    }

    pub(crate) fn current_task(&self) -> (BranchId, Id) {
        self.current_task
            .borrow()
            .expect("should only be called from a Future")
    }
}
