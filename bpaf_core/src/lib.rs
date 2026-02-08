#![warn(clippy::unused_async)]
mod adapters;
mod anything;
mod arg;
mod args;
mod complete;
mod console_writer;
mod consumers;
mod core_consumers;
mod ctx;
mod error;
mod help_cmd;
mod info;
mod macros;
mod miniansi;
mod os_str;
mod pecking;
mod repeat;
mod traits;
mod utils;
mod visitors;

pub mod api {
    //! # Tutorial for User facing API
    //!
    //! <div class="warning">
    //!
    //! Everything under this module is available through functions exported from the
    //! crate's top level, [`construct!`](crate::construct) macro, methods on
    //! the structures or via [`Parser`](crate::Parser) trait. Main purpose
    //! is to make names visible and to contain some tutorial style documentation.
    //!
    //! You can't interact with types listed here directly other than using them to name
    //! intermediate values, but even then using `impl Parser<T>` usually gives cleaner API
    //!
    //! </div>

    pub mod composite {

        //! TODO - blurb on products and sums
        //! - [`Named::nest`]
        //! - [`Literal::nest`]

        use crate::{
            Ctx, Kind, Op, Parser, Scope,
            error::Error,
            traits::{RcParser, VisitGroup, Visitor},
        };

        #[cfg(doc)]
        use crate::consumers::{Keyword, Literal, Named};

        /// A categorical sum of two or more parsers
        ///
        /// This is a parser that is composed of two or more parsers. For `Sum<T>` to succeed
        /// at least one member must succeed. If there are several succeeding variants - one that
        /// consumes more input wins. If this is equal - one that goes earlier wins.
        ///
        /// You can create it with [`construct!`](crate::construct)
        ///
        /// TODO - a few dummy examples
        pub struct Sum<T> {
            pub items: Vec<RcParser<T>>,
        }

        impl<T: 'static> Sum<T> {
            pub fn or_else(&mut self, other: impl Parser<Output = T> + 'static) {
                self.items.push(other.into_rc());
            }
        }

        impl<T: 'static> Parser for Sum<T> {
            type Output = T;
            fn or_else(mut self, other: impl Parser<Output = Self::Output> + 'static) -> Self {
                self.items.push(other.into_rc());
                self
            }

            fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
                visitor.push_group(VisitGroup::Sum);
                for i in &self.items {
                    i.visit(visitor);
                }
                visitor.pop_group();
            }
            async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
                let id = ctx.current_task.borrow().id;
                let mut scopes = Vec::new();
                let handles = self
                    .items
                    .iter()
                    .map(|parser| {
                        let (h, scope) = ctx.scoped_spawn(parser, Kind::Sum);
                        scopes.push(scope);
                        h
                    })
                    .collect::<Vec<_>>();
                ctx.pending_ops.borrow_mut().push_back(Op::RegisterSum {
                    id,
                    scope: Scope {
                        start: id,
                        end: scopes.last().unwrap().end,
                    },
                });
                ctx.wait_for_children().await;
                ctx.trim_children(scopes).await;

                let mut acc = Error::Silent("Empty Alt?");

                let consumed = ctx.current_task.borrow().consumed > 0;

                // prefer final error if present or the earliest result
                let mut val = None;
                for h in handles {
                    match h.take() {
                        Err(err) => acc = acc + err,
                        Ok(v) if consumed => return Ok(v),
                        Ok(v) => val = val.or(Some(v)),
                    }
                }
                if matches!(&acc, Error::Final(_)) {
                    // this handles a scenario for a sum parser,
                    // where one branch is a command and the other branch succeeds
                    // without consumption. Whole parser succeeds, but we end up
                    // with unconsumed input and executor looks for suggestions and finds
                    // the command parser resulting in "no such parser `foo`, did you mean `foo`?"
                    Err(acc)
                } else {
                    val.ok_or(acc)
                }
            }
        }
    }

    /// fallback
    pub mod fallback {}

    /// repeat
    pub mod repeat {}

    pub mod functor {}

    pub mod ux {}

    pub mod primitives {
        #![allow(rustdoc::redundant_explicit_links)]
        #![allow(unused_imports)]
        use crate::*;
        use std::str::FromStr;
    }

    pub mod algebras {}
}

use crate::{
    arg::{Adjacency, Arg, lex_os_arg},
    args::Args,
    complete::CompleteReq,
    console_writer::Styled,
    info::{Custom, Extra, Info},
    utils::{Vec1, reuse_vec},
    visitors::{
        errors::{BetterName, IsAcceptedOnce, IsDDash, ValidCommand},
        help::render_help,
    },
};

#[doc(inline)]
pub use crate::{
    adapters::OptionParser,
    anything::{any, any_from_str},
    consumers::*,
    help_cmd::help_command,
    traits::{Parser, Visited},
};

use crate::{
    error::{Error, ParseFailure, Problem},
    traits::{RcParser, VisitGroup, Visitor, *},
};

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::{OsStr, OsString},
    fmt::Write,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::Poll,
};

/// A newtype wrapper for sending a value out of a finishing task
pub struct JoinHandle<T> {
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

impl<T> JoinHandle<T> {
    pub fn take(&self) -> Result<T, Error> {
        self.result
            .take()
            .unwrap_or(Err(Error::Silent("Empty JoinHandle?")))
    }
}

/// A newtype wrapper for receiving a value from a finishing task
pub(crate) struct ExitHandle<T> {
    result: Rc<Cell<Option<Result<T, Error>>>>,
}
impl<T> ExitHandle<T> {
    pub(crate) fn exit(&self, value: Result<T, Error>) {
        self.result.set(Some(value))
    }
}

#[doc(hidden)]
pub mod __private {
    pub use crate::api::composite::Sum;
    pub use crate::error::Error;
    pub use crate::traits::*;
    pub use crate::{Ctx, Kind};
    pub use ::std::compile_error;
}

impl std::ops::Add for Error {
    type Output = Error;

    fn add(self, rhs: Self) -> Self::Output {
        self.combine(rhs)
    }
}

#[derive(Debug, Copy, Clone)]
enum TChange {
    Add,
    Remove,
}

#[derive(Clone)]
enum TTarget {
    Arg(Name<'static>),
    Flag(Name<'static>),
    Pos,
    Check(Rc<dyn Fn(&OsStr) -> bool>),
    Literal(Lit<'static>),
}

impl std::fmt::Debug for TTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arg(arg0) => f.debug_tuple("Arg").field(arg0).finish(),
            Self::Flag(arg0) => f.debug_tuple("Flag").field(arg0).finish(),
            Self::Pos => write!(f, "Pos"),
            Self::Check(_) => f.debug_tuple("Any").field(&"...").finish(),
            Self::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
        }
    }
}

/// A parsing thread tracked by the executor
///
/// Doesn't produce a value directly, instead uses a pair of [`ExitHandle`]/[`JoinHandle`] to
/// pass the value to the parent thread.
///
/// Tasks form a tree structure
struct Task<'a> {
    act: Pin<Box<dyn Future<Output = ()> + 'a>>,
    info: TaskInfo,
}

