#![allow(unused_variables)]
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    marker::PhantomData,
    rc::Rc,
};

#[derive(Debug, Copy, Clone)]
enum Error {
    NothingFound,
    Missing,
}

enum Pong<T> {
    Done(Result<T, Error>),
    NotReady,
}

#[derive(Debug, Copy, Clone)]
struct Id {
    start: u32,
    end: u32,
}

type Ctx = Rc<RefCell<RawCtx>>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum NamedTy {
    Arg,
    Flag,
}
#[derive(Default)]
struct RawCtx {
    /// Current position
    ///
    /// starts with 1 and advances forward 0 corresponds to application name
    pos: usize,

    /// all the items to parse, including 0 as an application name
    args: Rc<[String]>,

    /// parsers that accept anything, run sequentially
    any: BTreeMap<u32, Box<dyn Fn(ValueRef) -> Box<dyn Any>>>,

    /// Named parsers
    named: BTreeMap<Name<'static>, BTreeMap<u32, NamedTy>>,

    positional: BTreeSet<u32>,

    /// registration for soft "no parse" events - optional flags,
    no_parse: BTreeSet<u32>,

    after_ddash: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Name<'a> {
    Short(char),
    Long(&'a str),
}

#[derive(Debug, Clone)]
pub enum ValueRef {
    Str(Rc<str>),
    OsStr(Rc<OsString>),
}

fn run<T>(parser: impl Parser<T>, args: Vec<String>) -> Result<T, Error> {
    let args = <Rc<[String]>>::from(args);
    let ctx = RawCtx::new(args.clone());
    let mut events = Vec::<u32>::new();
    let mut handler = {
        let mut id = 0;
        parser.start(ctx.clone(), &mut id)
    };

    loop {
        let ctx = (*ctx).borrow();
        events.clear();

        if let Some(val) = args.get(ctx.pos) {
            match split_param(val) {
                Arg::Named { name, val } => match ctx.named.get(&name) {
                    Some(ids) => events.extend(ids.iter().filter_map(|(id, ty)| match &val {
                        // we got an attached value, emit an event only for arg ty
                        Some(val) => (*ty == NamedTy::Arg).then(|| *id),

                        None => Some(*id),
                    })),
                    None => todo!("unknown name {name:?}"),
                },
                Arg::ShortSet { names } => {
                    todo!("split things, emit a bunch of JSF events for {names:?}")
                }
                Arg::Positional { value } => events.extend(ctx.positional.iter().copied()),
            }
            drop(ctx);
            if handler.handle(events.as_slice()) {
                break;
            }
        } else {
            todo!("emit EndOfInput, terminate");
        }
    }
    handler.unwrap()
}

// fn run<T>(parser: impl Parser<T>, args: Vec<String>) -> Result<T, Error> {
//     let args = <Rc<[String]>>::from(args);
//     let mut ctx = RawCtx::new(args.clone());
//     let mut b_ids = Vec::new();
//     let mut b_str = Vec::new();
//     let mut handler = parser.spawn(ctx.clone());
//     // I'll need to copy things out of Ctx, it is an Rc so it can't be a reference
//     // the idea is to have things borrowed
//     loop {
//         let mut events = Vec::new();
//         {
//             (*ctx).borrow().next_events(&args);
//         }
//         for event in &events {
//             match event {
//                 Event::JoinedShortFlags { values, id } => todo!(),
//                 Event::Flag { name, id } => todo!(),
//                 Event::Arg { name, value, id } => todo!(),
//                 Event::Positional {
//                     after_ddash,
//                     value,
//                     id,
//                 } => todo!(),
//                 Event::NoParse { id } => todo!(),
//             }
//         }
//     }
//
//     todo!();
// }

enum Arg<'a> {
    Named {
        name: Name<'a>,
        val: Option<ValueRef>,
    },
    ShortSet {
        names: Vec<char>,
    },
    Positional {
        value: ValueRef,
    },
}

