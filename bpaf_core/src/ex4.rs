#![allow(unused_imports, dead_code, unused_variables)]
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet},
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

use crate::{long, named::Name, positional};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Id(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]

enum NodeKind {
    Sum,
    Prod,
}

impl Id {
    fn sum(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Prod,
            id: self,
            field,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Parent {
    kind: NodeKind,
    id: Id,
    field: u32,
}
impl Parent {
    fn new(id: Id, field: u32, kind: NodeKind) -> Self {
        Self { id, field, kind }
    }
}

struct Task<'a> {
    act: Pin<Box<dyn Future<Output = Option<Error>> + 'a>>,
}

pub struct RawCtx<'a> {
    current_task: RefCell<Option<Id>>,
    /// All the arguments passed to the app including the app name in 0th
    args: &'a [String],
    /// Current cursor position
    cur: AtomicUsize,
    /// through this tasks can request event scheduling, etc
    shared: RefCell<Pending<'a>>,
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

#[derive(Default)]
struct Pending<'a> {
    spawn: Vec<(Parent, Task<'a>)>,
    named: Vec<(&'a [Name<'static>], Waker)>,
    positional: Vec<Waker>,
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
        private: Default::default(),
        next_task_id: 0,
        positional: Vec::new(),
    };
    runner.run_parser(parser)
}

fn fork<T>() -> (Rc<ExitHandle<T>>, JoinHandle<T>) {
    let result = Rc::new(Cell::new(None));
    let exit = ExitHandle {
        waker: Cell::new(None),
        result: result.clone(),
    };
    let exit = Rc::new(exit);
    let join = JoinHandle {
        task: Rc::downgrade(&exit),
        result,
    };
    (exit, join)
}

impl<T> Drop for ExitHandle<T> {
    fn drop(&mut self) {
        let Some(waker) = self.waker.take() else {
            return;
        };
        println!("dropped handle  ");
    }
}

struct ExitHandle<T> {
    waker: Cell<Option<Waker>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

struct JoinHandle<T> {
    task: Weak<ExitHandle<T>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
}
impl<T: std::fmt::Debug> ExitHandle<T> {
    fn exit_task(self, result: Result<T, Error>) {
        println!("setting result to {result:?}");
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}
impl<T> Future for JoinHandle<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.task.upgrade() {
            Some(task) => {
                task.waker.set(Some(cx.waker().clone()));
                Poll::Pending
            }
            None => {
                println!("Getting result out!");

                Poll::Ready(self.result.take().expect("Task exit sets result"))
            }
        }
    }
}

impl<'a> Ctx<'a> {
    fn spawn<T, P>(&self, parent: Parent, parser: &'a P) -> JoinHandle<T>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
        println!("spawning {parent:?}");
        let ctx = self.clone();
        let (exit, join) = fork();
        let act = Box::pin(async move {
            println!("Waiting on spawned task");
            let r = parser.run(ctx).await;
            println!("we a got a result {r:?}");
            let out = r.as_ref().err().cloned();
            if let Ok(exit) = Rc::try_unwrap(exit) {
                exit.exit_task(r);
            }
            out
        });
        self.start_task(parent, Task { act });
        join
    }

    ///
    ///
    /// this needs name to know what to look for, waker since that's how futures work....
    /// Or do they...
    /// But I also need an ID so I can start placing items into priority forest
    fn named_wake(&self, name: &'a [Name<'static>], waker: Waker) {
        self.shared.borrow_mut().named.push((name, waker));
    }

    fn positional_wake(&self, waker: Waker) {
        self.shared.borrow_mut().positional.push(waker)
    }

    fn take_name(&self, name: &[Name<'static>]) -> Option<Name<'static>> {
        todo!()
    }

    fn start_task(&self, parent: Parent, task: Task<'a>) {
        println!("starting a task");

        self.shared.borrow_mut().spawn.push((parent, task));
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

pub(crate) struct PositionalFut<'a> {
    pub(crate) ctx: Ctx<'a>,
    pub(crate) registered: bool,
}

impl<'ctx> Future for PositionalFut<'ctx> {
    type Output = Result<&'ctx str, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx.positional_wake(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(match self.ctx.args.get(self.ctx.cur()) {
            Some(s) if s.starts_with('-') => Err(Error::Invalid),
            Some(s) => {
                self.ctx.advance(1);
                Ok(s.as_str())
            }
            None => Err(Error::Missing),
        })
    }
}