impl std::fmt::Debug for Task<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Task { act: _, info } = self;
        f.debug_struct("Task").field("info", info).finish()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Sum,
    Prod,
}

/// Shared state and meta info about tasks
///
/// Used by the executor but also shared with the task itself as it runs in
/// [`Ctx::current_task`]
#[derive(Debug)]
struct TaskInfo {
    /// Current task Id
    id: Id,
    parent_kind: Kind,
    parent_id: Parent,
    consumed: u32,
    /// number of children this task is currently waiting for, we are going to wake them up only if
    // there's no pending children - if we can return immediately
    pending: u32,
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            id: Id(0),
            parent_kind: Kind::Sum,
            parent_id: Parent(0),
            consumed: 0,
            pending: 0,
        }
    }
}
#[derive(Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
/// Placeholder name used in help messages for [`Argument`] and [`Positional`]
pub struct Metavar(&'static str);
impl Metavar {
    #[inline(never)]
    pub fn width(&self) -> usize {
        let mut chars = 0;
        let mut angle = 0;
        for c in self.0.chars() {
            chars += 1;
            if !(c.is_ascii_digit() || c.is_ascii_uppercase() || c == '-' || c == '_') {
                angle = 2;
            }
        }
        chars + angle
    }
}

impl std::fmt::Display for Metavar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self
            .0
            .as_bytes()
            .iter()
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || *c == b'-' || *c == b'_')
        {
            write!(f, "{}", self.0)
        } else {
            write!(f, "<{}>", self.0)
        }
    }
}

/// A future that yields on the first poll and returns on the second one
///
/// Created with [`r#yield`], one of the key building blocks responsible
/// for cooperative multitasking
struct Yield(bool);

fn r#yield() -> Yield {
    Yield(false)
}

impl Future for Yield {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut val = self.as_mut();
        if val.0 {
            Poll::Ready(())
        } else {
            val.0 = true;
            Poll::Pending
        }
    }
}

/// A newtyped [`Id`] for parser tracking relationship
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Parent(u32);
impl Parent {
    fn as_id(&self) -> Id {
        Id(self.0)
    }

    fn is_root(&self) -> bool {
        *self == Parent(0)
    }
}

impl Id {
    #[cfg(test)]
    fn as_parent(&self) -> Parent {
        Parent(self.0)
    }
}

/// An `Id` of a parser instance
///
/// Used to track parsers in the Executor as well as relationship between parsers in the
/// [`PeckingOrder`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Id(u32);

pub use ctx::*;
#[derive(Debug)]
/// "we could have been consume `name`, but we consumed whatever was at `pos` instead"
enum Conflict {
    /// Named item - flag, argument
    Named { pos: u32, name: Name<'static> },
    /// Literal - command
    Lit { pos: u32, name: Lit<'static> },
    /// Positional item
    Pos { pos: u32 },
}

#[derive(Copy, Clone, Debug)]
enum KillReason {
    Conflict,
    NoMatchingInput,
    TooShort,
}

#[derive(Debug)]
enum Reason<'p> {
    /// Task is waken up so core consumer gets a chance to consume anything.
    /// - `Some` corresponds to a value requested by a trigger
    /// - `None` - leaf task is being told that there is no more matching arguments
    ///   left and it should produce what it can or fail.
    Arg(Arg<'p>),

    Kill(KillReason),

    Pass,

    /// Waking up trunk tasks when all the children are done
    Push,
    /// Notification from the executor to sums about children making progress
    ChildProgress(Vec1<Id>),
    Complete(CompleteReq),
}

fn make_chan<T: 'static>() -> (ExitHandle<T>, JoinHandle<T>) {
    let result = Rc::new(Cell::new(None));
    let exit = ExitHandle { result };
    let join = JoinHandle {
        result: exit.result.clone(),
    };
    (exit, join)
}

impl<'p> RawCtx<'p> {
    fn task_parent_and_id(&self) -> (Parent, Id) {
        let cur = self.current_task.borrow();
        (cur.parent_id, cur.id)
    }

    #[inline(never)]
    pub(crate) async fn is_outconsumed_leaf(&self, success: bool) -> bool {
        // The only possible scenario for a consuming leaf is getting an Arg
        if !matches!(&*self.wakeup_reason.borrow(), Reason::Arg(_)) {
            return false;
        }
        // if parser fails to produce a result due to validation or `FromStr`
        // we reset `consumed` back to zero so more conservative parallel branches
        // that did managed to produce a result can succeed
        if !success {
            self.current_task.borrow_mut().consumed = 0;
        }

        r#yield().await;
        // surviving gets Pass, out-consumed - NoPass, but we want to preserve the
        // error message so return true only when there's an OK result we don't want.
        matches!(
            &*self.wakeup_reason.borrow(),
            Reason::Kill(KillReason::TooShort)
        ) && success
    }

    /// After consumption when called from a leaf node returns what was consumed
    pub(crate) fn leaf_range(&self) -> Option<(u32, u32)> {
        if !matches!(&*self.wakeup_reason.borrow(), Reason::Arg(_)) {
            return None;
        }
        let start = self.cursor.get();
        let end = start + self.current_task.borrow().consumed;
        (end > start).then_some((start, end))
    }

    pub(crate) fn leaf_cursor(&self) -> u32 {
        if matches!(&*self.wakeup_reason.borrow(), Reason::Arg(_)) {
            self.cursor.get()
        } else {
            u32::MAX
        }
    }

    /// After consumption, when called from a leaf node returns what was consumed
    /// as a string
    pub(crate) fn leaf_consumed(&self) -> Option<OsString> {
        let (start, end) = self.leaf_range()?;
        let mut out = OsString::new();
        for (ix, i) in (start..end).enumerate() {
            if ix > 0 {
                out.push(" ");
            }
            out.push(&self.args.items[i as usize]);
        }
        Some(out)
    }

    /// Convert a parser into a task that saves its output to a [`JoinHandle`]
    fn make_raw_task<T: 'static>(
        self: &Ctx<'p>,
        parser: &'p impl Parser<Output = T>,
    ) -> (JoinHandle<T>, Pin<Box<impl Future<Output = ()> + 'p>>) {
        let (out, handle) = make_chan();
        (handle, self.make_act(out, parser))
    }