fn split_param(s: &str) -> Arg {
    if let Some(long_name) = s.strip_prefix("--") {
        match long_name.split_once('=') {
            Some((name, arg)) => Arg::Named {
                name: Name::Long(name),
                val: Some(ValueRef::Str(arg.into())),
            },
            None => Arg::Named {
                name: Name::Long(long_name),
                val: None,
            },
        }
    } else if let Some(short_name) = s.strip_prefix("-") {
        match short_name.split_once('=') {
            Some((name, arg)) => {
                let name = name.chars().next().unwrap(); // TODO
                Arg::Named {
                    name: Name::Short(name),
                    val: Some(ValueRef::Str(arg.into())),
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
        Arg::Positional {
            value: ValueRef::Str(s.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    /// A group of short flags joined together: `-abc`
    JoinedShortFlags {
        /// Current event id - parser will be executed
        /// multiple times, once for each id - to deal with
        /// repeated flags and potential conflicts, so if -a and -b
        /// are mutually exclusive, parser for `-a` makes Alt finished
        /// and subsequent event for `b` gets converted into an error
        id: u32,
    },
    /// A short or long name flag, with no attached value: `-a` or `--foo`
    Flag {
        id: u32,
    },
    /// A short or long name argument with value attached: `-ofile` or `--foo=bar`
    /// but not `--foo bar` as two separate items
    Arg {
        value: ValueRef,
        id: u32,
    },
    /// positional item
    Positional {
        after_ddash: bool,
        value: ValueRef,
        id: u32,
    },
    /// No existing parsers could handle it
    /// this event is sent to flush all the parsers with fallback and to finish all the "many"
    /// parser variants
    NoParse {
        id: u32,
    },

    // Sent at the end of the input to everybody not done yet
    // the difference with noparse:
    // - EOI means there's no more input, we must produce the result or error out
    // - NP for things like `many` or `optional` means produce `empty` and potentially unblock
    //   other parsers
    EndOfInput {
        id: u32,
    },
}

impl RawCtx {
    fn new(args: Rc<[String]>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            pos: 0,
            args,
            named: BTreeMap::new(),
            no_parse: BTreeSet::new(),
            any: BTreeMap::new(),
            positional: BTreeSet::new(),
            after_ddash: false,
        }))
    }

    /// Add a listener for those names
    fn register_names(&mut self, names: &[Name<'static>], ty: NamedTy, id: u32) {
        for name in names {
            self.named.entry(*name).or_default().insert(id, ty);
        }
    }

    fn register_noparse(&mut self, id: u32) {
        self.no_parse.insert(id);
        //
    }

    fn unregister_names(&mut self, names: &[Name<'static>], id: u32) {
        use std::collections::btree_map::Entry;
        for name in names {
            if let Entry::Occupied(mut names) = self.named.entry(*name) {
                names.get_mut().remove(&id);
                if names.get().is_empty() {
                    names.remove();
                }
            }
        }
        //
    }

    fn remove_noparse(&mut self, id: u32) {
        self.no_parse.remove(&id);
    }

    // fn emit_no_parse(&self) -> Vec<Event<'static>> {
    //     self.no_parse.iter().map(|id| Event::NoParse { id: () }
    // }
}

enum X<T> {
    B(Box<dyn FnMut(Event) -> (usize, Pong<T>)>),
    R(Result<T, Error>),
    Done,
}
impl<T> X<T> {
    fn ping(&mut self, event: Event) -> usize {
        if let X::B(b) = self {
            let (cnt, r) = b(event);
            if let Pong::Done(d) = r {
                *self = X::R(d);
            }
            cnt
        } else {
            0
        }
    }
}

// fn pair() {
//     let ctx: Rc<RefCell<RawCtx>> = Default::default();
//     let a = Flag::add('a', 0, ctx.clone());
//     let b = Flag::add('b', 1, ctx.clone());
//
//     let mut ba = X::B(Box::new(a));
//     let mut bb = X::B(Box::new(b));
//     let mk = |a: Result<(), Error>, b: Result<(), Error>| Ok((a?, b?));
//     let mut clo = move |e: Event| {
//         ba.ping(e);
//         bb.ping(e);
//
//         match (ba, bb) {
//             (X::R(r1), X::R(r2)) => Pong::Done(mk(r1, r2)),
//             _ => Pong::NotReady,
//         }
//     };
// }

/// We have futures at home
///
/// The main difference is that I want to be able to run all the await
/// points in parallel so runner passes events to
///
/// Result is stored in Spark::Ready rather than part of the handle
/// since I want to be able to keep sending events to products of sparks
/// until all of them are ready then to use the results to produce the final product
enum Spark<'a, T> {
    Pending {
        range: (u32, u32),
        disconnect: Option<Disconnect<'a, T>>,
        handler: Box<dyn FnMut(&[u32]) -> Option<Result<(usize, T), Error>> + 'a>,
    },
    Ready(Result<(usize, T), Error>),
    Done,
}

struct Disconnect<'a, T> {
    id: u32,
    ctx: Ctx,
    parser: &'a dyn Parser<T>,
}
impl<T> Drop for Disconnect<'_, T> {
    fn drop(&mut self) {
        self.parser.stop(self.ctx.clone(), self.id);
    }
}

impl<T> Spark<'_, T> {
    fn handle(&mut self, events: &[u32]) -> bool {
        let Spark::Pending { range, handler, .. } = self else {
            panic!("sparks should not be called once they are ready");
        };
        let from = events.partition_point(|id| *id >= range.0);
        let to = events.partition_point(|id| *id < range.1);
        println!("got events {events:?}, my range is {range:?}, pp-ed to {from}..{to}");
        let events = &events[from..to];
        if let Some(result) = handler(events) {
            *self = Spark::Ready(result);
            true
        } else {
            false
        }
    }
    fn unwrap(self) -> Result<T, Error> {
        match self {
            Spark::Ready(r) => r.map(|r| r.1),
            Spark::Pending { .. } => panic!("spark is not ready"),
            Spark::Done => panic!("spark is done and the result is out"),
        }
    }
}

trait Parser<T> {
    fn start<'a>(&'a self, ctx: Ctx, id: &mut u32) -> Spark<'a, T>;
    fn stop(&self, ctx: Ctx, id: u32);
    //    fn spawn(&self, ctx: Ctx) -> Box<dyn FnMut(&Event) -> Pong<T>>;
}

