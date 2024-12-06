#![allow(unused_imports, dead_code, unused_variables)]
use crate::{long, named::Name, positional};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::{pin, Pin},
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicU32, AtomicUsize},
        Arc, Mutex,
    },
    task::{Context, Poll, Wake, Waker},
};

// # those error messages can be handled
//     /// Tried to consume an env variable which wasn't set
//     can handle
//     NoEnv(&'static str),
//
//     /// User specified an error message on some
//     ParseSome(&'static str),
//
//     /// User asked for parser to fail explicitly
//     ParseFail(&'static str),
//
//     /// pure_with failed to parse a value
//     PureFailed(String),
//
//     /// Expected one of those values
//     ///
//     /// Used internally to generate better error messages
//     Missing(Vec<MissingItem>),
//
// -------------------------------------------------------------------------------------
//     /// Parsing failed and this is the final output
//     ParseFailure(ParseFailure),
//
//     /// Tried to consume a strict positional argument, value was present but was not strictly
//     /// positional
//     StrictPos(usize, Metavar),
//
//     /// Tried to consume a non-strict positional argument, but the value was strict
//     NonStrictPos(usize, Metavar),
//
//     /// Parser provided by user failed to parse a value
//     ParseFailed(Option<usize>, String),
//
//     /// Parser provided by user failed to validate a value
//     GuardFailed(Option<usize>, &'static str),
//
//     /// Argument requres a value but something else was passed,
//     /// required: --foo <BAR>
//     /// given: --foo --bar
//     ///        --foo -- bar
//     ///        --foo
//     NoArgument(usize, Metavar),
//
//     /// Parser is expected to consume all the things from the command line
//     /// this item will contain an index of the unconsumed value
//     Unconsumed(/* TODO - unused? */ usize),
//
//     /// argument is ambigoups - parser can accept it as both a set of flags and a short flag with no =
//     Ambiguity(usize, String),
//
//     /// Suggested fixes for typos or missing input
//     Suggestion(usize, Suggestion),
//
//     /// Two arguments are mutually exclusive
//     /// --release --dev
//     Conflict(/* winner */ usize, usize),
//
//     /// Expected one or more items in the scope, got someting else if any
//     Expected(Vec<Item>, Option<usize>),
//
//     /// Parameter is accepted but only once
//     OnlyOnce(/* winner */ usize, usize),
// }

mod family;
mod futures;

use family::{BranchId, FamilyTree, *};
pub use futures::*;

type Action<'a> = Pin<Box<dyn Future<Output = Option<Error>> + 'a>>;

struct Task<'a> {
    action: Action<'a>,
    branch: BranchId,
    waker: Waker,
}

impl Task<'_> {
    fn poll(&mut self, id: Id, ctx: &Ctx) -> Poll<Option<Error>> {
        *ctx.current_task.borrow_mut() = Some(id);
        let mut cx = Context::from_waker(&self.waker);
        let poll = self.action.as_mut().poll(&mut cx);
        *ctx.current_task.borrow_mut() = None;
        poll
    }
}

pub struct RawCtx<'a> {
    /// Gets populated with current taskid when it is running
    current_task: RefCell<Option<Id>>,
    /// All the arguments passed to the app including the app name in 0th
    args: &'a [String],
    /// Current cursor position
    cur: AtomicUsize,
    /// through this tasks can request event scheduling, etc
    shared: RefCell<VecDeque<Op<'a>>>,
}

#[derive(Clone)]
#[repr(transparent)]
// this is a newtype instead of struct since things like RawCtx::spawn
// need to pass it by ownership
pub struct Ctx<'a>(Rc<RawCtx<'a>>);

impl<'a> std::ops::Deref for Ctx<'a> {
    type Target = RawCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum Op<'a> {
    SpawnTask {
        parent: Parent,
        action: Action<'a>,
    },
    WakeTask {
        id: Id,
    },
    KillTask {
        id: Id,
    },
    AddNamedListener {
        names: &'a [Name<'static>],
        waker: Waker,
    },
    RemoveNamedListener {
        names: &'a [Name<'static>],
        id: Id,
    },
    AddPositionalListener {
        waker: Waker,
    },
}