    fn make_act<T: 'static>(
        self: &Ctx<'p>,
        out: ExitHandle<T>,
        parser: &'p impl Parser<Output = T>,
    ) -> Pin<Box<impl Future<Output = ()> + 'p>> {
        let ctx = self.clone();
        Box::pin(async move {
            let mut res: Result<T, Error> = parser.eval(ctx.clone()).await;
            let is_final = matches!(res, Ok(_) | Err(Error::Final(_)));
            if ctx.is_outconsumed_leaf(is_final).await {
                res = Err(Error::OUTCONSUMED);
            }
            out.exit(res)
        })
    }

    pub fn spawn<T: 'static>(
        self: &Ctx<'p>,
        kind: Kind,
        parser: &'p impl Parser<Output = T>,
    ) -> JoinHandle<T> {
        let (handle, act) = self.make_raw_task(parser);
        let info = self.make_child_info(kind);
        let task = Task { act, info };
        self.add_task(task);
        handle
    }

    /// Spawn a parser and collect the range it occupies
    fn scoped_spawn<T: 'static>(
        self: &Ctx<'p>,
        parser: &'p impl Parser<Output = T>,
        kind: Kind,
    ) -> (JoinHandle<T>, Scope) {
        let start = Id(self.next_free.get());
        let h = self.spawn(kind, parser);
        let end = Id(self.next_free.get());
        (h, Scope { start, end })
    }

    #[inline(never)]
    fn make_child_info(&self, kind: Kind) -> TaskInfo {
        let mut cur = self.current_task.borrow_mut();
        cur.pending += 1;
        let id = self.next_free.get();
        self.next_free.set(id + 1);
        TaskInfo {
            id: Id(id),
            consumed: 0,
            parent_kind: kind,
            parent_id: Parent(cur.id.0),
            pending: 0,
        }
    }

    #[inline(never)]
    fn add_task(&self, mut task: Task<'p>) {
        *self.wakeup_reason.borrow_mut() = Reason::Pass;
        if self.poll_in_context(&mut task) {
            let mut cur = self.current_task.borrow_mut();
            cur.pending -= 1;
            debug_assert_eq!(task.info.consumed, 0);
        } else {
            self.pending_ops.borrow_mut().push_back(Op::Spawn(task));
        }
    }

    /// Check if task is done
    ///
    /// Polls a [`Task`] with shared [`TaskInfo`] set to the right value
    fn poll_in_context(&self, task: &mut Task) -> bool {
        use std::task::{Context, Waker};
        const NOOP: Context<'static> = Context::from_waker(Waker::noop());
        std::mem::swap(&mut task.info, &mut self.current_task.borrow_mut());
        #[expect(const_item_mutation)]
        let done = task.act.as_mut().poll(&mut NOOP).is_ready();
        std::mem::swap(&mut task.info, &mut self.current_task.borrow_mut());
        done
    }

    /// When called from an Alt node it keeps track of children progress and trims
    /// under-consuming children
    async fn trim_children(&self, mut scopes: Vec<Scope>) {
        loop {
            match *self.wakeup_reason.borrow() {
                Reason::ChildProgress(ref ids) => {
                    scopes.retain(|scope| {
                        let lives = ids.as_slice().iter().any(|id| scope.contains(*id));
                        if !lives {
                            let op = Op::KillScope {
                                scope: *scope,
                                cursor: self.cursor.get(),
                                reason: KillReason::Conflict,
                            };
                            self.pending_ops.borrow_mut().push_back(op);
                        }
                        lives
                    });
                    if scopes.len() == 1 {
                        self.pending_ops.borrow_mut().push_back(Op::DeregisterSum {
                            id: self.current_task.borrow().id,
                        });
                    }
                }
                Reason::Push => {
                    if self.current_task.borrow().pending == 0 {
                        break;
                    }
                }
                _ => break,
            }
            r#yield().await;
        }
    }
}

/// Executor connected to a context
///
/// Created with [`Executor::new`]
struct Executor<'a, 'p> {
    ctx: Ctx<'p>,
    tasks: BTreeMap<Id, Task<'p>>,
    to_wake: Vec<Id>,
    to_propagate: VecDeque<(Id, Parent, u32)>,
    sums: BTreeMap<Id, Scope>,
    triggers: Triggers,
    visited: &'a dyn Visited,
}

impl<'a, 'p> Executor<'a, 'p> {
    fn new(ctx: Ctx<'p>, visited: &'a dyn Visited) -> Self {
        Self {
            ctx,
            tasks: BTreeMap::new(),
            to_wake: Vec::new(),
            to_propagate: VecDeque::new(),
            sums: BTreeMap::new(),
            triggers: Default::default(),
            visited,
        }
    }

    // try to parse a set of short flags `-vvv` as separate flags `-v -v -v`
    fn execute_group(&mut self, group: Group) -> Result<(), Error> {
        let res = self.execute_group_inner(&group.0);
        if res.is_ok() {
            self.ctx.cursor.update(|c| c + 1);
        } else {
            self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
        }
        res
    }

    fn execute_group_inner(&mut self, names: &[char]) -> Result<(), Error> {
        let mut mixer_capacity = Mixer::default();
        for (ix, name) in names.iter().copied().enumerate() {
            // set a wakeup reason, just in case
            *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(Arg::Named {
                name: Name::Short(name),
                value: None,
            });

            let mut mixer = mixer_capacity.reuse_capacity();
            mixer.populate_short_flag(&name, &self.triggers);
            let mut cnt = 0;
            while let Some(next) = mixer.consume_next_item(&self.tasks) {
                cnt += 1;
                let task = &mut self.tasks.get_mut(&next.id).unwrap();
                let r = self.ctx.poll_in_context(task);
                assert!(!r); // this breaks the API
                if task.info.consumed != 1 {
                    // but this is a problem, should generate an error message that we can't parse
                    // current argument...
                    todo!("should have consumed a single item")
                }
                self.to_wake.push(task.info.id);
            }
            if cnt == 0 {
                let mut group = String::with_capacity(names.len() + 1);
                group.push('-');
                group.extend(names.iter());

                let ix = ix as u32 + 1;
                let problem = Problem::OnlyOnceInGroup { group, name, ix };
                return Err(Error::Problem(self.ctx.cursor.get(), problem));
            }
            mixer_capacity = mixer.reuse_capacity();
            self.stage_2(1);
            self.propagate();
            self.process_scheduled();
        }

        Ok(())
    }

    /// Process scheduled operations
    fn process_scheduled(&mut self) {
        while let Some(op) = { self.ctx.pending_ops.borrow_mut().pop_front() } {
            match op {
                Op::Spawn(task) => {
                    let old = self.tasks.insert(task.info.id, task);
                    debug_assert!(old.is_none(), "Duplicated task id?");
                }
                Op::RegisterSum { id, scope } => {
                    self.sums.insert(id, scope);
                }
                Op::KillScope {
                    scope,
                    cursor,
                    reason,
                } => {
                    let old = self.ctx.cursor.replace(cursor);
                    self.kill_in_scope(scope, reason);
                    self.ctx.cursor.set(old);
                }
                Op::DeregisterSum { id } => {
                    self.sums.remove(&id);
                }
                Op::Trigger {
                    change,
                    target,
                    parent,
                    id,
                } => {
                    let triggers = &mut self.triggers;

                    fn remove_from<K: Eq + std::hash::Hash>(
                        map: &mut HashMap<K, PeckingOrder>,
                        name: K,
                        parent: Parent,
                        id: Id,
                    ) {
                        use std::collections::hash_map::Entry;
                        if let Entry::Occupied(mut e) = map.entry(name) {
                            if e.get_mut().remove(parent, id) {
                                e.remove();
                            }
                        } else {
                            todo!("Trying to remove something that isn't there?");
                        }
                    }
                    match (change, target) {
                        (TChange::Add, TTarget::Arg(name)) => {
                            triggers.args.entry(name).or_default().insert(parent, id);
                        }
                        (TChange::Add, TTarget::Flag(name)) => {
                            triggers.flags.entry(name).or_default().insert(parent, id);
                        }
                        (TChange::Add, TTarget::Pos) => triggers.pos.insert(parent, id),
                        (TChange::Add, TTarget::Check(check)) => {
                            triggers.checks.insert(id, (parent, check));
                        }
                        (TChange::Add, TTarget::Literal(name)) => {
                            triggers.literal.entry(name).or_default().insert(parent, id);
                        }
                        (TChange::Remove, TTarget::Arg(name)) => {
                            remove_from(&mut triggers.args, name, parent, id);
                        }
                        (TChange::Remove, TTarget::Flag(name)) => {
                            remove_from(&mut triggers.flags, name, parent, id);
                        }
                        (TChange::Remove, TTarget::Pos) => {
                            triggers.pos.remove(parent, id);
                        }
                        (TChange::Remove, TTarget::Check(_)) => {
                            triggers.checks.remove(&id);
                        }
                        (TChange::Remove, TTarget::Literal(name)) => {
                            remove_from(&mut triggers.literal, name, parent, id);
                        }
                    }
                }
            }
        }
    }