// (1)
struct Named {
    names: Vec<Name<'static>>,
}

fn for_me(xs: &[u32], me: u32) -> bool {
    todo!()
}

fn narrow(xs: &[u32], from: u32, to: u32) -> Option<&[u32]> {
    todo!()
}

struct Many<P, T> {
    inner: P,
    t: PhantomData<T>,
}
impl<P, T, C> Parser<C> for Many<P, T>
where
    P: Parser<T>,
    C: FromIterator<T>,
{
    fn start<'a>(&'a self, ctx: Ctx, id: &mut u32) -> Spark<'a, C> {
        let from = *id;
        *id += 1;
        let inner = self.inner.start(ctx, id);
        let to = *id;
        todo!()
    }

    fn stop(&self, ctx: Ctx, id: u32) {
        todo!()
    }
}
impl Parser<()> for Named {
    fn start<'a>(&'a self, ctx: Ctx, id: &mut u32) -> Spark<'a, ()> {
        let range = (*id, *id);
        (*ctx)
            .borrow_mut()
            .register_names(&self.names, NamedTy::Flag, *id);
        let disconnect = Some(Disconnect {
            id: *id,
            ctx: ctx.clone(),
            parser: self,
        });
        *id += 1;
        Spark::Pending {
            range,
            disconnect,
            handler: Box::new(move |_| {
                let ctx = (*ctx).borrow_mut();
                print!(
                    "need to check if name {:?} is present, but let's assume it does",
                    self.names
                );

                Some(Ok((1, ())))
            }),
        }
    }

    fn stop(&self, ctx: Ctx, id: u32) {
        (*ctx).borrow_mut().unregister_names(&self.names, id);
    }
}

// (3)
// 1. user makes something impl Parser<T> - structure is fixed, but it knows nothing about ids
//    or context
//    This part is user facing and exists to give access to bits that can be shuffled randomly.
//    the same file() parser can be used in multiple places
//
//
// 2. Parser::<T>::make prepares something that can be executed - should know about ids
//    Once we have a parser we want to be able to run it multiple times maintainging the
//    ids, so we want the ability to spawn it without losing access to IDs and cancel them
//
//
// 3. Something from the previous step gets executed, keeps it state and custom logic!
//    produces a result and gets retired, needs to implement drop to
//
//    events are routed to (3) by (2) and (3) should receive exactly one event at a time
#[test]

fn demo() {
    let p = Named {
        names: vec![Name::Short('a'), Name::Short('b'), Name::Long("abra")],
    };

    let x = run(p, vec!["-a".to_owned()]);
    todo!("{x:?}");
}
