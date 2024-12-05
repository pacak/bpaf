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
    vec,
};

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
        named: BTreeMap::new(),
        pending: Default::default(),
        next_task_id: 0,
        positional: Default::default(),
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

    fn take_name(&self, name: &[Name<'static>]) -> Option<Name<'static>> {
        todo!()
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
        todo!()
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
    named: BTreeMap<Name<'static>, Id>,
    positional: Pecking,
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
                Arg::Named {
                    name: Name::Short(name),
                    val: Some(arg),
                }
            }
            None => {
                let name = short_name.chars().next().unwrap(); // TODO
                Arg::Named {
                    name: Name::Short(name),
                    val: None,
                }
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
                        // start with a dummy branch
                        branch: BranchId::ROOT,
                    };

                    if task.poll(id, &self.ctx).is_pending() {
                        // task is still pending, get the real BranchId
                        // and save it
                        self.family.insert(parent, id);
                        task.branch = self.family.branch_for(id);
                        println!("started {id:?} in {:?}", task.branch);
                        self.tasks.insert(id, task);
                    } else {
                        println!("done already");
                    }
                }
                Op::AddNamedListener { names, waker } => {
                    for name in names.iter() {
                        let id = self.resolve(&waker);
                        self.named.insert(*name, id);
                    }
                }
                Op::AddPositionalListener { waker } => {
                    let id = self.resolve(&waker);
                    let branch = self.family.branch_for(id);
                    self.positional.insert(id, branch);
                    println!("positional: {:?}", self.positional);
                }
                Op::RemoveNamedListener { names, id } => {
                    for name in names {
                        self.named.remove(name);
                    }
                    println!("remove named listener for {names:?} {id:?}");
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

    /// Populate ids with tasks that subscribed for the next token
    fn parsers_for_next_word(&mut self) -> Result<(), Error> {
        println!("currently args are {:?}[{:?}]", self.ctx.args, self.ctx.cur);
        // first we need to decide what parsers to run
        if let Some(front) = self.ctx.args.get(self.ctx.cur()) {
            let name = if let Some(long) = front.strip_prefix("--") {
                Name::Long(long)
            } else if let Some(short) = front.strip_prefix("-") {
                Name::Short(short.chars().next().unwrap())
            } else {
                let x = self.positional.pop_front(&mut self.ids);
                if x == 0 {
                    println!("no positionals");
                    return Err(Error::Invalid);
                }
                return Ok(());
            };
            println!("{:?}", self.named);
            match self.named.get(&name).copied() {
                Some(c) => {
                    println!("waking {c:?} to parse {name:?}");
                    self.ids.push_back(c);
                    Ok(())
                }
                None => {
                    println!(
                        "unknown name - complain {:?} / {:?} / {:?}",
                        name, front, self.named
                    );
                    Err(Error::Invalid)
                }
            }
        } else {
            println!(
                "nothing to parse, time to terminate things {:?}",
                self.named
            );
            Ok(())
        }
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
        let mut par = Vec::new();
        loop {
            self.handle_non_consuming();

            assert!(self.ctx.shared.borrow().is_empty());
            assert!(self.pending.lock().expect("poison").is_empty());

            self.parsers_for_next_word()?;

            if self.ids.is_empty() {
                for id in self.named.values() {
                    println!("waking {id:?} to handle noparse");
                    self.ids.push_back(*id);
                }
                self.named.clear();
                self.positional.drain_to(&mut self.ids);

                if self.ids.is_empty() {
                    break;
                }
            }

            println!("We are going to parse the next workd with {:?}", self.ids);
            // actual feed consumption happens here
            let mut max_consumed = 0;
            while let Some(id) = self.ids.pop_front() {
                // each scheduled task gets a chance to run,
                if let Some(task) = self.tasks.get_mut(&id) {
                    let (poll, consumed) = self.ctx.run_task(task);
                    if let Poll::Ready(r) = poll {
                        if let Some(err) = r {
                            print!("check if parent is interested in this error {err:?}");
                        }
                        self.tasks.remove(&id);
                    }
                    max_consumed = consumed.max(max_consumed);
                    par.push((consumed, id));
                    println!("{id:?} consumed {consumed}!");
                } else {
                    //                    todo!("task was scheduled yet is terminated somehow");
                }
                par.retain(|(len, _id)| *len == max_consumed);
            }

            println!("forest: {:?}", self.family);
            // next task is to go over all the `par` results up to root, mark
            // all the alt branches that are still present in `par` and their
            // parents up to the top most alt branch as safe and
            // terminate all unmarked branches
            self.ctx.advance(max_consumed);
        }
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

/// # Pecking order
///
/// For as long as there's only one task to wake up for the input - it is safe to just
/// wake it up and be done with it, but users are allowed to specify multiple consumers for the
/// same name as well as multiple positional consumers that don't have names at all. This requires
/// deciding which parser gets to run first or gets to run at all.
///
/// Rules for priority are:
///
/// - sum branches run in parallel, left most wins if there's multiple successes
/// - parsers inside a product run sequentially, left most wins
///
/// Therefore we are going to arrange tasks in following order:
/// There's one queue for each branch_id (sum parent id + field), every queue contains
/// items from the same product, so their priority is how far from the left end they are
///
/// "any" parsers get to run for both named and positional input inside their branch
/// accoding to their priority, if at the front. Consider a few queues
/// - `[named, any]` - `any` doesn't run since `named` takes priority
/// - `[any1, named, any2]` - `any1` runs, if it fails to match anything - `named` runs.
/// - `[any1, any2, named]` - `any1` runs, if not - `any2`, if not - `named`
///
/// "any" are mixed with positional items the same way so we'll have to mix them in dynamically...
///
///
/// # Operations needed
///
/// - `Pecking::insert`
/// - `Pecking::select`
/// - `Pecking::remove`?
#[derive(Debug, Default)]
enum Pecking {
    /// No parsers at all, this makes sense for `positional` and `any` items, with
    /// named might as well drop the parser
    #[default]
    Empty,

    /// A single parser
    ///
    /// Usually a unique named argument or a single positional item to the parser
    Single(BranchId, Id),
    /// There's multiple parsers, but they all belong to the same queue
    ///
    /// Several positional items
    Queue(BranchId, VecDeque<Id>),

    /// Multiple alternative branches, VecDeque contains at least one item
    Forest(HashMap<BranchId, VecDeque<Id>>),
}

impl Pecking {
    fn insert(&mut self, id: Id, branch: BranchId) {
        match self {
            Pecking::Empty => *self = Pecking::Single(branch, id),
            Pecking::Single(prev_bi, prev_id) => {
                if *prev_bi == branch {
                    let mut queue = VecDeque::new();
                    queue.push_back(*prev_id);
                    queue.push_back(id);
                    *self = Pecking::Queue(branch, queue)
                } else {
                    let mut forest = HashMap::new();
                    let mut queue = VecDeque::new();
                    queue.push_back(*prev_id);
                    forest.insert(*prev_bi, queue);

                    let mut queue = VecDeque::new();
                    queue.push_back(id);
                    forest.insert(branch, queue);
                    *self = Pecking::Forest(forest)
                }
            }
            Pecking::Queue(prev_bi, vec_deque) => {
                if *prev_bi == branch {
                    vec_deque.push_back(id);
                } else {
                    let mut forest = HashMap::new();
                    forest.insert(*prev_bi, std::mem::take(vec_deque));
                    let mut queue = VecDeque::new();
                    queue.push_back(id);
                    forest.insert(branch, queue);
                    *self = Pecking::Forest(forest);
                }
            }
            Pecking::Forest(forest) => {
                forest.entry(branch).or_default().push_back(id);
            }
        }
    }

    fn pop_front(&mut self, ids: &mut VecDeque<Id>) -> usize {
        match self {
            Pecking::Empty => 0,
            Pecking::Single(branch_id, id) => {
                ids.push_back(*id);
                *self = Pecking::Empty;
                1
            }
            Pecking::Queue(branch_id, vec_deque) => {
                if let Some(f) = vec_deque.pop_front() {
                    ids.push_back(f);
                    1
                } else {
                    0
                }
            }
            Pecking::Forest(hash_map) => {
                let mut cnt = 0;
                for m in hash_map.values_mut() {
                    if let Some(f) = m.pop_front() {
                        ids.push_back(f);
                        cnt += 1;
                    }
                }
                cnt
            }
        }
    }

    fn drain_to(&mut self, ids: &mut VecDeque<Id>) {
        match self {
            Pecking::Empty => {}
            Pecking::Single(branch_id, id) => {
                ids.push_back(*id);
            }
            Pecking::Queue(branch_id, vec_deque) => ids.extend(vec_deque.drain(..)),
            Pecking::Forest(hash_map) => {
                for mut queue in std::mem::take(hash_map).into_values() {
                    ids.extend(queue.drain(..));
                }
            }
        }
        *self = Pecking::Empty;
    }
}

#[cfg(test)]
mod tests;