    fn check_autocomplete(&mut self, arg: &OsStr) -> Option<Result<(), Error>> {
        if !self.ctx.args.complete || self.ctx.cursor.get() + 1 != self.ctx.args.len() {
            return None;
        }

        let mut mixer = Mixer::default();
        let arg = lex_os_arg(arg);

        // TODO - populate_prefix doesn't really have to be a method on Mixer
        // it can be a standalone method
        let (reason, mgroup) =
            mixer.populate_prefix(&arg, &self.triggers, self.ctx.strict_pos.get());
        *self.ctx.wakeup_reason.borrow_mut() = Reason::Complete(reason);

        // Normally we traverse each pecking order at once since only sum items can run
        // in parallel, but for autocomplete any item from a prod can run so we'll run
        // orders independent from each other
        let orders = std::mem::take(&mut mixer.pecking);
        for po in orders {
            mixer.pecking.push(po);
            while let Some(item) = mixer.consume_next_item(&self.tasks) {
                self.to_wake.push(item.id);
            }
            mixer.clear();
        }

        if self.to_wake.is_empty() {
            return Some(Err(Error::CompReply(Vec1::default())));
        }
        self.to_wake.sort();
        self.to_wake.dedup();

        while let Some(id) = self.to_wake.pop() {
            let mut task = self.tasks.remove(&id).unwrap();
            let r = self.ctx.poll_in_context(&mut task);
            if task.info.parent_id == Parent(0) {
                continue;
            }
            self.to_propagate
                .push_back((task.info.id, task.info.parent_id, task.info.consumed));
            assert!(r);
        }
        self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
        self.propagate();
        Some(Ok(()))
    }

    fn execute(&mut self) -> Result<(), Error> {
        // Keep running as long as we are making progress:
        // - consuming new items
        // -
        let mut mixer_capacity = Mixer::default();

        loop {
            assert!(self.to_propagate.is_empty());
            self.process_scheduled();

            let Some(front) = self.ctx.args.get(self.ctx.cursor.get()) else {
                break;
            };

            if let Some(out) = self.check_autocomplete(front) {
                return out;
            }

            if !self.ctx.strict_pos.get() && front == "--" {
                self.ctx.strict_pos.set(true);
                self.ctx.cursor.update(|c| c + 1);
                continue;
            }

            // Waking up tasks is done in two stages: during the first stage we wake up
            // all the tasks with matching triggers, during the second stage tasks that don't
            // consume the biggest amount from the first stage are terminated.
            //
            // This allows us to parse things as both flag and an argument (`--foo` | `--foo BAR`)
            // in parallel branches. Longer should win.
            //
            // This can't be done in a single pass for two reasons:
            // - to know the biggest consumer we need to check all the candidates
            // - to release the triggers we need to release the reactor
            let (best_size, mixer, mgroup) = self.stage_1(front, mixer_capacity);
            mixer_capacity = mixer;

            if let Some(group) = mgroup
                && best_size == 0
            {
                self.execute_group(group)?;
                self.propagate();
                continue;
            }

            if best_size == 0
                && let Some(scope) = { self.ctx.early_exit.borrow().last().copied() }
                && self.kill_in_scope(scope, KillReason::NoMatchingInput)
            {
                continue;
            }

            if best_size == 0 {
                self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
                let pos = self.ctx.cursor.get();
                return Err(Error::Problem(pos, self.complain_about(front)));
            }

            self.stage_2(best_size);
            self.propagate();

            self.ctx.cursor.update(|c| c + best_size);
        }

        // terminate all the currently active tasks
        self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
        self.process_scheduled();

        // then do it one more time...
        // If there's no tasks active - this is a very fast no-op,
        // but we could have some tasks spawned by variants of .many
        // Killing them again will ensure `many` sees that parser is
        // not advancing and it will exit, producing the result as expected
        self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
        self.process_scheduled();

        assert!(
            self.tasks.is_empty(),
            "All tasks should be terminated when exiting execution"
        );

        assert!(self.ctx.pending_ops.borrow().is_empty());
        assert!(
            self.ctx
                .pending_ops
                .borrow()
                .iter()
                .all(|po| matches!(po, Op::Trigger { .. }))
        );
        // assert!(
        //     self.sums.is_empty(),
        //     "All sums should be removed, {:?}",
        //     self.sums
        // );

        Ok(())
    }

    fn complain_about(&self, unexpected: &OsStr) -> Problem {
        match lex_os_arg(unexpected) {
            Arg::Named {
                name: unexpected,
                value,
            } => {
                // is it a conflict?
                for conflict in self.ctx.conflicts.borrow().iter() {
                    if let Conflict::Named { pos, name: dropped } = conflict
                        && &unexpected == dropped
                    {
                        return Problem::Conflict {
                            accepted: self.ctx.args[*pos].clone(),
                            unexpected: unexpected.into_owned(),
                        };
                    }
                }

                // is this an only once name or a typo?
                let once = IsAcceptedOnce::new(&unexpected);
                let better = match &unexpected {
                    Name::Short(_) => None,
                    Name::Long(cow) => Some(BetterName::new(cow)),
                };
                let is_ddash = IsDDash::attempt(&unexpected, value.as_ref());
                let mut visitors = (once, better, is_ddash);
                self.visited.vi(&mut visitors);
                if let Some(problem) = visitors.0.into_problem() {
                    return problem;
                }
                if let Some(problem) = visitors.1.and_then(|v| v.into_problem()) {
                    return problem;
                }
                if let Some(problem) = visitors.2.and_then(|v| v.into_problem()) {
                    return problem;
                }
            }
            Arg::Pos {
                value,
                value_as_name: unexpected_name,
            } => {
                // is it a conflict?
                for conflict in self.ctx.conflicts.borrow().iter() {
                    match conflict {
                        Conflict::Named { .. } => {}
                        Conflict::Lit {
                            pos,
                            name: conflicted_name,
                        } => {
                            if unexpected_name.is_some_and(|n| &n == conflicted_name) {
                                return Problem::ConflictPos {
                                    accepted: self.ctx.args[*pos].clone(),
                                    unexpected: value.to_os_string(),
                                };
                            }

                            return Problem::ConflictPos {
                                accepted: self.ctx.args[*pos].clone(),
                                unexpected: value.to_os_string(),
                            };
                        }
                        Conflict::Pos { pos } => {
                            return Problem::ConflictPos {
                                accepted: self.ctx.args[*pos].clone(),
                                unexpected: value.to_os_string(),
                            };
                        }
                    }
                }

                if let Some(target) = value.to_str() {
                    let mut is_command = ValidCommand::new(target);
                    self.visited.vi(&mut is_command);
                    if let Some(problem) = is_command.into_problem() {
                        return problem;
                    }
                }
            }
        }
        Problem::Unconsumed {
            value: unexpected.to_os_string(),
        }
    }

