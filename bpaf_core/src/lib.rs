#![warn(clippy::unused_async)]
mod adapters;
mod anything;
mod arg;
mod args;
pub mod complete;
mod completions;
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
mod tasks;
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
            Ctx, Kind, Parser, Scope,
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
                let id = ctx.shared.current_task.borrow().id;
                let mut scopes = Vec::with_capacity(self.items.len());
                let mut handles = Vec::with_capacity(self.items.len());
                for parser in &self.items {
                    let (h, scope) = ctx.scoped_spawn(parser, Kind::Sum);
                    scopes.push(scope);
                    handles.push(h);
                }
                ctx.shared.sums.borrow_mut().insert(
                    id,
                    Scope {
                        start: id,
                        end: scopes.last().unwrap().end,
                    },
                );
                ctx.wait_for_children().await; // give children a chance to start
                ctx.all_children_finish(scopes).await;
                ctx.shared.sums.borrow_mut().remove(&id);

                let mut acc = Error::Silent("Empty Sum?");

                let consumed = ctx.shared.current_task.borrow().consumed > 0;

                // prefer final error if present or the earliest result
                let mut val = None;
                for h in handles {
                    match h.take() {
                        Err(err) => acc = acc.combine(err, Kind::Sum),
                        Ok(v) if consumed => return Ok(v),
                        Ok(v) => val = val.or(Some(v)),
                    }
                }

                // If the succeeding parser didn't consume a value - prefer to return an accumulated
                // error instead. We are dealing with a fallback-like case here. Such cases should
                // only handle "missing" style errors.
                //
                // Sample scenarios where this branch is taken:
                // - a sub-parser handles `--help`. It fails with `Error::Final` and we don't want
                //   to discard it if there's an alternative branch that succeeds without consuming
                //   anything: input that leads to the sub-parser is still there and needs to be
                //   consumed.
                // - in one branch an argument parser fails to consume the trigger due to missing
                //   value, a different parser in a concurrent branch succeeds without consuming
                //   anything. We want the error from the argument parser, otherwise the whole parser
                //   will fail to make progress and we'll have to produce an error explaining the
                //   unparsed trigger part of the argument.
                if !consumed && matches!(&acc, Error::Final(_) | Error::Problem(_, _)) {
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
    SharedCtx,
    arg::{Adjacency, Arg, lex_os_arg},
    args::Args,
    complete::CReq,
    console_writer::Styled,
    info::{Custom, Extra, Info},
    pecking::Mixer,
    utils::Vec1,
    visitors::{errors::IsInCommand, help::render_help},
};

#[doc(inline)]
pub use crate::{
    adapters::OptionParser,
    anything::{any, any_from_str},
    completions::CompHelp,
    console_writer::{Colorscheme, Style},
    consumers::*,
    help_cmd::help_command,
    traits::{Parser, Visited},
};

