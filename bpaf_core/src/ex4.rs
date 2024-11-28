#![allow(unused_imports, dead_code)]
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
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

struct Task {
    parent: Parent,
    act: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Clone)]
struct Args {
    all: Rc<[String]>,
    cur: usize,
}

impl Args {
    fn take_name(&mut self, names: &[Name<'static>]) -> Option<Name<'static>> {
        None
    }
}

#[derive(Clone)]
struct X(Rc<RefCell<ChildCtx>>);

impl X {
    async fn spawn<T: 'static>(
        &self,
        parent: Parent,
        parser: impl Parser<T> + 'static,
    ) -> Result<T, Error> {
        let (a, b) = sync_channel(1);
        let ictx = self.clone();
        let act = Box::pin(async move {
            let r = parser.run(ictx.clone()).await;
            ictx.try_finalize().await;
            a.send(r).unwrap();
        });

        let task = Task { parent, act };
        let ctx = (self.0).borrow_mut();
        ctx.spawner.send(task).unwrap();
        // block here until send completes
        b.recv().unwrap()
    }
}

impl X {
    fn try_finalize(&self) -> impl Future<Output = ()> {
        let ctx = self.clone().0;
        std::future::poll_fn(move |_| {
            if ctx.borrow_mut().commit {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

struct ChildCtx {
    args: Args,
    cur: usize,
    commit: bool,
    spawner: std::sync::mpsc::Sender<Task>,
}

type Ctx = Rc<RefCell<RawCtx>>;
struct RawCtx {
    next_id: u32,
    args: Args,
    actions: BTreeMap<Id, Task>,
    named: BTreeMap<Name<'static>, BTreeSet<Id>>,
    positional: BTreeSet<Id>,
}

impl RawCtx {
    fn new(args: Vec<String>) -> Self {
        let args = Args {
            all: Rc::from(args),
            cur: 0,
        };
        Self {
            next_id: 0,
            args,
            actions: Default::default(),
            named: Default::default(),
            positional: Default::default(),
        }
    }

    async fn spawn<T: 'static>(
        &mut self,
        parent: Parent,
        parser: impl Parser<T> + 'static,
    ) -> Result<T, Error> {
        todo!()
    }
}

trait Parser<T: 'static>: Clone + 'static {
    fn run(&self, input: X) -> impl Future<Output = Result<T, Error>>;
}

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
    async fn run(&self, input: X) -> Result<(RA, RB), Error> {
        let id = Id(0);
        let futa = input.spawn(Parent { id, field: 0 }, self.0.clone());
        let futb = input.spawn(Parent { id, field: 1 }, self.1.clone());
        Ok((futa.await?, futb.await?))
    }
}

#[derive(Clone)]
struct Many<P>(P);
impl<T: 'static, P> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    async fn run(&self, input: X) -> Result<Vec<T>, Error> {
        let id = Id(0);
        let mut res = Vec::new();
        let parent = Parent { id, field: 0 };
        loop {
            match input.spawn(parent, self.0.clone()).await {
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
    async fn run(&self, input: X) -> Result<R, Error> {
        Ok((self.1)(self.0.run(input.clone()).await?))
    }
}

struct Named {
    name: Rc<[Name<'static>]>,
    args: Rc<RefCell<Args>>,
    spawn: Option<std::sync::mpsc::Sender<(Rc<[Name<'static>]>, Waker)>>,
}

impl Future for Named {
    type Output = Option<Name<'static>>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(snd) = self.spawn.take() {
            snd.send((self.name.clone(), cx.waker().clone())).unwrap();
            return Poll::Pending;
        }
        Poll::Ready(self.args.borrow_mut().take_name(&self.name))
    }
}

struct WakeTask {
    id: Id,
    queue: std::sync::mpsc::Sender<Id>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        self.queue.send(self.id).unwrap();
    }
}

struct Runner {
    queue: Receiver<Id>,
    ctx: Ctx,
    tasks: BTreeMap<Id, (Task, Waker)>,
    named: BTreeMap<Name<'static>, BTreeSet<Id>>,
}

impl RawCtx {
    fn front_name(&self) -> Name<'static> {
        todo!();
    }
    fn stash(&mut self) {}
}

impl Runner {
    fn run(&mut self) {
        let front = self.ctx.borrow().front_name();
        match self.named.get(&front) {
            Some(xs) => {
                if xs.len() == 1 {
                    let id = xs.first().unwrap();
                    let (task, waker) = self.tasks.get_mut(id).unwrap();

                    let mut cx = Context::from_waker(waker);
                    task.act.as_mut().poll(&mut cx);
                } else {
                    self.ctx.borrow_mut().stash();
                    for candidate in xs.iter() {}
                }
            }
            None => todo!(),
        }
    }
}