    fn kill_in_scope(&mut self, scope: Scope, reason: KillReason) -> bool {
        let mut advanced = false;
        *self.ctx.wakeup_reason.borrow_mut() = Reason::Kill(reason);
        for (_, mut task) in self
            .tasks
            .extract_if(scope.start..scope.end, |_, t| t.info.pending == 0)
        {
            let done = self.ctx.poll_in_context(&mut task);
            assert!(done);
            if task.info.parent_id == Parent(0) {
                continue;
            }
            advanced = true;
            self.to_propagate
                .push_back((task.info.id, task.info.parent_id, task.info.consumed));
        }
        self.propagate();
        advanced
    }

    /// Run the first stage of the trigger
    ///
    /// Each task indicates how much it will consume if allowed to run till the end
    fn stage_1(
        &mut self,
        front: &'p OsStr,
        mixer_capacity: Mixer<'static>,
    ) -> (u32, Mixer<'static>, Option<Group>) {
        let mut mixer = mixer_capacity.reuse_capacity();
        let arg = if self.ctx.strict_pos.get() {
            Arg::Pos {
                value: front,
                value_as_name: None,
            }
        } else {
            lex_os_arg(front)
        };

        // pre-populate pecking order with any check wakeups that can match
        self.triggers.active_checks.clear();
        for (id, (parent, check)) in self.triggers.checks.iter() {
            if check(front) {
                // Here we rely on checks idempotence, run all the checks
                // at once, collect those that succeed and let usual mixer
                // mechanism to wake up those that will advance.
                // If task won't get a chance to run - we'll try to wake it up later
                // and the check should update the inner state.
                self.triggers.active_checks.insert(*parent, *id);
            }
        }

        let mgroup = mixer.populate(&arg, &self.triggers, self.ctx.strict_pos.get());

        *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(arg);

        let mut best_size = 0;
        while let Some(next) = mixer.consume_next_item(&self.tasks) {
            let task = &mut self.tasks.get_mut(&next.id).unwrap();
            let r = self.ctx.poll_in_context(task);
            assert!(!r);
            self.to_wake.push(next.id);

            best_size = best_size.max(task.info.consumed);
        }
        *self.ctx.current_value.borrow_mut() = None;
        (best_size, mixer.reuse_capacity(), mgroup)
    }

    /// Run the second stage of the trigger
    ///
    /// Depending on how much a task is consuming relative to other tasks
    /// task are either allowed to run or are being terminated
    fn stage_2(&mut self, best_size: u32) {
        for id in self.to_wake.drain(..) {
            let Some(mut task) = self.tasks.remove(&id) else {
                todo!("Second stage got a task id that isn't there?");
            };
            let term = task.info.consumed < best_size;
            *self.ctx.wakeup_reason.borrow_mut() = if term {
                Reason::Kill(KillReason::TooShort)
            } else {
                Reason::Pass
            };

            if self.ctx.poll_in_context(&mut task) {
                let size = if term { 0 } else { task.info.consumed };
                if task.info.parent_id.is_root() {
                    continue;
                }
                self.to_propagate
                    .push_back((task.info.id, task.info.parent_id, size));
            } else {
                unreachable!("Task failed to exit during the second stage?");
            }
        }
    }

    // Propagate trigger results to parents recursively
    fn propagate(&mut self) {
        // part 1, letting sums upstream know what branches are advancing so they can terminate
        // those that don't.
        // not reusing capacity, in most cases it will be exactly one branch advancing
        let mut advancing = Vec1::default();
        for (id, _parent, consumed) in self.to_propagate.iter().copied() {
            if consumed > 0 {
                advancing.push(id)
            }
        }
        *self.ctx.wakeup_reason.borrow_mut() = Reason::ChildProgress(advancing.clone());
        // This should be cheap. `advancing` should have just one item, notifying
        // iterates through all the sums in the current level - should be small and we
        // are not even checking all of them
        for id in advancing.as_slice().iter().copied() {
            for (sid, range) in self.sums.iter() {
                if range.contains(id) {
                    let task = self.tasks.get_mut(sid).unwrap();
                    let r = self.ctx.poll_in_context(task);
                    assert!(!r);
                }
                // we don't have to go though the whole sums set, once
                // sum ids start after current id - they can't possibly contain the item.
                if *sid > id {
                    break;
                }
            }
        }

        // part 2, pushing parsed results up
        *self.ctx.wakeup_reason.borrow_mut() = Reason::Push;
        while let Some((_id, parent, consumed)) = self.to_propagate.pop_front() {
            let id = parent.as_id();
            let std::collections::btree_map::Entry::Occupied(mut occ_task) = self.tasks.entry(id)
            else {
                todo!("Unknown task {id:?} in propagete?")
            };
            let task = occ_task.get_mut();
            task.info.pending -= 1;
            task.info.consumed += consumed;
            if task.info.pending > 0 {
                continue;
            }
            if self.ctx.poll_in_context(task) {
                let task = occ_task.remove();
                if task.info.parent_id.is_root() {
                    // the parent is root = there's no task to wake up
                    continue;
                }
                self.to_propagate.push_back((
                    task.info.id,
                    task.info.parent_id,
                    task.info.consumed,
                ));
            } else {
                assert!(
                    task.info.pending > 0,
                    "{task:?} didn't exit and didn't spawn children"
                );
            }
        }
    }
}

/// Intermediate state used by mixer to figure out what tasks can run
/// concurrently with tasks that alredy got chance to run
#[derive(Debug, Default)]
struct Tracker {
    seen: BTreeSet<Parent>,
    stack: Vec<Parent>,
}

/// Can this task run concurrently or not?
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum WalkResult {
    /// Pick this value, next value can be in the same family
    PickAndSame,
    /// Pick this value, next value should be in the different family
    PickAndNext,
    /// Skip this value, next value should be in the different family
    Skip,
}