fn parse_args<T>(parser: &impl Parser<T>, args: &[String]) -> Result<T, Error>
where
    T: 'static + std::fmt::Debug,
{
    let ctx = Ctx(Rc::new(RawCtx {
        args,
        current_task: Default::default(),
        shared: Default::default(),
        cur: AtomicUsize::from(0),
    }));

    let runner = Runner {
        ctx,
        ids: Default::default(),
        tasks: BTreeMap::new(),
        pending: Default::default(),
        next_task_id: 0,
        family: Default::default(),
    };
    runner.run_parser(parser)
}

impl<'a> Ctx<'a> {
    fn spawn<T, P>(&self, parent: Parent, parser: &'a P) -> JoinHandle<'a, T>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
        let ctx = self.clone();
        let (exit, join) = self.fork();
        let act = Box::pin(async move {
            exit.id.set(ctx.current_task());
            let r = parser.run(ctx).await;
            let out = r.as_ref().err().cloned();
            if let Ok(exit) = Rc::try_unwrap(exit) {
                exit.exit_task(r);
            }
            out
        });
        self.start_task(parent, act);
        join
    }

    ///
    ///
    /// this needs name to know what to look for, waker since that's how futures work....
    /// Or do they...
    /// But I also need an ID so I can start placing items into priority forest
    fn named_wake(&self, names: &'a [Name<'static>], waker: Waker) {
        self.shared
            .borrow_mut()
            .push_back(Op::AddNamedListener { names, waker });
    }

    fn positional_wake(&self, waker: Waker) {
        self.shared
            .borrow_mut()
            .push_back(Op::AddPositionalListener { waker })
    }

    fn start_task(&self, parent: Parent, action: Action<'a>) {
        self.shared
            .borrow_mut()
            .push_back(Op::SpawnTask { parent, action });
    }

    fn current_task(&self) -> Option<Id> {
        *self.current_task.borrow()
    }

    /// Run a task in a context, return number of items consumed an a result
    ///
    /// does not advance the pointer
    fn run_task(&self, task: &mut Task<'a>) -> (Poll<Option<Error>>, usize) {
        let before = self.cur();
        let mut cx = Context::from_waker(&task.waker);
        let r = task.action.as_mut().poll(&mut cx);
        let after = self.cur();
        self.set_cur(before);
        (r, after - before)
    }
}

impl<A, B, RA, RB> Parser<(RA, RB)> for (A, B)
where
    A: Parser<RA>,
    B: Parser<RB>,
    RA: 'static + std::fmt::Debug,
    RB: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, (RA, RB)> {
        todo!()
    }
}

impl<T> Parser<T> for Vec<Box<dyn Parser<T>>>
where
    T: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async {
            todo!();
            //
        })
    }
}

pub type Fragment<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;
pub trait Parser<T: 'static + std::fmt::Debug> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T>;
    fn into_box(self) -> Box<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn into_rc(self) -> Rc<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    Missing,
    Invalid,
    /// Low priority error that gets created when a branch gets killed
    /// to allow more successful alternative to run.
    /// At least one branch in the sum
    Killed,
}
impl Error {
    fn combine_with(self, e2: Error) -> Error {
        match (self, e2) {
            (e @ Error::Invalid, _) | (_, e @ Error::Invalid) => e,
            (e, Error::Killed) | (Error::Killed, e) => e,
            (Error::Missing, Error::Missing) => Error::Missing,
        }
    }
    fn can_handle(&self) -> bool {
        match self {
            Error::Missing => true,
            Error::Invalid => false,
            Error::Killed => todo!(),
        }
    }
}

#[derive(Clone)]
struct Pair<A, B>(A, B);
impl<A, B, RA, RB> Parser<(RA, RB)> for Pair<A, B>
where
    A: Parser<RA>,
    B: Parser<RB>,
    RA: 'static + std::fmt::Debug,
    RB: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, (RA, RB)> {
        Box::pin(async move {
            let id = ctx.current_id();
            let futa = ctx.spawn(
                Parent {
                    id,
                    field: 0,
                    kind: NodeKind::Prod,
                },
                &self.0,
            );
            let futb = ctx.spawn(
                Parent {
                    id,
                    field: 1,
                    kind: NodeKind::Prod,
                },
                &self.1,
            );
            Ok((futa.await?, futb.await?))
        })
    }
}

#[derive(Clone)]
struct Many<P>(P);
impl<T, P> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
    T: std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, Vec<T>> {
        let mut res = Vec::new();
        Box::pin(async move {
            let id = ctx.current_id();
            let parent = Parent {
                id,
                field: 0,
                kind: NodeKind::Sum,
            };

            loop {
                match ctx.spawn(parent, &self.0).await {
                    Ok(t) => res.push(t),
                    Err(Error::Missing) => return Ok(res),
                    Err(e) => return Err(e),
                }
            }
        })
    }
}

