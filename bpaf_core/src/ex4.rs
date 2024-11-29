#![allow(unused_imports, dead_code, unused_variables)]
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::{Rc, Weak},
    sync::mpsc::{sync_channel, Receiver, Sender, SyncSender},
    task::{Context, Poll, Wake, Waker},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Id(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Parent {
    id: Id,
    field: u32,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Name<'a> {
    Short(char),
    Long(&'a str),
}

// what assigns ids?
// spawn

struct Task {
    parent: Parent,
    act: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Debug, Clone)]
struct Args {
    all: Rc<[String]>,
    cur: usize,
}

impl Args {
    fn take_name(&mut self, names: &[Name<'static>]) -> Option<Name<'static>> {
        todo!()
    }
}

#[derive(Clone)]
struct ParserCtx(Rc<RefCell<RawParserCtx>>);

fn run<T: 'static>(parser: impl Parser<T>, args: &[String]) -> Result<T, Error> {
    let args = Args {
        all: Rc::from(args),
        cur: 0,
    };
    let (spawner, queue) = std::sync::mpsc::channel();
    let pctx = RawParserCtx {
        next_id: 0,
        args,
        spawner,
    };
    let ctx = ParserCtx(Rc::new(RefCell::new(pctx)));
    let runner = Runner {
        queue,
        ctx: ctx.clone(),
        tasks: BTreeMap::new(),
        named: BTreeMap::new(),
    };
    let handle = ctx.spawn(
        Parent {
            id: Id(0),
            field: 0,
        },
        parser.clone(),
    );
    runner.block_on(handle)
}

fn fork<T>() -> (Rc<ExitHandle<T>>, JoinHandle<T>) {
    let exit = ExitHandle {
        waker: Cell::new(None),
        result: Rc::new(Cell::new(None)),
    };
    let exit = Rc::new(exit);
    let join = JoinHandle {
        task: Rc::downgrade(&exit),
        result: Default::default(),
    };
    (exit, join)
}

struct ExitHandle<T> {
    waker: Cell<Option<Waker>>,
    result: Rc<Cell<Option<T>>>,
}

struct JoinHandle<T> {
    task: Weak<ExitHandle<T>>,
    result: Rc<Cell<Option<T>>>,
}
impl<T> ExitHandle<T> {
    fn exit_task(self, result: T) {
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}
impl<ReturnType> Future for JoinHandle<ReturnType> {
    type Output = ReturnType;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.task.upgrade() {
            Some(task) => {
                task.waker.set(Some(cx.waker().clone()));
                Poll::Pending
            }
            None => Poll::Ready(self.result.take().expect("Task exit sets result")),
        }
    }
}

impl ParserCtx {
    fn spawn<T: 'static>(
        &self,
        parent: Parent,
        parser: impl Parser<T> + 'static,
    ) -> JoinHandle<Result<T, Error>> {
        let ictx = self.clone();
        let (exit, join) = fork();
        let act = Box::pin(async move {
            let r = parser.run(&ictx).await;
            if let Ok(exit) = Rc::try_unwrap(exit) {
                exit.exit_task(r);
            }
        });

        (self.0)
            .borrow_mut()
            .spawner
            .send(Command::Spawn(Task { parent, act }))
            .unwrap();

        join
    }

    ///
    ///
    /// this needs name to know what to look for, waker since that's how futures work....
    /// Or do they...
    /// But I also need an ID so I can start placing items into priority forest
    fn named_wake(&self, id: Id, name: Rc<[Name<'static>]>, waker: Waker) {
        let ctx = self.0.borrow_mut();
        ctx.spawner
            .send(Command::NamedWake(id, name, waker))
            .unwrap();
    }

    fn take_name(&self, name: &[Name<'static>]) -> Option<Name<'static>> {
        todo!()
    }

    fn next_id(&self) -> Id {
        let mut ctx = self.0.borrow_mut();
        let id = ctx.next_id;
        ctx.next_id += 1;
        Id(id)
    }
}
unsafe impl Sync for Command {}
unsafe impl Send for Command {}
enum Command {
    Spawn(Task),
    NamedWake(Id, Rc<[Name<'static>]>, Waker),
    Wake(Id),
}

struct RawParserCtx {
    next_id: u32,
    args: Args,
    spawner: Sender<Command>,
}

// type Ctx = Rc<RefCell<RawCtx>>;
// struct RawCtx {
//     args: Args,
//     actions: BTreeMap<Id, Task>,
//     named: BTreeMap<Name<'static>, BTreeSet<Id>>,
//     positional: BTreeSet<Id>,
// }
//
// impl RawCtx {
//     fn new(args: Vec<String>) -> Self {
//         let args = Args {
//             all: Rc::from(args),
//             cur: 0,
//         };
//         Self {
//             args,
//             actions: Default::default(),
//             named: Default::default(),
//             positional: Default::default(),
//         }
//     }
//
//     async fn spawn<T: 'static>(
//         &mut self,
//         parent: Parent,
//         parser: impl Parser<T> + 'static,
//     ) -> Result<T, Error> {
//         todo!()
//     }
// }

trait Parser<T: 'static>: Clone + 'static {
    fn run(&self, ctx: &ParserCtx) -> impl Future<Output = Result<T, Error>>;
}

#[derive(Debug, Copy, Clone)]
enum Error {
    Missing,
    Invalid,
}

#[derive(Clone)]
struct Pair<A, B>(A, B);
impl<A, B, RA: 'static, RB: 'static> Parser<(RA, RB)> for Pair<A, B>
where
    A: Parser<RA>,
    B: Parser<RB>,
{
    async fn run(&self, ctx: &ParserCtx) -> Result<(RA, RB), Error> {
        let id = ctx.next_id();
        let futa = ctx.spawn(Parent { id, field: 0 }, self.0.clone());
        let futb = ctx.spawn(Parent { id, field: 1 }, self.1.clone());
        Ok((futa.await?, futb.await?))
    }
}

#[derive(Clone)]
struct Many<P>(P);
impl<T: 'static, P> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    async fn run(&self, ctx: &ParserCtx) -> Result<Vec<T>, Error> {
        let id = ctx.next_id();
        let mut res = Vec::new();
        let parent = Parent { id, field: 0 };
        loop {
            match ctx.spawn(parent, self.0.clone()).await {
                Ok(t) => res.push(t),
                Err(Error::Missing) => return Ok(res),
                Err(e) => return Err(e),
            }
        }
    }
}

#[derive(Clone)]
struct Map<P, F, T>(P, F, PhantomData<T>);
impl<T: 'static, R, F, P> Parser<R> for Map<P, F, T>
where
    P: Parser<T>,
    T: Clone,
    R: 'static,
    F: Fn(T) -> R + 'static + Clone,
{
    async fn run(&self, ctx: &ParserCtx) -> Result<R, Error> {
        Ok((self.1)(self.0.run(ctx).await?))
    }
}

struct NamedFut {
    name: Rc<[Name<'static>]>,
    ctx: ParserCtx,
    registered: bool,
}

impl Future for NamedFut {
    type Output = Result<Name<'static>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx
                .named_wake(Id(0), self.name.clone(), cx.waker().clone());
            Poll::Pending
        } else if let Some(name) = self.ctx.take_name(&self.name) {
            Poll::Ready(Ok(name))
        } else {
            Poll::Ready(Err(Error::Missing))
        }
    }
}

struct DummyWaker;

impl Wake for DummyWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

struct Waketask1 {
    id: Id,
    ctx: ParserCtx,
}
struct WakeTask {
    id: Id,
    queue: Sender<Command>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        self.queue.send(Command::Wake(self.id)).unwrap();
    }
}

struct Runner {
    queue: Receiver<Command>,
    ctx: ParserCtx,
    tasks: BTreeMap<Id, (Task, Waker)>,
    named: BTreeMap<Name<'static>, BTreeSet<Id>>,
}

impl Runner {
    fn tasks_to_run(&self, ids: &mut Vec<Id>) {
        // first we need to decide what parsers to run
        let ctx = self.ctx.0.borrow();
        if let Some(front) = ctx.args.all.get(ctx.args.cur) {
            let name = if let Some(long) = front.strip_prefix("--") {
                Name::Long(long)
            } else if let Some(short) = front.strip_prefix("-") {
                Name::Short(short.chars().next().unwrap())
            } else {
                todo!("nothing matches, time to complain");
            };

            match self.named.get(&name) {
                Some(c) => ids.extend(c.iter()),
                None => todo!(
                    "unknown name - complain {:?} / {:?} / {:?}",
                    name,
                    front,
                    self.named
                ),
            }
        } else {
            todo!("nothing to parse, time to terminate things");
        }
    }

    fn block_on<T>(mut self, fut: JoinHandle<T>) -> T {
        let mut ids = Vec::new();

        let root_waker: Waker = Waker::from(std::sync::Arc::new(WakeTask {
            id: Id(999),
            queue: self.ctx.0.borrow().spawner.clone(),
        }));
        let mut cx = Context::from_waker(&root_waker);
        let mut fut = std::pin::pin!(fut);
        if let Poll::Ready(r) = fut.as_mut().poll(&mut cx) {
            return r;
        }

        loop {
            while let Ok(x) = self.queue.try_recv() {
                match x {
                    Command::Spawn(t) => todo!(),
                    Command::NamedWake(id, x, y) => todo!(),
                    Command::Wake(_) => todo!(),
                }
            }

            self.tasks_to_run(&mut ids);

            for id in &ids {
                if *id == Id(0) {
                    let mut cx = Context::from_waker(&root_waker);
                    if let Poll::Ready(r) = fut.as_mut().poll(&mut cx) {
                        return r;
                    }
                    continue;
                }
                let (task, waker) = self.tasks.get_mut(id).unwrap();
                let mut cx = Context::from_waker(waker);
                let r = task.act.as_mut().poll(&mut cx);
                if r.is_ready() {
                    todo!();
                }
            }
        }
    }
}

#[derive(Clone)]
struct Named {
    name: Rc<[Name<'static>]>,
}

impl Parser<Name<'static>> for Named {
    fn run(&self, input: &ParserCtx) -> impl Future<Output = Result<Name<'static>, Error>> {
        NamedFut {
            name: self.name.clone(),
            ctx: input.clone(),
            registered: false,
        }
    }
}

#[derive(Clone)]
struct Flag<T> {
    name: Named,
    present: T,
    absent: Option<T>,
}
impl<T: Clone + 'static> Parser<T> for Flag<T> {
    async fn run(&self, input: &ParserCtx) -> Result<T, Error> {
        match self.name.run(input).await {
            Ok(_) => Ok(self.present.clone()),
            Err(Error::Missing) => match self.absent.as_ref().cloned() {
                Some(v) => Ok(v),
                None => Err(Error::Missing),
            },
            Err(e) => Err(e),
        }
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
    let r = run(flag, &["--bob".into()]);
    todo!("{:?}", r);
}