impl Tracker {
    /// Check if current item can run concurrently with all the previously seen ones
    ///
    /// The idea is that if current task intersects with previously executed tasks
    /// on a sum - it can run and next sibling can run as well. If it intersects
    /// with a prod - it can't. First task can always run
    fn walk_tasks_up(&mut self, mut cur: Id, tasks: &BTreeMap<Id, Task>) -> WalkResult {
        let blank = self.seen.is_empty();
        self.stack.clear();
        loop {
            let Some(task) = tasks.get(&cur) else {
                debug_assert_eq!(cur, Id(0));
                return WalkResult::PickAndSame;
            };
            if !self.seen.insert(task.info.parent_id) {
                return match (blank, task.info.parent_kind) {
                    (_, Kind::Sum) => WalkResult::PickAndSame,
                    (true, Kind::Prod) => WalkResult::PickAndNext,
                    (false, Kind::Prod) => {
                        for item in self.stack.drain(..) {
                            self.seen.remove(&item);
                        }
                        WalkResult::Skip
                    }
                };
            } else {
                self.stack.push(task.info.parent_id);
            }
            cur = task.info.parent_id.as_id();
        }
    }

    fn clear(&mut self) {
        self.seen.clear();
    }

    fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}
#[cfg(test)]
mod tracker_tests {
    use super::*;
    fn mk_dummy_task(id: u32, parent: u32, parent_kind: Kind) -> (Id, Task<'static>) {
        let id = Id(id);
        let task = Task {
            act: Box::pin(async {}),
            info: TaskInfo {
                id,
                parent_kind,
                parent_id: Parent(parent),
                consumed: 0,
                pending: 0,
            },
        };
        (id, task)
    }
    fn add_dummy_task(tasks: &mut BTreeMap<Id, Task>, id: u32, parent: u32, pkind: Kind) {
        let (id, task) = mk_dummy_task(id, parent, pkind);
        tasks.insert(id, task);
    }

    #[test]
    fn product_left_wins() {
        let mut t = Tracker::default();
        let mut tasks = BTreeMap::default();
        add_dummy_task(&mut tasks, 1, 0, Kind::Prod); // root
        add_dummy_task(&mut tasks, 2, 1, Kind::Prod);
        add_dummy_task(&mut tasks, 3, 1, Kind::Prod);
        assert_eq!(t.walk_tasks_up(Id(2), &tasks), WalkResult::PickAndSame);
        assert_eq!(t.walk_tasks_up(Id(3), &tasks), WalkResult::Skip);
    }

    #[test]
    fn product_left_wins_over_sum() {
        let mut t = Tracker::default();
        let mut tasks = BTreeMap::default();
        add_dummy_task(&mut tasks, 1, 0, Kind::Prod); // root
        add_dummy_task(&mut tasks, 2, 1, Kind::Prod);
        add_dummy_task(&mut tasks, 3, 1, Kind::Prod);
        add_dummy_task(&mut tasks, 4, 3, Kind::Prod);
        add_dummy_task(&mut tasks, 5, 3, Kind::Prod);
        assert_eq!(t.walk_tasks_up(Id(2), &tasks), WalkResult::PickAndSame);
        assert_eq!(t.walk_tasks_up(Id(4), &tasks), WalkResult::Skip);
        assert_eq!(t.walk_tasks_up(Id(5), &tasks), WalkResult::Skip);
    }

    #[test]
    fn sums_are_concurrent() {
        let mut t = Tracker::default();
        let mut tasks = BTreeMap::default();
        add_dummy_task(&mut tasks, 1, 0, Kind::Prod); // root
        add_dummy_task(&mut tasks, 2, 1, Kind::Sum);
        add_dummy_task(&mut tasks, 3, 1, Kind::Sum);
        assert_eq!(t.walk_tasks_up(Id(2), &tasks), WalkResult::PickAndSame);
        assert_eq!(t.walk_tasks_up(Id(3), &tasks), WalkResult::PickAndSame);
    }

    #[test]
    fn sums_can_be_nested_left() {
        let mut t = Tracker::default();
        let mut tasks = BTreeMap::default();
        add_dummy_task(&mut tasks, 1, 0, Kind::Prod); // root
        add_dummy_task(&mut tasks, 2, 1, Kind::Sum);
        add_dummy_task(&mut tasks, 3, 1, Kind::Sum);
        add_dummy_task(&mut tasks, 4, 2, Kind::Sum);
        // assert_eq!(t.walk_tasks_up(Id(2), &tasks), WalkResult::PickAndNext);
        assert_eq!(t.walk_tasks_up(Id(3), &tasks), WalkResult::PickAndSame);
        assert_eq!(t.walk_tasks_up(Id(4), &tasks), WalkResult::PickAndSame);
    }

    #[test]
    fn sums_can_be_nested_right() {
        let mut t = Tracker::default();
        let mut tasks = BTreeMap::default();
        add_dummy_task(&mut tasks, 1, 0, Kind::Prod); // root
        add_dummy_task(&mut tasks, 2, 1, Kind::Sum);
        add_dummy_task(&mut tasks, 3, 2, Kind::Sum);
        add_dummy_task(&mut tasks, 4, 2, Kind::Sum);
        // assert!(t.walk_tasks_up(Id(3), &tasks));
        // assert!(t.walk_tasks_up(Id(4), &tasks));
        // assert_eq!(t.walk_tasks_up(Id(2), &tasks), WalkResult::PickAndNext);
        assert_eq!(t.walk_tasks_up(Id(3), &tasks), WalkResult::PickAndSame);
        assert_eq!(t.walk_tasks_up(Id(4), &tasks), WalkResult::PickAndSame);
    }
}

use pecking::PeckingOrder;

type DynamicOsStrCheck = Rc<dyn Fn(&OsStr) -> bool>;
/// A collection of mappings from values encountered in arguments to
/// tasks scheduled to consume them, in some priority order
#[derive(Default)]
struct Triggers {
    // `-f`, `--foo`
    flags: HashMap<Name<'static>, PeckingOrder>,
    // `-f=bar` `--foo=bar`, `-fbar`
    args: HashMap<Name<'static>, PeckingOrder>,
    // `foo`
    pos: PeckingOrder,
    // `-1`
    checks: BTreeMap<Id, (Parent, DynamicOsStrCheck)>,
    active_checks: PeckingOrder,
    // `a`, `alpha`
    literal: HashMap<Lit<'static>, PeckingOrder>,
}

// reactor - listens for readiness, notifies tasks
// executor - runs ready tasks

impl<'p> RawCtx<'p> {
    pub async fn wait_for_children(&self) {
        if self.current_task.borrow().pending > 0 {
            r#yield().await;
        }
    }
}

