#![allow(unused_imports, dead_code, unused_variables)]
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    marker::PhantomData,
    pin::{pin, Pin},
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Id(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]

enum ParentKind {
    Sum,
    Prod,
    Root,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Parent {
    kind: ParentKind,
    id: Id,
    field: u32,
}
impl Parent {
    fn new(id: Id, field: u32, kind: ParentKind) -> Self {
        Self { id, field, kind }
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Name<'a> {
    Short(char),
    Long(&'a str),
}

struct Task<'a> {
    parent: Parent,
    act: Pin<Box<dyn Future<Output = Option<Error>> + 'a>>,
}

#[derive(Debug, Clone)]
struct Args {
    all: Rc<[String]>,
    cur: usize,
}

#[derive(Clone)]
pub struct Ctx<'a> {
    data: Rc<RefCell<RawCtx>>,
    spawn: Arc<Mutex<Pending<'a>>>,
    pending: Arc<Mutex<Vec<Id>>>,
}

#[derive(Default)]
struct Pending<'a> {
    spawn: Vec<(Id, Task<'a>, Waker)>,
    name: Vec<(Rc<[Name<'static>]>, Id)>,
}
fn parse_args<T>(parser: impl Parser<T>, args: &[String]) -> Result<T, Error>
where
    T: 'static + std::fmt::Debug,
{
    let ctx = RefCell::new(RawCtx {
        next_id: 0,
        args: Args {
            all: Rc::from(args),
            cur: 0,
        },
    });
    let ctx = Ctx {
        data: Rc::new(ctx),
        spawn: Default::default(),
        pending: Default::default(),
    };

    let runner = Runner {
        ctx,
        tasks: BTreeMap::new(),
        named: BTreeMap::new(),
    };
    runner.run_parser(&parser)
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
    result: Rc<Cell<Option<T>>>,
}

struct JoinHandle<T> {
    task: Weak<ExitHandle<T>>,
    result: Rc<Cell<Option<T>>>,
}
impl<T: std::fmt::Debug> ExitHandle<T> {
    fn exit_task(self, result: T) {
        println!("setting result to {result:?}");
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}
impl<T> Future for JoinHandle<T> {
    type Output = T;

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
    fn spawn<T, P>(&self, parent: Parent, parser: &'a P) -> JoinHandle<Result<T, Error>>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
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
        self.start_task(Task { parent, act });
        join
    }

    ///
    ///
    /// this needs name to know what to look for, waker since that's how futures work....
    /// Or do they...
    /// But I also need an ID so I can start placing items into priority forest
    fn named_wake(&self, id: Id, name: Rc<[Name<'static>]>, waker: Waker) {
        let mut ctx = self.spawn.lock().expect("poison");
        ctx.name.push((name, id));
    }

    fn take_name(&self, name: &[Name<'static>]) -> Option<Name<'static>> {
        todo!()
    }

    fn peek_next_id(&self) -> Id {
        Id(self.data.borrow().next_id)
    }
    fn next_id(&self) -> Id {
        let mut ctx = self.data.borrow_mut();
        let id = ctx.next_id;
        ctx.next_id += 1;
        Id(id)
    }

    fn start_task(&self, task: Task<'a>) {
        let id = self.next_id();
        let waker = self.waker_for(id);
        self.spawn
            .lock()
            .expect("poison")
            .spawn
            .push((id, task, waker));
        self.pending.lock().expect("poision").push(id);
    }

    fn waker_for(&self, id: Id) -> Waker {
        Waker::from(Arc::new(WakeTask {
            id,
            pending: self.pending.clone(),
        }))
    }
}

struct RawCtx {
    next_id: u32,
    args: Args,
}

impl<A, B, RA, RB> Parser<(RA, RB)> for (A, B)
where
    A: Parser<RA>,
    B: Parser<RB>,
    RA: 'static + std::fmt::Debug,
    RB: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, (RA, RB)> {
        todo!()
    }
}

impl<T> Parser<T> for Vec<Box<dyn Parser<T>>>
where
    T: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T> {
        todo!()
    }
}

type BoxedFrag<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;
pub trait Parser<T: 'static + std::fmt::Debug> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T>;
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Missing,
    Invalid,
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
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, (RA, RB)> {
        Box::pin(async move {
            let id = ctx.next_id();
            let futa = ctx.spawn(
                Parent {
                    id,
                    field: 0,
                    kind: ParentKind::Prod,
                },
                &self.0,
            );
            let futb = ctx.spawn(
                Parent {
                    id,
                    field: 1,
                    kind: ParentKind::Prod,
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
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, Vec<T>> {
        let id = ctx.next_id();

        let mut res = Vec::new();
        let parent = Parent {
            id,
            field: 0,
            kind: ParentKind::Sum,
        };
        Box::pin(async move {
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
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, R> {
        Box::pin(async move {
            let t = self.0.run(ctx).await?;
            Ok((self.1)(t))
        })
    }
}

struct NamedFut<'a> {
    name: Rc<[Name<'static>]>,
    ctx: Ctx<'a>,
    registered: bool,
}

impl Future for NamedFut<'_> {
    type Output = Result<Name<'static>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx
                .named_wake(Id(0), self.name.clone(), cx.waker().clone());
            return Poll::Pending;
        }

        let mut data = self.ctx.data.borrow_mut();
        let Some(front) = data.args.all.get(data.args.cur) else {
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
                    data.args.cur += 1;
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
    ctx: Ctx<'ctx>,
    tasks: BTreeMap<Id, (Task<'ctx>, Waker)>,
    named: BTreeMap<Name<'static>, Id>,
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
    fn tasks_to_run(&self, ids: &mut Vec<Id>) {
        // first we need to decide what parsers to run
        let ctx = self.ctx.data.borrow();
        if let Some(front) = ctx.args.all.get(ctx.args.cur) {
            let name = if let Some(long) = front.strip_prefix("--") {
                Name::Long(long)
            } else if let Some(short) = front.strip_prefix("-") {
                Name::Short(short.chars().next().unwrap())
            } else {
                todo!("nothing matches, time to complain");
            };

            match self.named.get(&name).copied() {
                Some(c) => ids.push(c),
                None => todo!(
                    "unknown name - complain {:?} / {:?} / {:?}",
                    name,
                    front,
                    self.named
                ),
            }
        } else {
            println!("nothing to parse, time to terminate things");
        }
    }

    fn run_parser<P, T>(mut self, parser: &'a P) -> Result<T, Error>
    where
        P: Parser<T>,
        T: std::fmt::Debug + 'static,
    {
        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead.
        let mut handle = pin!(self
            .ctx
            .spawn(Parent::new(Id(0), 0, ParentKind::Root), parser));
        let root_waker = self.ctx.waker_for(Id(0));

        // poll root handle once so whatever needs to be
        // register - gets a chance to do so then
        // set it aside until all child tasks are satisfied
        let mut root_cx = Context::from_waker(&root_waker);
        if let Poll::Ready(r) = handle.as_mut().poll(&mut root_cx) {
            todo!("make sure there's no unconsumed data");
            return r;
        }

        // get shared data out of the context for easier use
        let spawn = self.ctx.spawn.clone();
        let pending = self.ctx.pending.clone();

        let mut ids = Vec::new();
        let mut par = Vec::new();
        loop {
            // first we wake spawn all the pending tasks and poll them to
            // make sure things propagate. this might take several loops
            loop {
                let mut to_spawn = spawn.lock().expect("poison");
                for (id, task, waker) in to_spawn.spawn.drain(..) {
                    self.tasks.insert(id, (task, waker));
                }

                for (names, id) in to_spawn.name.drain(..) {
                    for name in names.iter() {
                        self.named.insert(*name, id);
                    }
                }

                drop(to_spawn);

                let mut to_wake = pending.lock().expect("poison");
                if to_wake.is_empty() {
                    println!("nothing to wake?");
                    break;
                }
                ids.extend(to_wake.drain(..));
                drop(to_wake);
                println!("to wake: {ids:?}");

                for id in ids.drain(..) {
                    if let Some((task, waker)) = self.tasks.get_mut(&id) {
                        let mut cx = Context::from_waker(waker);
                        if task.act.as_mut().poll(&mut cx).is_ready() {
                            println!("task {id:?} is done, removing from poll");
                            self.tasks.remove(&id);
                        }
                    }
                }
            }

            self.tasks_to_run(&mut ids);
            if ids.is_empty() {
                if self.named.is_empty() {
                    println!("we are done, let's finish !, {:?}", self.named);
                    break;
                } else {
                    ids.extend(self.named.values());
                    self.named.clear();
                }
            }

            // actual feed consumption happens here
            let mut max_consumed = 0;
            for id in ids.drain(..) {
                if let Some((task, waker)) = self.tasks.get_mut(&id) {
                    let before = self.ctx.data.borrow().args.cur;
                    let mut cx = Context::from_waker(waker);
                    if task.act.as_mut().poll(&mut cx).is_ready() {
                        println!("task {id:?} is done from parse");
                        self.tasks.remove(&id);
                    }
                    let after = self.ctx.data.borrow().args.cur;
                    self.ctx.data.borrow_mut().args.cur = before;
                    let consumed = after - before;
                    max_consumed = consumed.max(max_consumed);
                    par.push((consumed, id));
                }
                par.retain(|(len, _id)| *len == max_consumed);
            }

            // next task is to go over all the `par` results up to root, mark
            // all the alt branches that are still present in `par` and their
            // parents up to the top most alt branch as safe and
            // terminate all unmarked branches
            self.ctx.data.borrow_mut().args.cur = max_consumed;
        }
        match handle.as_mut().poll(&mut root_cx) {
            Poll::Ready(r) => r,
            Poll::Pending => todo!(),
        }
    }
}

#[derive(Clone)]
struct Named {
    name: Rc<[Name<'static>]>,
}

impl Parser<Name<'static>> for Named {
    fn run<'a>(&'a self, input: Ctx<'a>) -> BoxedFrag<'a, Name<'static>> {
        Box::pin(NamedFut {
            name: self.name.clone(),
            ctx: input.clone(),
            registered: false,
        })
    }
}

#[derive(Clone)]
struct Flag<T> {
    name: Named,
    present: T,
    absent: Option<T>,
}
impl<T> Parser<T> for Flag<T>
where
    T: std::fmt::Debug + Clone + 'static,
{
    fn run<'a>(&'a self, input: Ctx<'a>) -> BoxedFrag<'a, T> {
        Box::pin(async move {
            match self.name.run(input).await {
                Ok(_) => Ok(self.present.clone()),
                Err(Error::Missing) => match self.absent.as_ref().cloned() {
                    Some(v) => Ok(v),
                    None => Err(Error::Missing),
                },
                Err(e) => Err(e),
            }
        })
    }
}

#[test]
fn asdf() {
    let name = Named {
        name: Rc::from(vec![Name::Long("bob")].as_slice()),
    };
    let flag = Flag {
        name,
        present: true,
        absent: Some(false),
    };
    //    let r = parse_args(flag, &["--bob".into()]);
    let r = parse_args(flag, &[]);
    todo!("{:?}", r);
}

struct Alt<T: Clone + 'static> {
    items: Vec<Box<dyn Parser<T>>>,
}

impl<T> Parser<T> for Box<dyn Parser<T>>
where
    T: std::fmt::Debug + Clone + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T> {
        self.as_ref().run(ctx)
    }
}

impl<T> Parser<T> for Alt<T>
where
    T: Clone + std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T> {
        Box::pin(async move {
            let id = Id(0);
            for (ix, p) in self.items.iter().enumerate() {
                let field = ix as u32;
                ctx.spawn(
                    Parent {
                        id,
                        field,
                        kind: ParentKind::Sum,
                    },
                    p,
                );
            }

            for _ in self.items.iter() {
                let m_err = ChildErrors { id }.await;
            }
            // loop
            // subscribe for any events related to all the handles
            // trim handles that didn't advance enough
            // return first succesful result
            todo!()
        })
    }
}

// priority forest
// killing underperforming tasks
// conflicts

struct ChildErrors {
    id: Id,
}

impl Future for ChildErrors {
    type Output = Option<(/* field */ u32, Error)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
