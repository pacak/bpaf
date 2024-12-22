use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize},
    task::{Context, Poll},
};

use crate::{
    executor::{
        futures::{ErrorHandle, JoinHandle},
        Action, BranchId, Id, Op, Parent, Task,
    },
    named::Name,
    split::{Arg, OsOrStr},
    Error, Parser,
};

pub struct RawCtx<'a> {
    /// Gets populated with current taskid when it is running
    pub(crate) current_task: RefCell<Option<(BranchId, Id)>>,
    /// All the arguments passed to the app including the app name in 0th
    pub(crate) args: &'a [OsOrStr<'a>],
    /// Current cursor position
    cur: AtomicUsize,
    pub(crate) front: RefCell<Option<Arg<'a>>>,
    /// through this tasks can request event scheduling, etc
    pub(crate) shared: RefCell<VecDeque<Op<'a>>>,

    /// Used to pass information about children exit
    pub(crate) child_exit: Cell<Option<Error>>,

    /// number of ietms consumed by children tasks
    pub(crate) items_consumed: Cell<u32>,

    /// By the end we are trying to kill all the children, when set - children
    /// should fail when encounter unexpected input, otherwise - keep running
    term: AtomicBool,
}

#[derive(Clone)]
#[repr(transparent)]
// this is a newtype instead of struct since things like RawCtx::spawn
// need to pass it by ownership
pub struct Ctx<'a>(Rc<RawCtx<'a>>);

impl<'a> Ctx<'a> {
    pub(crate) fn new(args: &'a [OsOrStr<'a>]) -> Self {
        Ctx(Rc::new(RawCtx {
            args,
            current_task: Default::default(),
            items_consumed: Default::default(),
            shared: Default::default(),
            cur: AtomicUsize::from(0),
            front: Default::default(),
            child_exit: Default::default(),
            term: Default::default(),
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
        P: Parser<T>,
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

    pub(crate) fn add_named_wake(
        &self,
        flag: bool,
        names: &'a [Name<'static>],
        branch: BranchId,
        id: Id,
    ) {
        self.shared.borrow_mut().push_back(Op::AddNamedListener {
            flag,
            names,
            branch,
            id,
        });
    }

    pub(crate) fn add_fallback(&self, branch: BranchId, id: Id) {
        self.shared
            .borrow_mut()
            .push_back(Op::AddFallback { branch, id });
    }

    pub(crate) fn remove_fallback(&self, branch: BranchId, id: Id) {
        self.shared
            .borrow_mut()
            .push_back(Op::RemoveFallback { branch, id });
    }

    pub(crate) fn add_children_exit_listener(&self, parent: Id) {
        self.shared
            .borrow_mut()
            .push_back(Op::AddExitListener { parent });
    }

    pub(crate) fn remove_children_exit_listener(&self, parent: Id) {
        self.shared
            .borrow_mut()
            .push_back(Op::RemoveExitListener { parent });
    }

    pub(crate) fn remove_named_listener(
        &self,
        flag: bool,
        branch: BranchId,
        id: Id,
        names: &'a [Name<'static>],
    ) {
        self.shared.borrow_mut().push_back(Op::RemoveNamedListener {
            flag,
            names,
            branch,
            id,
        });
    }

    pub(crate) fn positional_wake(&self, branch: BranchId, id: Id) {
        self.shared
            .borrow_mut()
            .push_back(Op::AddPositionalListener { branch, id })
    }

    pub(crate) fn start_task(&self, parent: Parent, action: Action<'a>, keep_id: bool) {
        self.shared.borrow_mut().push_back(Op::SpawnTask {
            parent,
            action,
            keep_id,
        });
    }

    pub(crate) fn current_task(&self) -> (BranchId, Id) {
        self.current_task
            .borrow()
            .expect("should only be called from a Future")
    }

    /// Run a task in a context, return number of items consumed an a result
    ///
    /// does not advance the pointer
    pub(crate) fn run_task(&self, task: &mut Task<'a>) -> (Poll<ErrorHandle>, usize) {
        let before = self.cur();
        let mut cx = Context::from_waker(&task.waker);
        let r = task.action.as_mut().poll(&mut cx);
        let after = self.cur();
        self.set_cur(before);
        (r, after - before)
    }
}