impl<'p> RawCtx<'p> {
    /// Create a copy of a context suitable to run an executor
    pub(crate) fn fork<'o>(&'o self, level: Option<&str>) -> Ctx<'o>
    where
        'p: 'o,
    {
        let mut path = self.path.clone();
        if let Some(name) = level {
            path.push(' ');
            path.push_str(name);
        }
        Self::make(
            path,
            self.args,
            self.cursor.get(),
            self.strict_pos.get(),
            self.custom,
        )
    }

    pub(crate) fn new(args: &'p Args, custom: &'p Custom) -> Ctx<'p> {
        Self::make(args.app.clone(), args, 0, false, custom)
    }

    fn make<'o>(
        path: String,
        args: &'o Args,
        cursor: u32,
        strict_pos: bool,
        custom: &'o Custom,
    ) -> Ctx<'o> {
        Rc::new(RawCtx {
            args,
            current_task: Default::default(),
            cursor: Cell::new(cursor),
            early_exit: Default::default(),
            pending_ops: Default::default(),
            next_free: Cell::new(1),
            wakeup_reason: RefCell::new(Reason::Pass),
            conflicts: Default::default(),
            strict_pos: Cell::new(strict_pos),
            custom,
            path,
            current_value: Default::default(),
        })
    }

    fn execute(
        self: &Ctx<'p>,
        lazy: bool,
        parser: &dyn Visited,
        info: Option<&Info>,
    ) -> Result<(), Error> {
        let r = Executor::new(self.clone(), parser).execute();

        if matches!(r, Err(Error::Problem(_, Problem::Unconsumed { .. }))) {
            if lazy {
                return Ok(());
            }
            let Some(info) = info else { return r };
            let extra = self.custom.create(info.version);
            let ctx = self.fork(None);
            let (handle, act) = ctx.make_raw_task(&extra);
            let info = ctx.make_child_info(Kind::Prod);
            let task = Task { act, info };
            ctx.add_task(task);

            if Executor::new(ctx.clone(), parser).execute().is_ok() {
                match handle.take() {
                    Ok(xtra) => {
                        return Err(Error::Final(match xtra {
                            Extra::Help | Extra::LongHelp => {
                                ParseFailure::Stdout(crate::visitors::help::render_help(
                                    parser,
                                    Some(&extra),
                                    &ctx.path,
                                    xtra == Extra::LongHelp,
                                ))
                            }
                            Extra::Version(v) => ParseFailure::stdout(format!("Version: {v}\n")),
                        }));
                    }
                    Err(e @ Error::Final(_)) => return Err(e),
                    _ => {}
                }
            }
        }
        r
    }
}

/// Allows traversing several different pecking orders
///
/// We need it to run several parsers concurrently
#[derive(Default, Debug)]
struct Mixer<'a> {
    pecking: Vec<&'a PeckingOrder>,
    tracker: Tracker,
    /// We can run multiple parsers concurrently that belong to the same parent
    /// for Sum types, but not for Prod types. same_family gets set based on the parent
    same_family: bool,
    prev: Option<pecking::Item>,
}

#[must_use]
#[derive(Debug)]
/// A collection of short flags we try to fall back to if we can't parse it as an argument
///
/// Created automatically by [`Mixer`]
struct Group(Vec<char>);

impl<'a> Mixer<'a> {
    fn populate_short_flag(&mut self, name: &char, triggers: &'a Triggers) {
        self.pecking_push(triggers.flags.get(&Name::Short(*name)));
    }

    fn clear(&mut self) {
        self.tracker.clear();
        self.prev = None;
        self.pecking.clear();
        self.tracker.clear();
    }

    fn populate_prefix(
        &mut self,
        arg: &Arg<'a>,
        triggers: &'a Triggers,
        strict_pos: bool,
    ) -> (CompleteReq, Option<Group>) {
        // TODO - want to include all the "any" parsers here
        // self.pecking_push(Some(&triggers.any));
        match arg {
            Arg::Named {
                name: Name::Long(name),
                value: None,
            } => {
                for (n, po) in triggers.args.iter() {
                    let Name::Long(n) = n else {
                        continue;
                    };
                    if n.starts_with(name.as_ref()) {
                        self.pecking_push(Some(po));
                    }
                }
                for (n, po) in triggers.flags.iter() {
                    let Name::Long(n) = n else {
                        continue;
                    };
                    if n.starts_with(name.as_ref()) {
                        self.pecking_push(Some(po));
                    }
                }
                let prefix = Some(name.as_ref().into());
                (CompleteReq::Name { prefix }, None)
            }
            Arg::Named {
                name: name @ Name::Short(_),
                value: None,
            } => {
                self.pecking_push(triggers.args.get(name));
                self.pecking_push(triggers.flags.get(name));
                (CompleteReq::Name { prefix: None }, None)
            }
            Arg::Named {
                name,
                value: Some((adj, val)),
            } => {
                self.pecking.clear(); // wipe triggers.any since they can't fire // TODO - why?
                self.pecking_push(triggers.args.get(name));
                let req = match val.to_str() {
                    Some(p) => CompleteReq::Literal { prefix: p.into() },
                    None => todo!(),
                };
                (req, None)
            }
            Arg::Pos {
                value,
                value_as_name: _,
            } => {
                let Some(value) = value.to_str() else {
                    todo!("Completing non-utf?");
                };
                let (short, long, lit);
                let request;
                if value.is_empty() {
                    (short, long, lit) = (true, true, true);
                    request = CompleteReq::Anything;
                } else if value == "-" {
                    (short, long, lit) = (true, true, false);
                    request = CompleteReq::Anything;
                } else if value == "--" {
                    (short, long, lit) = (false, true, false);
                    request = CompleteReq::Name {
                        prefix: Some(Default::default()),
                    };
                } else {
                    (short, long, lit) = (false, false, true);
                    request = CompleteReq::Literal {
                        prefix: value.into(),
                    }
                }
                if short && !strict_pos {
                    for (n, f) in triggers.args.iter() {
                        if matches!(n, Name::Short(_)) {
                            self.pecking_push(Some(f));
                        }
                    }
                    for (n, f) in triggers.flags.iter() {
                        if matches!(n, Name::Short(_)) {
                            self.pecking_push(Some(f));
                        }
                    }
                }
                // TODO - can avoid iterating twice here
                if long && !strict_pos {
                    for (n, f) in triggers.args.iter() {
                        if matches!(n, Name::Long(_)) {
                            self.pecking_push(Some(f));
                        }
                    }
                    for (n, f) in triggers.flags.iter() {
                        if matches!(n, Name::Long(_)) {
                            self.pecking_push(Some(f));
                        }
                    }
                }
                if lit && !strict_pos {
                    for (name, f) in triggers.literal.iter() {
                        if name.starts_with(value) {
                            self.pecking_push(Some(f));
                        }
                    }
                }
                self.pecking_push(Some(&triggers.pos));

                (request, None)
            }
        }
    }