#[derive(Clone)]
struct Map<P, F, T>(P, F, PhantomData<T>);
impl<T: 'static, R, F, P> Parser<R> for Map<P, F, T>
where
    P: Parser<T>,
    T: std::fmt::Debug + Clone,
    R: std::fmt::Debug + 'static,
    F: Fn(T) -> R + 'static + Clone,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, R> {
        Box::pin(async move {
            let t = self.0.run(ctx).await?;
            Ok((self.1)(t))
        })
    }
}

struct WakeTask {
    id: Id,
    pending: Arc<Mutex<Vec<Id>>>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        println!("Waking up {:?}", self.id);
        self.pending.lock().expect("poison").push(self.id);
    }
}

struct Runner<'ctx> {
    next_task_id: u32,
    ctx: Ctx<'ctx>,
    ids: VecDeque<Id>,
    tasks: BTreeMap<Id, Task<'ctx>>,

    family: FamilyTree,

    /// Shared with Wakers,
    ///
    /// contains a vector [`Id`] for tasks to wake up.
    pending: Arc<Mutex<Vec<Id>>>,
}

enum Arg<'a> {
    Named {
        name: Name<'a>,
        val: Option<&'a str>,
    },
    ShortSet {
        names: Vec<char>,
    },
    Positional {
        value: &'a str,
    },
}

fn spa(value: &str) -> Arg {
    todo!()
}

fn split_param(value: &str) -> Arg {
    if let Some(long_name) = value.strip_prefix("--") {
        match long_name.split_once('=') {
            Some((name, arg)) => Arg::Named {
                name: Name::Long(name),
                val: Some(arg),
            },
            None => Arg::Named {
                name: Name::Long(long_name),
                val: None,
            },
        }
    } else if let Some(short_name) = value.strip_prefix("-") {
        match short_name.split_once('=') {
            Some((name, arg)) => {
                let name = name.chars().next().unwrap(); // TODO
                todo!();
                // Arg::Named {
                //     name: Name::Short(name),
                //     val: Some(arg),
                // }
            }
            None => {
                let name = short_name.chars().next().unwrap(); // TODO
                todo!();
                // Arg::Named {
                //     name: Name::Short(name),
                //     val: None,
                // }
            }
        }
    } else {
        Arg::Positional { value }
    }
}

impl<'a> Runner<'a> {
    /// Get a task ID for the waker
    fn resolve(&self, waker: &Waker) -> Id {
        waker.wake_by_ref();
        self.pending
            .lock()
            .expect("poison")
            .pop()
            .expect("Misbehaving waker")
    }

    /// Looks for things requested by wakers and schedules it to ops
    fn wakers_to_ops(&self) {
        self.ctx.shared.borrow_mut().extend(
            self.pending
                .lock()
                .expect("poison")
                .drain(..)
                .map(|id| Op::WakeTask { id }),
        )
    }

    /// Handle scheduled operations
    ///
    /// This should advance all the tasks as far as possible without consuming the input
    fn handle_non_consuming(&mut self) {
        let before = self.ctx.cur();
        loop {
            let mut shared = self.ctx.shared.borrow_mut();
            let Some(item) = shared.pop_front() else {
                shared.extend(
                    self.pending
                        .lock()
                        .expect("poison")
                        .drain(..)
                        .map(|id| Op::WakeTask { id }),
                );
                if shared.is_empty() {
                    assert_eq!(before, self.ctx.cur());
                    return;
                } else {
                    continue;
                }
            };
            // tasks are going to borrow from shared when running
            drop(shared);

            match item {
                Op::SpawnTask { parent, action } => {
                    let (id, waker) = self.next_id();

                    let mut task = Task {
                        action,
                        waker,
                        branch: BranchId::DUMMY, // start with a dummy branch
                    };

                    if task.poll(id, &self.ctx).is_pending() {
                        // task is still pending, get the real BranchId to save it
                        task.branch = self.family.insert(parent, id);
                        self.tasks.insert(id, task);
                    } else {
                        println!("done already");
                    }
                }
                Op::AddNamedListener { names, waker } => {
                    let id = self.resolve(&waker);
                    println!("named add for {id:?}");
                    self.family.add_named(id, names);
                }
                Op::AddPositionalListener { waker } => {
                    let id = self.resolve(&waker);
                    self.family.add_positional(id);
                }
                Op::RemoveNamedListener { names, id } => {
                    self.family.remove_named(id, names);
                }
                Op::KillTask { id } => {
                    self.tasks.remove(&id);
                    println!("kill task {id:?}");
                }
                Op::WakeTask { id } => {
                    let Some(task) = self.tasks.get_mut(&id) else {
                        println!("waking up removed task {id:?}");
                        continue;
                    };

                    if task.poll(id, &self.ctx).is_ready() {
                        self.tasks.remove(&id);
                    }
                }
            }
        }
    }