use crate::{
    error::{Error, ParseFailure, Problem},
    pecking::PeckingOrder,
    traits::{RcParser, VisitGroup, Visitor, *},
};

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
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
pub struct ExitHandle<T> {
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

#[derive(Clone)]
enum TTarget {
    Arg(Name<'static>),
    Flag(Name<'static>),
    Pos,
    Literal(Lit<'static>),
}

impl std::fmt::Debug for TTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arg(arg0) => f.debug_tuple("Arg").field(arg0).finish(),
            Self::Flag(arg0) => f.debug_tuple("Flag").field(arg0).finish(),
            Self::Pos => write!(f, "Pos"),
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
/// [`SharedCtx::current_task`]
#[derive(Debug, Clone, Copy)]
struct TaskInfo {
    /// Current task Id
    id: Id,
    parent_kind: Kind,
    parent_id: Parent,
    /// For trigger tasks having it nonzero means the task is done
    /// and won't be consuming anything
    consumed: u32,
    /// number of children this task is currently waiting for, we are going to wake them up only if
    // there's no pending children - if we can return immediately
    pending: u32,
    state: TaskState,
    /// If true, this task (and all its children) use the global trigger set
    global: bool,
}

/// Tracks tasks state
///
/// Updates once, right before storing result in the output handle
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TaskState {
    Pending,
    Success,
    Failure,
}

impl From<bool> for TaskState {
    fn from(success: bool) -> Self {
        if success {
            Self::Success
        } else {
            Self::Failure
        }
    }
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            id: Id(0),
            parent_kind: Kind::Sum,
            parent_id: Parent(0),
            consumed: 0,
            pending: 0,
            state: TaskState::Pending,
            global: false,
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
    Named {
        pos: u32,
        name: Name<'static>,
        id: Id,
        global: bool,
    },
    /// Literal - command
    Lit {
        pos: u32,
        name: Lit<'static>,
        id: Id,
        global: bool,
    },
    /// Positional item
    Pos { pos: u32, id: Id, global: bool },
    /// An error caught by `.catch()` that should be displayed if there's no alternative
    Caught {
        pos: u32,
        msg: String,
        id: Id,
        global: bool,
    },
}

impl Conflict {
    fn id(&self) -> Id {
        match self {
            Conflict::Named { id, .. }
            | Conflict::Lit { id, .. }
            | Conflict::Pos { id, .. }
            | Conflict::Caught { id, .. } => *id,
        }
    }
    fn global(&self) -> bool {
        match self {
            Conflict::Named { global, .. }
            | Conflict::Lit { global, .. }
            | Conflict::Pos { global, .. }
            | Conflict::Caught { global, .. } => *global,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum KillReason {
    /// Terminate this parser, record a conflict
    Conflict,
    NoMatchingInput,
    /// Terminate this parser without recording a conflict.
    ///
    /// This one is used when multiple parsers respond to the
    /// same trigger and some consume more than others. Usually it's
    /// `--foo` as a flag vs `--foo bar` as an argument.
    /// Recording conflict here will result in an unhelpful
    /// "`--foo` cannot be used with `--foo`"
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

    Complete(complete::ShellRender, Vec<CReq<'p>>),
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
    #[inline(never)]
    /// is_final includes both "Ok" result and an error that can be caught.
    ///
    /// TODO: this whole function is a mess, starting from a name and up to
    /// control flow...
    pub(crate) async fn is_outconsumed_leaf(&self, success: bool) -> bool {
        // If killed by TooShort or Conflict, it means another branch consumed and this one was
        // killed as a result. Mark as outconsumed if parser succeeded.
        // Note: NoMatchingInput is NOT outconsumed - it just means there's no more input.
        if matches!(
            &*self.shared.wakeup_reason.borrow(),
            Reason::Kill(KillReason::TooShort) | Reason::Kill(KillReason::Conflict)
        ) {
            let mut st = self.shared.current_task.borrow_mut();
            st.state = TaskState::Failure;
            st.consumed = 0;
            return success;
        }

        // The only possible scenario for a consuming leaf is getting an `Arg`
        if !matches!(&*self.shared.wakeup_reason.borrow(), Reason::Arg(_)) {
            self.shared.current_task.borrow_mut().state = TaskState::from(success);
            return false;
        }
        // if parser fails to produce a result due to validation or `FromStr`
        // we reset `consumed` back to zero so more conservative parallel branches
        // that did manage to produce a result can succeed
        if !success {
            self.shared.current_task.borrow_mut().consumed = 0;
        }

        r#yield().await; // between stage_1 and stage_2
        // surviving gets Pass, one that consumed less - `NoPass`, but we want to preserve the
        // error message so return true only when there's an OK result we don't want.
        let killed = matches!(
            &*self.shared.wakeup_reason.borrow(),
            Reason::Kill(KillReason::TooShort)
        );

        let mut st = self.shared.current_task.borrow_mut();
        st.state = TaskState::from(success && !killed);

        if killed {
            st.consumed = 0;
        }

        killed && success
    }

    /// After consumption when called from a leaf node returns what was consumed
    pub(crate) fn leaf_range(&self) -> Option<(u32, u32)> {
        if !matches!(&*self.shared.wakeup_reason.borrow(), Reason::Arg(_)) {
            return None;
        }
        let start = self.cursor().get();
        let end = start + self.shared.current_task.borrow().consumed;
        (end > start).then_some((start, end))
    }

    pub(crate) fn leaf_cursor(&self) -> u32 {
        if matches!(&*self.shared.wakeup_reason.borrow(), Reason::Arg(_)) {
            self.cursor().get()
        } else {
            u32::MAX
        }
    }

    /// After consumption, when called from a leaf node returns what was consumed
    /// as a string
    pub(crate) fn leaf_consumed(&self) -> Option<String> {
        let (start, end) = self.leaf_range()?;
        let mut out = String::new();
        for (ix, i) in (start..end).enumerate() {
            if ix > 0 {
                out.push(' ');
            }
            out.push_str(&self.shared.args.items[i as usize].to_string_lossy());
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

            // For global parsers we need to stash the error in case it fails
            // to parse in the inner context so inner context can access it.
            // Otherwise, the value remains unconsumed and the inner context
            // complains about it in a confusing way
            if let Err(Error::Problem(pos, p)) = &res
                && ctx.current_value.borrow().is_some()
                && let cur = ctx.shared.current_task.borrow()
                && cur.global
            {
                let problem = Conflict::Caught {
                    pos: *pos,
                    msg: p.to_string(),
                    id: cur.id,
                    global: cur.global,
                };
                ctx.shared.conflicts.borrow_mut().push(problem);
            }

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
        let start = Id(self.shared.next_free.get());
        let h = self.spawn(kind, parser);
        let end = Id(self.shared.next_free.get());
        (h, Scope { start, end })
    }

    /// Run a `parser` in an inner executor
    fn run_inner_executor<T: 'static>(
        self: &Ctx<'p>,
        lazy: bool,
        parser: &'p impl Parser<Output = T>,
        visited: &dyn Visited,
        info: Option<&Info>,
        scope_start: u32,
    ) -> Result<T, Error> {
        let saved_current = self.shared.current_task.replace(TaskInfo::default());
        let (handle, act) = self.make_raw_task(parser);
        let task_info = self.make_child_info(Kind::Prod);
        self.add_task(Task {
            act,
            info: task_info,
        });
        let executor_res = self.execute(lazy, visited, info, scope_start);
        self.shared.current_task.replace(saved_current);
        let res = handle.take();
        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1.with_executor(e2)),
        }
    }

    #[inline(never)]
    fn make_child_info(&self, kind: Kind) -> TaskInfo {
        let mut cur = self.shared.current_task.borrow_mut();
        cur.pending += 1;
        let id = self.shared.next_free.get();
        self.shared.next_free.set(id + 1);
        TaskInfo {
            id: Id(id),
            consumed: 0,
            parent_kind: kind,
            parent_id: Parent(cur.id.0),
            pending: 0,
            state: TaskState::Pending,
            global: cur.global,
        }
    }

    #[inline(never)]
    fn add_task(&self, mut task: Task<'p>) {
        let prev = std::mem::replace(&mut *self.shared.wakeup_reason.borrow_mut(), Reason::Pass);
        if self.poll_in_context(&mut task) {
            *self.shared.wakeup_reason.borrow_mut() = prev;
            let mut cur = self.shared.current_task.borrow_mut();
            cur.pending -= 1;
        } else {
            *self.shared.wakeup_reason.borrow_mut() = prev;
            let old = self.shared.tasks.borrow_mut().insert(task.info.id, task);
            debug_assert!(old.is_none(), "Duplicated task id?");
        }
    }

    /// Check if task is done
    ///
    /// Polls a [`Task`] with shared [`TaskInfo`] set to the right value.
    /// For global tasks, swaps local and global trigger sets so trigger
    /// operations operate on the global set.
    fn poll_in_context(&self, task: &mut Task) -> bool {
        use std::task::{Context, Waker};
        const NOOP: Context<'static> = Context::from_waker(Waker::noop());

        std::mem::swap(&mut task.info, &mut self.shared.current_task.borrow_mut());
        #[expect(const_item_mutation)]
        let done = task.act.as_mut().poll(&mut NOOP).is_ready();
        std::mem::swap(&mut task.info, &mut self.shared.current_task.borrow_mut());

        done
    }
}

use crate::tasks::Tasks;

/// Executor connected to a context
///
/// Created with [`Executor::new`]
struct Executor<'a, 'p> {
    ctx: Ctx<'p>,
    scope_start: Id,
    to_wake: Vec<Id>,
    to_propagate: VecDeque<TaskInfo>,
    mixer: Mixer,
    visited: &'a dyn Visited,
    /// Is this a full parser with help and everything?
    ///
    /// Set to `true` for command parsers and similar,
    /// Set to `false` for nested parsers.
    ///
    /// global parsers run only in full parsers
    full_parser: bool,
}

impl<'a, 'p> Executor<'a, 'p> {
    fn new(ctx: Ctx<'p>, visited: &'a dyn Visited, scope_start: Id, full_parser: bool) -> Self {
        Self {
            ctx,
            scope_start,
            to_wake: Vec::new(),
            to_propagate: VecDeque::new(),
            visited,
            mixer: Default::default(),
            full_parser,
        }
    }

    fn current_scope(&self) -> Scope {
        Scope {
            start: self.scope_start,
            end: Id(self.ctx.shared.next_free.get()),
        }
    }

    // try to parse a set of short flags `-vvv` as separate flags `-v -v -v`
    fn execute_group(&mut self, group: Group) -> Result<(), Error> {
        let res = self.execute_group_inner(&group.0);
        if res.is_ok() {
            self.ctx.cursor().update(|c| c + 1);
        } else {
            self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
        }
        res
    }

    fn execute_group_inner(&mut self, names: &[char]) -> Result<(), Error> {
        for (ix, name) in names.iter().copied().enumerate() {
            // set a wakeup reason, just in case
            *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Arg(Arg::Named {
                name: Name::Short(name),
                value: None,
            });

            self.mixer
                .populate_short_flag(&name, &self.ctx.triggers.borrow());
            let mut cnt = 0;
            let ids = self.mixer.for_wake(&self.ctx.shared.tasks.borrow());
            for id in ids {
                cnt += 1;
                let mut task = self
                    .ctx
                    .shared
                    .tasks
                    .borrow_mut()
                    .remove(id)
                    .unwrap_or_else(|| panic!("Task {id:?} not found in group"));
                let r = self.ctx.poll_in_context(&mut task);
                assert!(!r); // this breaks the API
                if task.info.consumed != 1 {
                    // but this is a problem, should generate an error message that we can't parse
                    // current argument...
                    todo!("should have consumed a single item")
                }
                self.to_wake.push(task.info.id);
                self.ctx.shared.tasks.borrow_mut().insert(id, task);
            }
            if cnt == 0 {
                let mut group = String::with_capacity(names.len() + 1);
                group.push('-');
                group.extend(names.iter());

                let ix = ix as u32 + 1;
                let problem = Problem::OnlyOnceInGroup { group, name, ix };
                return Err(Error::Problem(self.ctx.cursor().get(), problem));
            }

            self.stage_2(1);
            self.propagate();
            self.process_scheduled();
        }

        Ok(())
    }

    /// Process scheduled operations
    fn process_scheduled(&mut self) {
        while let Some(op) = { self.ctx.shared.pending_ops.borrow_mut().pop_front() } {
            match op {
                Op::KillScope {
                    scope,
                    cursor,
                    reason,
                } => {
                    let old = self.ctx.cursor().replace(cursor);
                    self.kill_in_scope(scope, reason);
                    self.ctx.cursor().set(old);
                }
            }
        }
    }

    fn execute(&mut self) -> Result<(), Error> {
        let mut prev_pos = self.ctx.cursor().get();
        let mut duds = 0;

        // Save wakeup_reason so that nested executor runs don't leak
        // their modifications back to the parent.
        let saved_reason = std::mem::replace(
            &mut *self.ctx.shared.wakeup_reason.borrow_mut(),
            Reason::Pass,
        );

        // We'll stop once there's no more data or no more candidates to run
        loop {
            // If we want to track progress by having pending parsers - we need to
            // also avoid loops where the same parser gets spawned and consumes nothing.
            // Currently all the repeated parsers makes sure to handle this, but this
            // makes a good sanity check.
            if prev_pos == self.ctx.cursor().get() {
                duds += 1;
                if duds == 10 {
                    let front = &self.ctx.shared.args[self.ctx.cursor().get()];
                    unreachable!(
                        "We made a lot of iterations trying to parse {front:?} but made no progress. \
                                  This shouldn't be reachable, please report it as a bug."
                    );
                }
            } else {
                duds = 0;
            }
            prev_pos = self.ctx.cursor().get();

            assert!(self.to_propagate.is_empty());
            self.process_scheduled();

            let Some(front) = self.ctx.shared.args.get(self.ctx.cursor().get()) else {
                break;
            };

            if let Some(out) = self.check_autocomplete(front) {
                *self.ctx.shared.wakeup_reason.borrow_mut() = saved_reason;
                return out;
            }

            if !self.ctx.strict_pos.get() && front == "--" {
                self.ctx.strict_pos.set(true);
                self.ctx.cursor().update(|c| c + 1);
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
            let (best_size, mgroup) = self.stage_1(front);

            if let Some(group) = mgroup
                && self.to_wake.is_empty()
            {
                self.execute_group(group)?;
                self.propagate();
                continue;
            }

            if self.to_wake.is_empty()
                && let Some(scope) = { self.ctx.early_exit.borrow().last().copied() }
                && self.kill_in_scope(scope, KillReason::NoMatchingInput)
            {
                continue;
            }

            if self.to_wake.is_empty() {
                self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
                let pos = self.ctx.cursor().get();
                *self.ctx.shared.wakeup_reason.borrow_mut() = saved_reason;
                return Err(Error::Problem(pos, self.complain_about(front)));
            }

            self.stage_2(best_size);
            self.propagate();

            self.ctx.cursor().update(|c| c + best_size);
        }

        // terminate all the currently active tasks
        self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
        self.process_scheduled();

        assert!(self.ctx.shared.pending_ops.borrow().is_empty());
        self.ctx
            .shared
            .tasks
            .borrow_mut()
            .assert_no_tasks_past_end(self.scope_start.0);

        {
            let sums = self.ctx.shared.sums.borrow();
            let in_scope: Vec<_> = sums
                .keys()
                .filter(|k| self.current_scope().contains(**k))
                .collect();
            assert!(
                in_scope.is_empty(),
                "All sums in executor scope should be removed, {:?}",
                in_scope
            );
        }

        // Clear conflicts created within this executor's scope
        {
            let scope = self.current_scope();
            self.ctx
                .shared
                .conflicts
                .borrow_mut()
                .retain(|c| !scope.contains(c.id()));
        }

        // reset next_free to reclaim the ID range for this executor
        self.ctx.shared.next_free.set(self.scope_start.0);

        *self.ctx.shared.wakeup_reason.borrow_mut() = saved_reason;
        Ok(())
    }

    fn complain_about(&self, unexpected: &OsStr) -> Problem {
        use crate::visitors::errors::{
            BetterName, IsAcceptedOnce, IsDDash, ProblemVisitor, ValidCommand,
        };

        // Check for caught errors first - errors that were stored by .catch()
        let pos = self.ctx.cursor().get();
        let scope = self.current_scope();
        for conflict in self.ctx.shared.conflicts.borrow().iter() {
            if let Conflict::Caught {
                pos: caught_pos,
                msg,
                id,
                global,
            } = conflict
                && *caught_pos == pos
                && (scope.contains(*id) || *global)
            {
                return Problem::Dynamic { err: msg.clone() };
            }
        }

        match lex_os_arg(unexpected) {
            Arg::Named {
                name: unexpected,
                value,
            } => {
                // is it a conflict?
                for conflict in self.ctx.shared.conflicts.borrow().iter() {
                    if let Conflict::Named {
                        pos,
                        name: dropped,
                        id,
                        global,
                    } = conflict
                        && &unexpected == dropped
                        && (scope.contains(*id) || *global)
                    {
                        return Problem::Conflict {
                            accepted: self.ctx.shared.args[*pos].to_string_lossy().into_owned(),
                            unexpected: unexpected.into_owned(),
                        };
                    }
                }

                // is this an only once name or a typo?
                let mut once = IsAcceptedOnce::new(&unexpected);
                let mut better = match &unexpected {
                    Name::Short(_) => None,
                    Name::Long(cow) => Some(BetterName::new(cow)),
                };
                let mut is_ddash = IsDDash::attempt(&unexpected, value.as_ref());
                let mut vc = IsInCommand::new(&unexpected);

                let mut visitors: Vec<&mut dyn ProblemVisitor<'_>> = Vec::new();
                visitors.push(&mut once);
                if let Some(b) = better.as_mut() {
                    visitors.push(b);
                }
                if let Some(d) = is_ddash.as_mut() {
                    visitors.push(d);
                }
                visitors.push(&mut vc);

                self.visited.vi(&mut visitors);
                if let Some(problem) = visitors.iter().find_map(|v| v.see_problem()) {
                    return problem;
                }
            }
            Arg::Pos { value } => {
                let unexpected_name = arg::as_name(value);
                // is it a conflict?
                for conflict in self.ctx.shared.conflicts.borrow().iter() {
                    if !scope.contains(conflict.id()) && !conflict.global() {
                        continue;
                    }
                    match conflict {
                        Conflict::Named { .. } => {}
                        Conflict::Caught { .. } => {}
                        Conflict::Lit {
                            pos,
                            name: conflicted_name,
                            ..
                        } => {
                            if unexpected_name
                                .as_ref()
                                .is_some_and(|n| n == conflicted_name)
                            {
                                return Problem::ConflictPos {
                                    accepted: self.ctx.shared.args[*pos]
                                        .to_string_lossy()
                                        .into_owned(),
                                    unexpected: value.to_string_lossy().into_owned(),
                                };
                            }
                        }
                        Conflict::Pos { pos, .. } => {
                            return Problem::ConflictPos {
                                accepted: self.ctx.shared.args[*pos].to_string_lossy().into_owned(),
                                unexpected: value.to_string_lossy().into_owned(),
                            };
                        }
                    }
                }

                if let Some(target) = value.to_str() {
                    let mut is_command = ValidCommand::new(target);
                    self.visited.vi(&mut is_command);
                    if let Some(problem) = is_command.see_problem() {
                        return problem;
                    }
                }
            }
        }
        Problem::Unconsumed {
            value: unexpected.to_string_lossy().into_owned(),
        }
    }

    /// Terminate all the tasks in `scope` with the `reason`
    ///
    /// It will poll inner tasks as necessary, it won't poll any tasks outside of the scope
    fn kill_in_scope(&mut self, scope: Scope, reason: KillReason) -> bool {
        let mut state = self.ctx.shared.tasks.borrow().drain_tasks_rev(scope);
        let mut killed_anything = false;
        self.propagate();
        self.process_scheduled();
        while let Some(mut task) = { self.ctx.shared.tasks.borrow_mut().drain_next(&mut state) } {
            assert!(self.ctx.shared.pending_ops.borrow().is_empty());
            *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Kill(reason);

            let done = self.ctx.poll_in_context(&mut task);
            killed_anything |= done;
            assert!(done);
            // there's no more pushing to be done
            if task.info.parent_id.is_root() {
                continue;
            }
            let pid = task.info.parent_id.as_id();

            if scope.contains(pid) {
                // Parent is inside the killed scope and will be killed in a later iteration.
                let mut tasks = self.ctx.shared.tasks.borrow_mut();
                let parent = &mut tasks[pid];
                parent.info.pending -= 1;
                parent.info.consumed += task.info.consumed;
            } else {
                // Parent sits outside the killed scope - schedule a re-poll
                // once all its other children are done.
                self.to_propagate.push_back(task.info);
            }
        }
        self.propagate();
        killed_anything
    }

    /// Run the first stage of the trigger
    ///
    /// Each task indicates how much it will consume if allowed to run till the end
    fn stage_1(&mut self, front: &'p OsStr) -> (u32, Option<Group>) {
        let arg = if self.ctx.strict_pos.get() {
            Arg::Pos { value: front }
        } else {
            lex_os_arg(front)
        };

        let mut mgroup = None;

        if self.full_parser {
            mgroup = self.mixer.populate(
                front,
                &arg,
                &self.ctx.shared.global_triggers.borrow(),
                self.ctx.strict_pos.get(),
            );
        }

        if self.mixer.is_empty() && mgroup.is_none() {
            mgroup = self.mixer.populate(
                front,
                &arg,
                &self.ctx.triggers.borrow(),
                self.ctx.strict_pos.get(),
            );
        }

        *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Arg(arg);

        let mut best_size = 0;
        let to_wake = self.mixer.for_wake(&self.ctx.shared.tasks.borrow());
        for id in to_wake {
            let mut task = self
                .ctx
                .shared
                .tasks
                .borrow_mut()
                .remove(id)
                .unwrap_or_else(|| panic!("Task {id:?} not found in stage_1"));
            let r = self.ctx.poll_in_context(&mut task);
            assert!(!r, "task should not finish during this stage");
            self.to_wake.push(id);
            best_size = best_size.max(task.info.consumed);
            self.ctx.shared.tasks.borrow_mut().insert(id, task);
        }
        *self.ctx.current_value.borrow_mut() = None;
        (best_size, mgroup)
    }

    /// Run the second stage of the trigger
    ///
    /// Depending on how much a task is consuming relative to other tasks
    /// task are either allowed to run or are being terminated
    fn stage_2(&mut self, best_size: u32) {
        for id in self.to_wake.drain(..) {
            let Some(mut task) = self.ctx.shared.tasks.borrow_mut().remove(id) else {
                unreachable!("Second stage got a task {id:?} that isn't in tasks?");
            };
            let term = task.info.consumed < best_size;
            *self.ctx.shared.wakeup_reason.borrow_mut() = if term {
                Reason::Kill(KillReason::TooShort)
            } else {
                Reason::Pass
            };

            if self.ctx.poll_in_context(&mut task) {
                let consumed = if term { 0 } else { task.info.consumed };
                if task.info.parent_id.is_root() {
                    // the parent is root = there's no task to wake up
                    continue;
                }
                self.to_propagate.push_back(TaskInfo {
                    consumed,
                    ..task.info
                });
            } else {
                unreachable!("Task failed to exit during the second stage?");
            }
        }
    }

    /// Notify sums about children making progress
    ///
    /// Only sum items that made progress should continue - to avoid the data loss.
    /// When `global_advance` is true, all sums are notified regardless of executor scope.
    fn notify_sums(&mut self, advancing: &Vec1<Id>, global_advance: bool) {
        // This should be cheap. `advancing` should have just one item, notifying
        // iterates through all the sums in the current level - should be small and we
        // are not even checking all of them
        *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::ChildProgress(advancing.clone());
        assert!(self.to_wake.is_empty(), "should be left empty by stage 2");

        let sums = self.ctx.shared.sums.borrow();
        for id in advancing.as_slice().iter() {
            for (sid, range) in sums.iter() {
                if !global_advance && !self.current_scope().contains(*sid) {
                    continue;
                }
                if range.contains(*id) {
                    self.to_wake.push(*sid);
                }
                // we don't have to go though the whole sums set, once
                // sum ids start after current id - they can't possibly contain the item.
                if *sid > *id {
                    break;
                }
            }
        }
        drop(sums);
        for sid in self.to_wake.drain(..) {
            let mut task = self
                .ctx
                .shared
                .tasks
                .borrow_mut()
                .remove(sid)
                .unwrap_or_else(|| panic!("Task {sid:?} not found in notify_sums"));
            let r = self.ctx.poll_in_context(&mut task);
            assert!(!r);
            self.ctx.shared.tasks.borrow_mut().insert(sid, task);
        }
    }

    /// Propagate trigger results to parents recursively
    fn propagate(&mut self) {
        // part 1, letting sums upstream know what branches are advancing so they can terminate
        // those that don't.
        // not reusing capacity, in most cases it will be exactly one branch advancing
        let mut advancing = Vec1::default();
        let mut global_advance = false;
        for info in self.to_propagate.iter().copied() {
            if info.consumed > 0 {
                advancing.push(info.id);
                if info.global {
                    global_advance = true;
                }
            }
        }
        self.notify_sums(&advancing, global_advance);

        // part 2, pushing parsed results up
        *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Push;
        while let Some(info) = self.to_propagate.pop_front() {
            let id = info.parent_id.as_id();
            let mut task = self
                .ctx
                .shared
                .tasks
                .borrow_mut()
                .remove(id)
                .unwrap_or_else(|| panic!("Unknown task {id:?} in propagate?"));
            task.info.pending -= 1;
            task.info.consumed += info.consumed;

            // notifying parent Sum that one of the children is done so other children can be wiped
            if info.state == TaskState::Success
                && info.consumed > 0
                && info.parent_kind == Kind::Sum
                && task.info.pending > 0
            {
                *self.ctx.shared.wakeup_reason.borrow_mut() =
                    Reason::ChildProgress(Vec1::new(info.id));
                self.ctx.poll_in_context(&mut task);
                *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Push;
            };

            if task.info.pending > 0 {
                self.ctx.shared.tasks.borrow_mut().insert(id, task);
                continue;
            }

            if self.ctx.poll_in_context(&mut task) {
                if task.info.parent_id.is_root() {
                    // the parent is root = there's no task to wake up
                    continue;
                }
                self.to_propagate.push_back(task.info);
            } else {
                assert!(
                    task.info.pending > 0,
                    "{task:?} didn't exit and didn't spawn children"
                );
                self.ctx.shared.tasks.borrow_mut().insert(id, task);
            }
        }
    }
}

impl<'p> RawCtx<'p> {
    pub async fn wait_for_children(&self) {
        if self.shared.current_task.borrow().pending > 0 {
            Yield(false).await;
        }
    }
}

impl<'p> RawCtx<'p> {
    /// Create a copy of a context suitable to run an executor
    pub(crate) fn fork(&self, level: Option<&str>) -> Ctx<'p> {
        let mut path = self.path.clone();
        if let Some(name) = level {
            path.push(' ');
            path.push_str(name);
        }
        Self::make(path, self.shared.clone(), self.strict_pos.get())
    }

    pub(crate) fn new(args: &'p Args, custom: &'p Custom, extra: &'p BoxParser<Extra>) -> Ctx<'p> {
        let shared = Rc::new(SharedCtx {
            args,
            custom,
            help_and_version: extra,
            tasks: RefCell::new(Tasks::new()),
            next_free: Cell::new(1),
            wakeup_reason: RefCell::new(Reason::Pass),
            cursor: Cell::new(0),
            global_triggers: Rc::new(RefCell::new(Triggers::default())),
            current_task: RefCell::new(TaskInfo::default()),
            sums: RefCell::new(BTreeMap::new()),
            pending_ops: RefCell::new(VecDeque::new()),
            conflicts: RefCell::new(Vec::new()),
        });
        Self::make(args.app.clone(), shared, false)
    }

    fn make<'o>(path: String, shared: Rc<SharedCtx<'o>>, strict_pos: bool) -> Ctx<'o> {
        Rc::new(RawCtx {
            shared,

            early_exit: Default::default(),

            strict_pos: Cell::new(strict_pos),
            triggers: Default::default(),
            path,
            current_value: Default::default(),
        })
    }

    #[inline(never)]
    fn execute(
        self: &Ctx<'p>,
        lazy: bool,
        parser: &dyn Visited,
        info: Option<&Info>,
        scope_start: u32,
    ) -> Result<(), Error> {
        let r = Executor::new(self.clone(), parser, Id(scope_start), info.is_some()).execute();

        if matches!(r, Err(Error::Problem(_, Problem::Unconsumed { .. }))) {
            if lazy {
                return Ok(());
            }
            if info.is_none() {
                return r;
            };

            let ctx = self.fork(None);
            let (handle, act) = ctx.make_raw_task(ctx.shared.help_and_version);

            let alt_scope_start = ctx.shared.next_free.get();
            // See `OptionParser::run_in_ctx` for why we reset `current_task`
            // around the inner executor.
            let saved_current = ctx.shared.current_task.replace(TaskInfo::default());
            let info = ctx.make_child_info(Kind::Prod);
            let task = Task { act, info };
            ctx.add_task(task);
            assert!(ctx.shared.pending_ops.borrow().is_empty());
            let exec_res = Executor::new(ctx.clone(), parser, Id(alt_scope_start), false).execute();
            assert!(ctx.shared.pending_ops.borrow().is_empty());
            ctx.shared.current_task.replace(saved_current);
            if exec_res.is_ok() {
                match handle.take() {
                    Ok(xtra) => {
                        return Err(Error::Final(match xtra {
                            Extra::Help | Extra::LongHelp => {
                                ParseFailure::Stdout(crate::visitors::help::render_help(
                                    parser,
                                    Some(self.shared.help_and_version),
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

#[must_use]
#[derive(Debug)]
/// A collection of short flags we try to fall back to if we can't parse it as an argument
///
/// Created automatically by [`Mixer`]
struct Group(Vec<char>);

impl Mixer {
    fn populate_short_flag(&mut self, name: &char, triggers: &Triggers) {
        self.pecking_push(triggers.flags.get(&Name::Short(*name)));
    }

    fn populate(
        &mut self,
        front: &OsStr,
        arg: &Arg,
        triggers: &Triggers,
        strict_pos: bool,
    ) -> Option<Group> {
        if !strict_pos {
            // pre-populate pecking order with any check wakeups that can match
            for (id, check) in triggers.checks.iter() {
                if check(front) {
                    // Here we rely on checks idempotence, run all the checks
                    // at once, collect those that succeed and let usual mixer
                    // mechanism to wake up those that will advance.
                    // If task won't get a chance to run - we'll try to wake it up later
                    // and the check should update the inner state.
                    self.push(*id);
                }
            }
        }

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
                let prev_len = self.candidates_len();
                self.pecking_push(triggers.args.get(name));
                if triggers.flags.contains_key(name)
                    && self.candidates_len() == prev_len
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
            Arg::Pos { value } => {
                if let Some(name) = arg::as_name(value)
                    && !strict_pos
                {
                    self.pecking_push(triggers.literal.get(&name));
                }
                self.pecking_push(Some(&triggers.pos));
            }
        }
        None
    }

    /// Add a nonempty pecking order to the mix
    ///
    /// We'll often have empty pecking orders
    fn pecking_push(&mut self, order: Option<&PeckingOrder>) {
        if let Some(order) = order {
            self.push_peck(order);
        }
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
        if ctx.shared.args.get(0).is_some_and(|v| v == self.name) {
            ctx.cursor().update(|c| c + 1);
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
    mod argument;
    mod command;
    mod complete;
    mod construct;
    mod errors;
    mod flag;
    mod git;
    mod global;
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