    fn populate(
        &mut self,
        arg: &Arg<'a>,
        triggers: &'a Triggers,
        strict_pos: bool,
    ) -> Option<Group> {
        self.pecking_push(Some(&triggers.active_checks));
        match arg {
            Arg::Named {
                name: name @ Name::Long(_),
                value: _,
            } => {
                self.pecking_push(triggers.args.get(name));
                self.pecking_push(triggers.flags.get(name));
            }
            Arg::Named {
                name: name @ Name::Short(_),
                value: None | Some((Adjacency::WithEq, _)),
            } => {
                self.pecking_push(triggers.args.get(name));
                self.pecking_push(triggers.flags.get(name));
            }
            Arg::Named {
                name: name @ Name::Short(short_name),
                value: Some((Adjacency::Immediate, val)),
            } => {
                let prev_len = self.pecking.len();
                self.pecking_push(triggers.args.get(name));
                if triggers.flags.contains_key(name)
                    && self.pecking.len() == prev_len
                    && let Some(chars) = val.to_str()
                    && chars
                        .chars()
                        .all(|k| triggers.flags.contains_key(&Name::Short(k)))
                {
                    // There's no way to parse it as a single argument, but it might work
                    // if we treat it as a merged group of single letter flags

                    let mut group = vec![*short_name];
                    group.extend(chars.chars());
                    return Some(Group(group));
                } else {
                    self.pecking_push(triggers.flags.get(name));
                }
            }
            Arg::Pos {
                value: _,
                value_as_name: name,
            } => {
                if let Some(name) = name
                    && !strict_pos
                {
                    self.pecking_push(triggers.literal.get(name));
                }
                self.pecking_push(Some(&triggers.pos));
            }
        }
        None
    }

    /// Add a nonempty pecking order to the mix
    ///
    /// We'll often have empty pecking orders
    fn pecking_push(&mut self, order: Option<&'a PeckingOrder>) {
        if let Some(order) = order
            && !order.is_empty()
        {
            self.pecking.push(order);
        }
    }

    fn first(&self) -> Option<pecking::Item> {
        let mut res = None;
        for order in self.pecking.iter().filter_map(|v| v.peek()) {
            res = Some(res.map_or(order, |old| order.min(old)));
        }
        res
    }
}

impl<'a> Mixer<'a> {
    /// Reuse the capacity inside the executor while decoupling the lifetimes
    fn reuse_capacity<'b>(mut self) -> Mixer<'b> {
        self.tracker.clear();
        Mixer {
            pecking: reuse_vec(self.pecking),
            tracker: self.tracker,
            same_family: false,
            prev: None,
        }
    }

    fn consume_next_item(&mut self, tasks: &BTreeMap<Id, Task>) -> Option<pecking::Item> {
        loop {
            if let Some(prev) = self.prev {
                let prev = if self.same_family || self.tracker.is_empty() {
                    prev
                } else {
                    prev.next_family_item()
                };

                // find best possible item while throwing away orders that can't possibly produce
                // anything better
                let mut best_candidate = None;
                self.pecking.retain(|po| {
                    let Some(item) = po.item_after(prev).copied() else {
                        return false;
                    };
                    match best_candidate {
                        Some(pmin) if item >= pmin => false,
                        _ => {
                            best_candidate = Some(item);
                            true
                        }
                    }
                });
                let best_candidate = best_candidate?;
                if self.tracker.is_empty() && !self.same_family {
                    self.tracker.walk_tasks_up(prev.id, tasks);
                }
                self.prev = Some(best_candidate);
                match self.tracker.walk_tasks_up(best_candidate.id, tasks) {
                    WalkResult::PickAndSame => {
                        self.same_family = true;
                        break;
                    }
                    WalkResult::PickAndNext => {
                        self.same_family = false;
                        break;
                    }
                    WalkResult::Skip => continue,
                }
            } else {
                self.prev = self.first();
                break;
            }
        }
        self.prev
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
enum Name<'a> {
    Short(char),
    Long(Cow<'a, str>),
}

impl std::fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::Short(s) => write!(f, "-{s}"),
            Name::Long(l) => write!(f, "--{l}"),
        }
    }
}

impl Name<'_> {
    pub(crate) fn into_owned(self) -> Name<'static> {
        match self {
            Name::Short(s) => Name::Short(s),
            Name::Long(cow) => Name::Long(Cow::Owned(cow.into_owned())),
        }
    }
}

impl From<char> for Name<'static> {
    fn from(value: char) -> Self {
        Name::Short(value)
    }
}

impl<'a> From<&'a str> for Name<'a> {
    fn from(value: &'a str) -> Self {
        Name::Long(Cow::Borrowed(value))
    }
}

impl From<String> for Name<'static> {
    fn from(value: String) -> Self {
        Name::Long(Cow::Owned(value))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
/// A newtype wrapper for [`Name`] to make it a [`Literal`] name instead
///
/// So no `-` or `--` for [`Display`](std::fmt::Display) and a helper
pub struct Lit<'a>(Name<'a>);
impl Lit<'_> {
    fn into_owned(self) -> Lit<'static> {
        Lit(self.0.into_owned())
    }

    /// Check if value is a valid prefix for Name
    fn starts_with(&self, value: &str) -> bool {
        let mut b = [0; 4];
        match &self.0 {
            Name::Short(c) => c.encode_utf8(&mut b),
            Name::Long(cow) => cow.as_ref(),
        }
        .starts_with(value)
    }
}

impl PartialEq<OsStr> for Lit<'_> {
    fn eq(&self, other: &OsStr) -> bool {
        let mut buf = [0; 4];
        match &self.0 {
            Name::Short(s) => s.encode_utf8(&mut buf) == other,
            Name::Long(cow) => cow.as_ref() == other,
        }
    }
}

impl std::fmt::Display for Lit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Name::Short(s) => f.write_char(*s),
            Name::Long(l) => f.write_str(l),
        }
    }
}

/// Parser that produces a failure with a given message
///
/// Created with [`fail`] or [`success`], useful for custom error messages or things
/// like custom `--version` flags, designed to be used with [`Parser::or_exit`] and
/// [`Parser::then_exit`]
pub struct Exit<T> {
    ctx: PhantomData<T>,
    code: i32,
    msg: Styled,
}
impl<T> Exit<T> {
    fn to_error(&self) -> Error {
        let msg = self.msg.clone();
        Error::Final(if self.code == 0 {
            ParseFailure::Stdout(msg)
        } else {
            ParseFailure::Stderr(msg)
        })
    }
}

impl<T: 'static> Parser for Exit<T> {
    type Output = T;
    fn eval(&self, _: Ctx) -> impl Future<Output = Result<T, Error>> {
        std::future::ready(Err(self.to_error()))
    }

    fn visit<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}

pub fn fail<T>(msg: impl Into<Styled>) -> Exit<T> {
    Exit {
        ctx: PhantomData,
        code: 1,
        msg: msg.into(),
    }
}

pub fn success<T>(msg: impl Into<Styled>) -> Exit<T> {
    Exit {
        ctx: PhantomData,
        code: 0,
        msg: msg.into(),
    }
}

pub struct Cargo<P> {
    name: &'static str,
    inner: P,
}
pub fn cargo_helper<P>(name: &'static str, inner: P) -> Cargo<P> {
    Cargo { name, inner }
}

impl<P: Parser> Parser for Cargo<P> {
    type Output = P::Output;

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<Self::Output, Error>> {
        if ctx.args.get(0).is_some_and(|v| v == self.name) {
            ctx.cursor.update(|c| c + 1);
        }
        self.inner.eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

#[cfg(test)]
mod tests {
    mod algebra;
    mod any;
    mod complete;
    mod construct;
    mod errors;
    mod help;
    mod nested;
    mod offset;
    mod osstring;
    mod parse;
    mod pure_with;
    mod repeat;
    mod sum;
    mod unsorted;
    mod usage;
    // mod wake;
}
