use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize},
};

use crate::{
    executor::{futures::JoinHandle, Id, IdStrat, Op, RawAction, Trigger},
    parsers::HelpWrap,
    split::OsOrStr,
    Parser,
};

// TODO - remove some items:
// - front_value and term - pass values to poll directly
// - replace cur with Cell<u32>

pub struct RawCtx<'a> {
    /// Gets populated with current taskid when it is running
    pub(crate) current_task: RefCell<Option<Id>>,
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
    pub fn current_id(&self) -> Id {
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
    pub fn spawn<T, P>(&self, parent: Id, strat: IdStrat, parser: &'a P) -> JoinHandle<'a, T>
    where
        P: Parser<T> + ?Sized,
        T: 'static,
    {
        let ctx = self.clone();
        let (exit, join) = self.fork();
        let act = Box::pin(async move {
            exit.id.set(Some(ctx.current_task()));
            let r = parser.run(ctx).await;

            if let Ok(exit) = Rc::try_unwrap(exit) {
                // This is used to handle erors in Con or similar in execution order
                // instead of definition order with `EarlyExitFut`
                exit.exit_task(r)
            } else {
                unreachable!("We have more than one copy of the exit handle!")
            }
        });
        self.start_task(parent, strat, act);
        join
    }

    pub(crate) fn start_trigger(&self, action: Trigger<'a>) {
        let parent = self.current_id();
        self.queue(Op::SpawnTrigger { parent, action });
    }

    fn queue(&self, op: Op<'a>) {
        self.shared.borrow_mut().push_back(op);
    }

    pub(crate) fn add_fallback(&self, id: Id) {
        self.queue(Op::AddFallback { id });
    }

    pub(crate) fn remove_fallback(&self, id: Id) {
        self.queue(Op::RemoveFallback { id });
    }

    pub(crate) fn start_task(&self, parent: Id, strat: IdStrat, action: RawAction<'a>) {
        self.queue(Op::SpawnTask {
            parent,
            action,
            strat,
        });
    }

    pub(crate) fn current_task(&self) -> Id {
        self.current_task
            .borrow()
            .expect("should only be called from a Future")
    }
}
