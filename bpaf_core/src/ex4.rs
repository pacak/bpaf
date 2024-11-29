#![allow(unused_imports, dead_code, unused_variables)]
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
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
struct Parent {
    id: Id,
    field: u32,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Name<'a> {
    Short(char),
    Long(&'a str),
}

struct Task<'a> {
    parent: Parent,
    act: Pin<Box<dyn Future<Output = ()> + 'a>>,
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
struct Ctx<'a> {
    data: Rc<RefCell<RawCtx<'a>>>,
}

fn parse_args<T>(parser: impl Parser<T>, args: &[String]) -> Result<T, Error>
where
    T: 'static,
{
    let ctx = RefCell::new(RawCtx {
        next_id: 0,
        args: Args {
            all: Rc::from(args),
            cur: 0,
        },
        tasks: BTreeMap::new(),
        named: BTreeMap::new(),
        pending: Default::default(),
    });
    let ctx = Ctx { data: Rc::new(ctx) };

    let runner = Runner { ctx };
    runner.block_on(&parser)
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

impl<'a> Ctx<'a> {
    fn spawn<T: 'static, P>(&self, parent: Parent, parser: &'a P) -> JoinHandle<Result<T, Error>>
    where
        P: Parser<T>,
    {
        let ctx = self.clone();
        let (exit, join) = fork();
        let act = Box::pin(async move {
            println!("Waiting on spawned task");
            let r = parser.run(ctx).await;
            if let Ok(exit) = Rc::try_unwrap(exit) {
                exit.exit_task(r);
            }
        });

        self.start_task(Task { parent, act });
        println!("{:?},", self.data.borrow().tasks.keys());

        join
    }

    ///
    ///
    /// this needs name to know what to look for, waker since that's how futures work....
    /// Or do they...
    /// But I also need an ID so I can start placing items into priority forest
    fn named_wake(&self, id: Id, name: Rc<[Name<'static>]>, waker: Waker) {
        let mut ctx = self.data.borrow_mut();
        for name in name.iter() {
            ctx.named.insert(*name, waker.clone());
        }
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

    fn start_task(&self, act: Task<'a>) {
        let id = self.next_id();
        let waker = self.waker_for(id);
        self.data.borrow_mut().tasks.insert(id, (act, waker));
        self.data.borrow().pending.lock().expect("poison").push(id);
    }

    fn waker_for(&self, id: Id) -> Waker {
        Waker::from(Arc::new(WakeTask {
            id,
            pending: self.data.borrow().pending.clone(),
        }))
    }
}

struct RawCtx<'a> {
    next_id: u32,
    args: Args,
    tasks: BTreeMap<Id, (Task<'a>, Waker)>,
    named: BTreeMap<Name<'static>, Waker>,
    pending: Arc<Mutex<Vec<Id>>>,
}

type BoxedFrag<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;
trait Parser<T: 'static> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T>;
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
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, (RA, RB)> {
        Box::pin(async move {
            let id = ctx.next_id();
            let futa = ctx.spawn(Parent { id, field: 0 }, &self.0);
            let futb = ctx.spawn(Parent { id, field: 1 }, &self.1);
            Ok((futa.await?, futb.await?))
        })
    }
}

#[derive(Clone)]
struct Many<P>(P);
impl<T: 'static, P> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, Vec<T>> {
        let id = ctx.next_id();

        let mut res = Vec::new();
        let parent = Parent { id, field: 0 };
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
    T: Clone,
    R: 'static,
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
}

impl<'a> Runner<'a> {
    fn tasks_to_run(&self, ids: &mut Vec<Id>) {
        // // first we need to decide what parsers to run
        // let ctx = self.ctx.0.borrow();
        // if let Some(front) = ctx.args.all.get(ctx.args.cur) {
        //     let name = if let Some(long) = front.strip_prefix("--") {
        //         Name::Long(long)
        //     } else if let Some(short) = front.strip_prefix("-") {
        //         Name::Short(short.chars().next().unwrap())
        //     } else {
        //         todo!("nothing matches, time to complain");
        //     };
        //
        //     match self.named.get(&name) {
        //         Some(c) => ids.extend(c.iter()),
        //         None => todo!(
        //             "unknown name - complain {:?} / {:?} / {:?}",
        //             name,
        //             front,
        //             self.named
        //         ),
        //     }
        // } else {
        //     todo!("nothing to parse, time to terminate things");
        // }
    }

    fn block_on<P, T>(self, parser: &'a P) -> Result<T, Error>
    where
        P: Parser<T>,
        T: 'static,
    {
        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead.
        let root_id = self.ctx.peek_next_id();
        let parent = Parent {
            id: Id(0),
            field: 0,
        };
        let h = pin!(self.ctx.spawn(parent, parser));
        let root_waker = self.ctx.waker_for(root_id);

        // poll root handle once so whatever needs to be
        // register - gets a chance to do so then
        // set it aside until all child tasks are satisfied
        let mut root_cx = Context::from_waker(&root_waker);
        if let Poll::Ready(r) = h.poll(&mut root_cx) {
            todo!("make sure there's no unconsumed data");
            return r;
        }

        let mut ids = Vec::new();
        ids.extend(
            self.ctx
                .data
                .borrow()
                .pending
                .lock()
                .expect("poison")
                .drain(..),
        );

        for id in ids.drain(..) {
            if let Some((task, waker)) = self.ctx.data.borrow_mut().tasks.get_mut(&id) {
                let mut cx = Context::from_waker(waker);
                let r = task.act.as_mut().poll(&mut cx);
                todo!("{:?}", r);
            }
        }
        todo!("{:?}", self.ctx.data.borrow().named.keys());

        todo!();
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
impl<T: Clone + 'static> Parser<T> for Flag<T> {
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
    let r = parse_args(flag, &["--bob".into()]);
    todo!("{:?}", r);
}

struct Alt<T: Clone + 'static> {
    items: Vec<Box<dyn Parser<T>>>,
}

impl<T: Clone + 'static> Parser<T> for Box<dyn Parser<T>> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T> {
        self.as_ref().run(ctx)
    }
}

impl<T: Clone + 'static> Parser<T> for Alt<T> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> BoxedFrag<'a, T> {
        Box::pin(async move {
            let id = Id(0);
            for (ix, p) in self.items.iter().enumerate() {
                let field = ix as u32;
                ctx.spawn(Parent { id, field }, p);
            }
            // loop
            // subscribe for any events related to all the handles
            // trim handles that didn't advance enough
            // return first succesful result
            todo!()
        })
    }
}