    fn run_scheduled(&mut self) -> bool {
        let changes = self.ids.is_empty();
        for id in self.ids.drain(..) {
            if let Some(task) = self.tasks.get_mut(&id) {
                if let Poll::Ready(res) = task.poll(id, &self.ctx) {
                    if let Some(err) = res {
                        println!("task failed, see if parent cares");
                    }
                    println!("Task {id:?} is done, dropping it");
                    self.tasks.remove(&id);
                }
            }
        }
        !changes
    }

    fn run_parser<P, T>(mut self, parser: &'a P) -> Result<T, Error>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
        let (root_id, root_waker) = self.next_id();

        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead.
        let mut handle = pin!(self.ctx.spawn(root_id.prod(0), parser));

        // poll root handle once so whatever needs to be
        // register - gets a chance to do so then
        // set it aside until all child tasks are satisfied
        let mut root_cx = Context::from_waker(&root_waker);
        // root spawns a child task - it can't return until
        // child task(s) finish - it won't happen until later
        assert!(handle.as_mut().poll(&mut root_cx).is_pending());

        // After this point progress is separated in two distinct parts:
        //
        // - First we repeatedly handle all the pending request until there's
        //   none left
        //
        // - then we pick one or more tasks to wake up to parse
        //   the prefix of the output and run them in parallel.
        //
        //   Tasks that consume the most - keep running, the rest
        //   gets terminated since they belong to alt branches
        //   that couldn't consume everything.
        let mut par = VecDeque::new();
        loop {
            self.handle_non_consuming();

            // need to generate errors for:
            // - conflict
            // - no parse
            //

            assert!(self.ctx.shared.borrow().is_empty());
            assert!(self.pending.lock().expect("poison").is_empty());

            let Some(front) = self.ctx.args.get(self.ctx.cur()) else {
                println!("nothing to consume");
                break;
            };
            // check how to parse next word
            self.family.pick_parsers_for(front, &mut par)?;

            // if we don't know how to parse it - prepare an error using
            // available names,
            if par.is_empty() {
                panic!("If there are unconsumed items ({front:?}) but no parsers");
                // for id in self.named.values() {
                //     println!("waking {id:?} to handle noparse");
                //     self.ids.push_back(*id);
                // }
                // self.named.clear();
                // self.positional.drain_to(&mut self.ids);
                //
                // if self.ids.is_empty() {
                //     break;
                // }
            }

            println!("We are going to parse the next workd with {:?}", self.ids);
            // actual feed consumption happens here
            let mut max_consumed = 0;
            for (id, t) in par.iter_mut() {
                // each scheduled task gets a chance to run,
                if let Some(task) = self.tasks.get_mut(id) {
                    let (poll, consumed) = self.ctx.run_task(task);
                    *t = consumed;
                    if let Poll::Ready(r) = poll {
                        if let Some(err) = r {
                            print!("check if parent is interested in this error {err:?}");
                        }
                        self.tasks.remove(id);
                    }
                    max_consumed = consumed.max(max_consumed);

                    println!("{id:?} consumed {consumed}!");
                } else {
                    println!("family gave us terminated parser {id:?} for {front:?}");
                }
            }

            par.retain(|(_id, len)| *len == max_consumed);

            println!("forest: {:?}", self.family);
            // next task is to go over all the `par` results up to root, mark
            // all the alt branches that are still present in `par` and their
            // parents up to the top most alt branch as safe and
            // terminate all unmarked branches
            self.ctx.advance(max_consumed);
        }

        // at this point there's nothing left to consume, let's run existing tasks to completion
        // first by waking them up all the consuming events (most of them fail with "not found")
        // and then by processing all the non-consuming events - this would either create some
        // errors or or those "not found" errors are going to be converted into something useful
        let mut shared = self.ctx.shared.borrow_mut();
        for id in self.tasks.keys().copied() {
            shared.push_back(Op::WakeTask { id });
        }
        drop(shared);
        self.handle_non_consuming();

        match handle.as_mut().poll(&mut root_cx) {
            Poll::Ready(r) => r,
            Poll::Pending => panic!("process is complete but somehow we don't have a result o_O"),
        }
    }

