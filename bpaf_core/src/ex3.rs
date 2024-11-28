#![allow(unused_imports, dead_code)]
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    future::Future,
    marker::PhantomData,
    rc::Rc,
    sync::mpsc::{sync_channel, Receiver, Sender, SyncSender},
    task::Poll,
};

/// An atomic unit of argument consumption
///
/// Action tries to consume as many items from the front,
/// as it needs only succeeds when it can produce meaningful
/// results.
///
/// Usually it corresponds to a thing like a single named argument
/// or a single positional
///
struct Atom {
    parent: Parent,
    act: Box<AtomicAction>,
}
type AtomicAction = dyn FnMut(Args, bool) -> Option<(usize, Box<dyn FnOnce()>)>;

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

struct RawCtx {
    next_id: u32,
    args: Args,
    actions: BTreeMap<Id, Atom>,
    named: BTreeMap<Name<'static>, BTreeSet<Id>>,
    positional: BTreeSet<Id>,
}

#[derive(Clone)]
struct Args {
    all: Rc<[String]>,
    cur: usize,
}

fn make_commit<T: 'static>(val: T, chan: SyncSender<T>) -> Box<dyn FnOnce()> {
    Box::new(move || {
        let _ = chan.send(val);
    })
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

    fn run_all(&mut self) {}

    fn spawn<T: 'static>(
        &mut self,
        parent: Parent,
        parser: impl Parser<T> + 'static,
    ) -> Receiver<Result<T, Error>> {
        let (sender, receiver) = sync_channel(1);

        let id = Id(self.next_id);
        self.next_id += 1;
        let act = Box::new(move |mut input: Args, force| {
            let before = input.cur;
            let t = parser.run(&mut input);
            let after = input.cur;
            if force || !matches!(t, Err(Error::Missing)) {
                Some((after - before, make_commit(t, sender.clone())))
            } else {
                None
            }
        });
        self.actions.insert(id, Atom { parent, act });
        receiver
    }
}

enum Error {
    Missing,
    Invalid,
}
trait Parser<T> {
    fn run(&self, input: &mut Args) -> Result<T, Error>;
}

struct Pair<A, B>(A, B);
impl<A, B, RA, RB> Parser<(RA, RB)> for Pair<A, B>
where
    A: Parser<RA>,
    B: Parser<RB>,
{
    fn run(&self, input: &mut Args) -> Result<(RA, RB), Error> {
        // spawn A
        // spawn B,
        // yield - yield means I need a future
        // block on reading from A
        // block on reading from B
        todo!()
    }
}

fn parse<T: 'static>(input: Vec<String>, parser: impl Parser<T> + 'static) -> Result<T, Error> {
    let mut ctx = RawCtx::new(input);

    let parent = Parent {
        id: Id(0),
        field: 0,
    };
    let res = ctx.spawn(parent, parser);
    ctx.run_all();
    res.recv().unwrap()
}