pub(crate) struct NamedFut<'a> {
    pub(crate) name: &'a [Name<'static>],
    pub(crate) ctx: Ctx<'a>,
    pub(crate) registered: bool,
}

impl Future for NamedFut<'_> {
    type Output = Result<Name<'static>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx.named_wake(self.name, cx.waker().clone());
            return Poll::Pending;
        }

        let Some(front) = self.ctx.args.get(self.ctx.cur()) else {
            return Poll::Ready(Err(Error::Missing));
        };

        Poll::Ready(match split_param(front) {
            Arg::Named { name, val } => {
                let r = self
                    .name
                    .iter()
                    .copied()
                    .find(|n| *n == name)
                    .ok_or(Error::Missing);
                if r.is_ok() {
                    self.ctx.advance(1);
                }

                r
            }
            Arg::ShortSet { .. } | Arg::Positional { .. } => Err(Error::Invalid),
        })
    }
}

struct DummyWaker;

impl Wake for DummyWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

struct WakeTask {
    id: Id,
    pending: Arc<Mutex<Vec<Id>>>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        println!("will try to wake up {:?}", self.id);
        self.pending.lock().expect("poison").push(self.id);
    }
}

struct Runner<'ctx> {
    next_task_id: u32,
    ctx: Ctx<'ctx>,
    /// A private copy of ctx
    private: RefCell<Pending<'ctx>>,
    ids: HashSet<Id>,
    tasks: BTreeMap<Id, (Task<'ctx>, Waker)>,
    named: BTreeMap<Name<'static>, Id>,
    positional: Vec<Id>,

    /// id to wake up
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
    fn waker_for(&self, id: Id) -> Waker {
        Waker::from(Arc::new(WakeTask {
            id,
            pending: self.pending.clone(),
        }))
    }

    fn resolve(&self, waker: &Waker) -> Id {
        waker.wake_by_ref();
        self.pending
            .lock()
            .expect("poision")
            .pop()
            .expect("Misbehaving waker")
    }

    fn handle_shared(&mut self) -> bool {
        let mut changed = false;
        self.private.swap(&self.ctx.shared);
        let mut shared = self.private.borrow_mut();
        for (parent, mut task) in shared.spawn.drain(..) {
            let id = Id(self.next_task_id);
            self.next_task_id += 1;
            let waker = self.waker_for(id);
            changed = true;

            let mut cx = Context::from_waker(&waker);
            println!("Polling freshly spawned {id:?}");
            *self.ctx.current_task.borrow_mut() = Some(id);
            if task.act.as_mut().poll(&mut cx).is_pending() {
                self.tasks.insert(id, (task, waker));
            } else {
                println!("done already");
            }
            *self.ctx.current_task.borrow_mut() = None;
        }

        for (names, waker) in shared.named.drain(..) {
            changed = true;
            for name in names.iter() {
                let id = self.resolve(&waker);
                self.named.insert(*name, id);
            }
        }
        for waker in shared.positional.drain(..) {
            changed = true;
            let id = self.resolve(&waker);
            self.positional.push(id);
        }
        changed
    }

    fn schedule_pending(&mut self) {
        self.ids
            .extend(self.pending.lock().expect("poison").drain(..));
    }
    fn run_scheduled(&mut self) -> bool {
        let changes = self.ids.is_empty();
        for id in self.ids.drain() {
            if let Some((task, waker)) = self.tasks.get_mut(&id) {
                let mut cx = Context::from_waker(waker);
                *self.ctx.current_task.borrow_mut() = Some(id);
                if task.act.as_mut().poll(&mut cx).is_ready() {
                    println!("Task {id:?} is done");
                    self.tasks.remove(&id);
                }
                *self.ctx.current_task.borrow_mut() = None;
            }
        }
        !changes
    }

    fn parsers_for_next_word(&mut self) {
        println!("currently args are {:?}[{:?}]", self.ctx.args, self.ctx.cur);
        // first we need to decide what parsers to run
        if let Some(front) = self.ctx.args.get(self.ctx.cur()) {
            let name = if let Some(long) = front.strip_prefix("--") {
                Name::Long(long)
            } else if let Some(short) = front.strip_prefix("-") {
                Name::Short(short.chars().next().unwrap())
            } else {
                // TODO - make sure positional items are actually present
                self.ids.insert(self.positional.remove(0));
                return;
            };
            println!("{:?}", self.named);
            match self.named.get(&name).copied() {
                Some(c) => {
                    println!("waking {c:?} to parse {name:?}");
                    self.ids.insert(c);
                }
                None => todo!(
                    "unknown name - complain {:?} / {:?} / {:?}",
                    name,
                    front,
                    self.named
                ),
            }
        } else {
            println!(
                "nothing to parse, time to terminate things {:?}",
                self.named
            );
        }
    }

    fn run_parser<P, T>(mut self, parser: &'a P) -> Result<T, Error>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
        let root_id = self.next_id();

        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead.
        let mut handle = pin!(self
            .ctx
            .spawn(Parent::new(root_id, 0, NodeKind::Prod), parser));
        let root_waker = self.waker_for(root_id);

        // poll root handle once so whatever needs to be
        // register - gets a chance to do so then
        // set it aside until all child tasks are satisfied
        let mut root_cx = Context::from_waker(&root_waker);
        if let Poll::Ready(r) = handle.as_mut().poll(&mut root_cx) {
            assert_eq!(self.ctx.cur(), self.ctx.args.len());
            return r;
        }

        let mut par = Vec::new();
        loop {
            // first we wake spawn all the pending tasks and poll them to
            // make sure things propagate. this might take several loops
            loop {
                while self.handle_shared() {
                    self.schedule_pending();
                }
                if !self.run_scheduled() {
                    break;
                }
            }

            self.schedule_pending();
            self.parsers_for_next_word();

            if self.ids.is_empty() {
                for id in self.named.values() {
                    println!("waking {id:?} to handle noparse");
                    self.ids.insert(*id);
                }
                self.named.clear();
                self.ids.extend(self.positional.drain(..));

                if self.ids.is_empty() {
                    break;
                }
            }

            println!("We are going to parse the next workd with {:?}", self.ids);
            // actual feed consumption happens here
            let mut max_consumed = 0;
            for id in self.ids.drain() {
                if let Some((task, waker)) = self.tasks.get_mut(&id) {
                    let before = self.ctx.cur();
                    let mut cx = Context::from_waker(waker);
                    if task.act.as_mut().poll(&mut cx).is_ready() {
                        println!("task {id:?} is done from parse");
                        self.tasks.remove(&id);
                    }
                    let after = self.ctx.cur();
                    self.ctx.set_cur(before);
                    let consumed = after - before;
                    max_consumed = consumed.max(max_consumed);
                    par.push((consumed, id));
                    println!("{id:?} consumed {consumed}!");
                }
                par.retain(|(len, _id)| *len == max_consumed);
            }

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

    fn next_id(&mut self) -> Id {
        let id = self.next_task_id;
        self.next_task_id += 1;
        Id(id)
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

#[cfg(test)]
mod tests;

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

            // TODO: this should be some low priority error
            let mut res = Err(Error::Killed);

            // return first succesful result or the best error
            for h in handles {
                res = match (res, h.await) {
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

// what do I need from the forest?
// positional items need to go to priority forest
type Forest = HashMap<Id, Node>;
#[derive(Debug)]
struct Node {
    ty: NodeKind,
    children: Vec<(Id, /* field */ u32)>,
}