    /// Allocate next task [`Id`] and a [`Waker`] for that task.
    fn next_id(&mut self) -> (Id, Waker) {
        let id = self.next_task_id;
        self.next_task_id += 1;
        let id = Id::new(id);
        let pending = self.pending.clone();
        let waker = Waker::from(Arc::new(WakeTask { id, pending }));
        (id, waker)
    }
}

impl RawCtx<'_> {
    fn current_id(&self) -> Id {
        self.current_task.borrow().expect("not in a task")
    }
    fn cur(&self) -> usize {
        self.cur.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn set_cur(&self, new: usize) {
        self.cur.store(new, std::sync::atomic::Ordering::Relaxed);
    }
    fn advance(&self, inc: usize) {
        self.cur
            .fetch_add(inc, std::sync::atomic::Ordering::Relaxed);
    }
}

struct Optional<P> {
    inner: P,
}

impl<P, T> Parser<Option<T>> for Optional<P>
where
    P: Parser<T>,
    T: std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, Option<T>> {
        Box::pin(async {
            match self.inner.run(ctx).await {
                Ok(ok) => Ok(Some(ok)),
                Err(e) if e.can_handle() => Ok(None),
                Err(e) => Err(e),
            }
        })
    }
}

struct Alt<T: Clone + 'static> {
    items: Vec<Box<dyn Parser<T>>>,
}

impl<T> Parser<T> for Box<dyn Parser<T>>
where
    T: std::fmt::Debug + Clone + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.as_ref().run(ctx)
    }
}

struct AltFuture<'a, T> {
    handles: Vec<JoinHandle<'a, T>>,
}

impl<T: std::fmt::Debug> Future for AltFuture<'_, T> {
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for (ix, mut h) in self.as_mut().handles.iter_mut().enumerate() {
            if let Poll::Ready(r) = pin!(h).poll(cx) {
                self.handles.remove(ix);
                println!("Got result out!!!!!!!!!!!!!!!!!!!!!!!!!!!! {r:?}");
                return Poll::Ready(r);
            }
        }
        Poll::Pending
    }
}

impl<T> Parser<T> for Alt<T>
where
    T: Clone + std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let id = ctx.current_id();

            // Spawn a task for all the branches
            let handles = self
                .items
                .iter()
                .enumerate()
                .map(|(ix, p)| ctx.spawn(id.sum(ix as u32), p))
                .collect::<Vec<_>>();

            let mut fut = AltFuture { handles };
            // TODO: this should be some low priority error
            let mut res = Err(Error::Killed);

            // must collect results as they come. If

            // return first succesful result or the best error
            while !fut.handles.is_empty() {
                let hh = (&mut fut).await;
                res = match (res, hh) {
                    (ok @ Ok(_), _) | (Err(_), ok @ Ok(_)) => return ok,
                    (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
                }
            }
            res
        })
    }
}

struct ChildErrors {
    id: Id,
}

impl Future for ChildErrors {
    type Output = Option<(/* field */ u32, Error)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

// For every Sum, as soon as we start making any progress with any branch, no matter how deep - we
// must terminate all branches that don't make progress
//
// ways to imlement it:
// - user land and executor - user land subscribes to advances and reports upstream, deals with
//   child termination
//   + the same logic can be reused in many-unadjacent
//   - lots of back and forth across the boundary
//
// - executor only with "sets" - propagates and terminates tasks... mark'n'sweep?
//   every child that finishes marks child node of a sum parent up to the top most branch
//   then second pass goes though marked nodes,
//   removes marks and kills children without marks
//   + simple
//   - many-unadjacent is still an open question...
//
// - every sum task has some notion of cursor position, children making progress
//
//
// For every Prod positional items go sequentially - just put them in a vector
// Every sibling of an alt can consume a separate instance ... positional
// items go into a set of queues keyed by (Id, i32) and we can run one positional item from each
// queue :)

struct PosPrio {
    prio: HashMap<Parent, VecDeque<Id>>,
}

// several named items with the same name in a product
// go sequentially

// For 'Any' - same idea as PosPrio, they just get to

#[cfg(test)]
mod tests;
