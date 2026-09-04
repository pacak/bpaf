#![warn(clippy::unused_async)]
mod adapters;
mod anything;
mod arg;
mod args;
pub mod complete;
mod completions;
mod composite;
mod console_writer;
mod consumers;
mod core_consumers;
mod ctx;
mod custom_help;
pub mod document;
mod error;
pub mod help;
mod help_cmd;
mod helpers;
mod info;
mod macros;
mod miniansi;
mod os_str;
mod pecking;
mod repeat;
mod tasks;
mod traits;
mod utils;
pub mod vault;
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
        pub use crate::{
            adapters::{CustomUsage, OrExit, ThenExit},
            composite::{AndAlso, Sum},
            consumers::Nested,
            traits::Leaf,
        };
    }

    pub mod complete {
        pub use crate::{completions::CompHelp, consumers::WithComplete};
    }

    /// fallback
    pub mod fallback {}

    /// repeat
    pub mod repeat {}

    pub mod functor {}

    pub mod ux {}

    pub mod primitives {
        pub use crate::{
            anything::{AnyCheck, Anything},
            consumers::{
                Argument, ArgumentLike, BasicArgument, Fail, Flag, Keyword, Leftovers, Named,
                NegArgument, OnMissingValue, Positional,
            },
        };
    }

    pub mod algebras {}
    pub mod internal {
        pub use crate::{
            ctx::{Ctx, RawCtx},
            traits::Visited,
        };
    }
    pub mod helpers {
        pub use crate::helpers::Cargo;
    }
}

use crate::{
    arg::{Adjacency, Arg, lex_os_arg},
    args::Args,
    complete::CReq,
    console_writer::Styled,
    consumers::{InnerExit, Literal, Named},
    ctx::{Ctx, Op, RawCtx, Scope, SharedCtx, Triggers},
    error::{Error, ParseFailure, Problem},
    help::Help,
    info::Info,
    os_str::OsStrExt,
    pecking::{Mixer, PeckingOrder},
    tasks::{ExitHandle, Id, JoinHandle, Kind, Parent, Task, TaskInfo, TaskState, Tasks},
    traits::{BoxParser, Item, Leaf, Lit, Metavar, Name, VKind, VisitGroup, Visited, Visitor},
    utils::Vec1,
    visitors::errors::IsInCommand,
};

#[doc(inline)]
pub use crate::{
    adapters::OptionParser,
    anything::{any, any_from_str},
    consumers::{Exit, env, fail, leftovers, literal, long, positional, pure, pure_with, short},
    helpers::cargo_helper,
    traits::Parser,
};

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    ffi::{OsStr, OsString},
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::Poll,
};

#[doc(hidden)]
pub mod __private {
    pub use crate::{
        api::composite::Sum,
        ctx::Ctx,
        error::Error,
        os_str::parse_os_str,
        tasks::{JoinHandle, Kind},
        traits::{BoxParser, Item, Leaf, Parser, VKind, VisitGroup, Visited, Visitor},
    };
    pub use ::std::compile_error;
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
        let executor_res = self.execute(lazy, info, scope_start);
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
            // Task finished immediately without spawning children. Reclaim its Id so tasks
            // can stay a dense vector
            self.shared.next_free.set(task.info.id.0);
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

    /// Poll a task by id, expecting it should not complete
    fn poll_incomplete(&self, id: Id) -> TaskInfo {
        let mut task = self
            .shared
            .tasks
            .borrow_mut()
            .remove(id)
            .unwrap_or_else(|| panic!("Task {id:?} not found"));
        let done = self.poll_in_context(&mut task);
        assert!(!done, "task should not finish during this stage");
        let info = task.info;
        self.shared.tasks.borrow_mut().insert(id, task);
        info
    }
}

/// Tracks forward progress of the executor to detect infinite loops.
///
/// If the cursor does not advance after several iterations, it means we have
/// a parser that spawns children but nothing consumes input - an unreachable
/// invariant violation.
struct ProgressTracker<'p> {
    prev_pos: u32,
    duds: u32,
    ctx: Ctx<'p>,
}

impl<'p> ProgressTracker<'p> {
    fn new(ctx: &Ctx<'p>) -> Self {
        Self {
            prev_pos: ctx.cursor().get(),
            duds: 0,
            ctx: ctx.clone(),
        }
    }

    fn tick(&mut self) {
        let current_pos = self.ctx.cursor().get();
        if self.prev_pos == current_pos {
            self.duds += 1;
            if self.duds == 10 {
                let front = self
                    .ctx
                    .shared
                    .args
                    .get(current_pos)
                    .expect("cursor past end of args during infinite loop detection");
                unreachable!(
                    "We made a lot of iterations trying to parse {front:?} but made no progress. \
                              This shouldn't be reachable, please report it as a bug."
                );
            }
        } else {
            self.duds = 0;
        }
        self.prev_pos = current_pos;
    }
}

/// Executor connected to a context
///
/// Created with [`Executor::new`]
struct Executor<'p> {
    ctx: Ctx<'p>,
    scope_start: Id,
    to_wake: Vec<Id>,
    to_propagate: VecDeque<TaskInfo>,
    mixer: Mixer,
    /// Is this a full parser with help and everything?
    ///
    /// Set to `true` for command parsers and similar,
    /// Set to `false` for nested parsers.
    ///
    /// global parsers run only in full parsers
    full_parser: bool,
}

impl<'p> Executor<'p> {
    fn new(ctx: Ctx<'p>, scope_start: Id, full_parser: bool) -> Self {
        Self {
            ctx,
            scope_start,
            to_wake: Vec::new(),
            to_propagate: VecDeque::new(),
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
    fn execute_group(&mut self, input: &'p OsStr) -> Result<(), Error> {
        let res = self.execute_group_inner(input);
        if res.is_ok() {
            self.ctx.cursor().update(|c| c + 1);
        } else {
            self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
        }
        res
    }

    fn execute_group_inner(&mut self, mut input: &'p OsStr) -> Result<(), Error> {
        let orig = input;
        let mut ix = 0;

        while let Some((arg, rest)) = self.populate_group(input)
            && !self.mixer.is_empty()
        {
            ix += 1;
            *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Arg(arg);
            let ids = self.mixer.for_wake(&self.ctx.shared.tasks.borrow());
            for id in ids {
                let info = self.ctx.poll_incomplete(id);
                if info.consumed != 1 {
                    // but this is a problem, should generate an error message that we can't parse
                    // current argument...
                    todo!("should have consumed a single item, it did {info:?}")
                }
                self.to_wake.push(info.id);
            }

            self.stage_2(1);
            self.propagate();
            self.process_scheduled();
            if rest.is_empty() {
                return Ok(());
            } else {
                input = rest;
            }
        }
        let name = input
            .next_char()
            .expect("non-empty input after failed group parse")
            .0;
        let group = format!("-{}", orig.to_string_lossy());
        let ix = ix as u32 + 1;
        let problem = Problem::OnlyOnceInGroup { group, name, ix };
        Err(Error::Problem(self.ctx.cursor().get(), problem))
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

    fn is_group_like(&mut self, front: &'p OsStr) -> Option<&'p OsStr> {
        if !matches!(
            lex_os_arg(front),
            Arg::Named {
                name: Name::Short(_),
                value: Some((Adjacency::Immediate, _)),
            }
        ) {
            return None;
        };
        let input = front.next_char()?.1;
        let mut current = input;

        debug_assert!(self.mixer.is_empty());
        loop {
            let rest = self.populate_group(current)?.1;
            self.mixer.clear();
            if rest.is_empty() {
                return Some(input);
            } else {
                current = rest;
            }
        }
    }
    fn populate_group(&mut self, input: &'p OsStr) -> Option<(Arg<'p>, &'p OsStr)> {
        let (ch, rest) = input.next_char()?;
        let name = Name::Short(ch);

        if self.full_parser {
            self.mixer
                .populate_short_arg(ch, &self.ctx.shared.global_triggers);
            if !self.mixer.is_empty() {
                // global argument, e.g. -n value
                let value = Some((Adjacency::Immediate, rest));
                return Some((Arg::Named { name, value }, OsStr::new("")));
            }

            self.mixer
                .populate_short_flag(ch, &self.ctx.shared.global_triggers);
            if !self.mixer.is_empty() {
                // global flag, e.g. -g where rest belongs to the next stacked flag
                return Some((Arg::Named { name, value: None }, rest));
            }
        }

        self.mixer.populate_short_arg(ch, &self.ctx.triggers);
        if !self.mixer.is_empty() {
            // local argument
            let value = Some((Adjacency::Immediate, rest));
            return Some((Arg::Named { name, value }, OsStr::new("")));
        }

        self.mixer.populate_short_flag(ch, &self.ctx.triggers);
        if !self.mixer.is_empty() {
            // local flag
            return Some((Arg::Named { name, value: None }, rest));
        }

        None
    }

    fn execute(&mut self) -> Result<(), Error> {
        let mut progress = ProgressTracker::new(&self.ctx);

        // We'll stop once there's no more data or no more candidates to run
        loop {
            if self.ctx.shared.stop.take() {
                break;
            }

            progress.tick();

            assert!(self.to_propagate.is_empty());
            self.process_scheduled();

            let Some(front) = self.ctx.shared.args.get(self.ctx.cursor().get()) else {
                break;
            };

            if let Some(out) = self.check_autocomplete(front) {
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
            let best_size = self.stage_1(front);

            if self.to_wake.is_empty() {
                if !self.ctx.strict_pos.get()
                    && let Some(front) = self.is_group_like(front)
                {
                    self.execute_group(front)?;
                    self.propagate();
                    continue;
                } else if let Some(scope) = { self.ctx.early_exit.borrow().last().copied() }
                    && self.kill_in_scope(scope, KillReason::NoMatchingInput)
                {
                    continue;
                } else {
                    self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
                    let pos = self.ctx.cursor().get();
                    return Err(Error::Problem(pos, self.complain_about(front)));
                }
            }

            self.stage_2(best_size);
            self.propagate();

            self.ctx.cursor().update(|c| c + best_size);
        }
        self.cleanup()
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        // terminate all the currently active tasks
        self.kill_in_scope(self.current_scope(), KillReason::NoMatchingInput);
        self.process_scheduled();

        assert!(self.ctx.shared.pending_ops.borrow().is_empty());
        self.ctx
            .shared
            .tasks
            .borrow_mut()
            .assert_no_tasks_past_end(self.scope_start.0);

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

        // Clear conflicts created within this executor's scope

        let scope = self.current_scope();
        self.ctx
            .shared
            .conflicts
            .borrow_mut()
            .retain(|c| !scope.contains(c.id()));

        // reset next_free to reclaim the ID range for this executor
        self.ctx.shared.next_free.set(self.scope_start.0);
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

        let arg = if self.ctx.strict_pos.get() {
            Arg::Pos { value: unexpected }
        } else {
            lex_os_arg(unexpected)
        };

        match arg {
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

                self.ctx.visited.vi(&mut visitors);
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
                    self.ctx.visited.vi(&mut is_command);
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
    fn stage_1(&mut self, front: &'p OsStr) -> u32 {
        let arg = if self.ctx.strict_pos.get() {
            Arg::Pos { value: front }
        } else {
            lex_os_arg(front)
        };

        if self.full_parser {
            self.mixer.populate(front, &arg, &self.ctx, true);
        }

        if self.mixer.is_empty() {
            self.mixer.populate(front, &arg, &self.ctx, false);
        }

        *self.ctx.shared.wakeup_reason.borrow_mut() = Reason::Arg(arg);

        let mut best_size = 0;
        let to_wake = self.mixer.for_wake(&self.ctx.shared.tasks.borrow());
        for id in to_wake {
            let info = self.ctx.poll_incomplete(id);
            self.to_wake.push(id);
            best_size = best_size.max(info.consumed);
        }
        *self.ctx.current_value.borrow_mut() = None;
        best_size
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
            self.ctx.poll_incomplete(sid);
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
    pub(crate) fn fork(&self, level: Option<&str>, visited: &'p dyn Visited) -> Ctx<'p> {
        let mut path = self.path.clone();
        if let Some(name) = level {
            path.push(' ');
            path.push_str(name);
        }
        Self::make(path, self.shared.clone(), self.strict_pos.get(), visited)
    }

    pub(crate) fn new(
        args: &'p Args,
        help: &'p BoxParser<Help>,
        visited: &'p dyn Visited,
    ) -> Ctx<'p> {
        let shared = Rc::new(SharedCtx {
            args,
            help,
            tasks: RefCell::new(Tasks::new()),
            next_free: Cell::new(1),
            wakeup_reason: RefCell::new(Reason::Pass),
            cursor: Cell::new(0),
            global_triggers: Rc::new(RefCell::new(Triggers::default())),
            current_task: RefCell::new(TaskInfo::default()),
            sums: RefCell::new(BTreeMap::new()),
            pending_ops: RefCell::new(VecDeque::new()),
            conflicts: RefCell::new(Vec::new()),
            parsers: RefCell::new(Vec::new()),
            vault: RefCell::new(vault::Storage::default()),
            stop: Cell::new(false),
        });
        Self::make(args.app.clone(), shared, false, visited)
    }

    fn make<'o>(
        path: String,
        shared: Rc<SharedCtx<'o>>,
        strict_pos: bool,
        visited: &'o dyn Visited,
    ) -> Ctx<'o> {
        Rc::new(RawCtx {
            shared,

            early_exit: Default::default(),

            strict_pos: Cell::new(strict_pos),
            triggers: Default::default(),
            path,
            current_value: Default::default(),
            visited,
        })
    }

    #[inline(never)]
    fn execute(
        self: &Ctx<'p>,
        lazy: bool,
        info: Option<&Info>,
        scope_start: u32,
    ) -> Result<(), Error> {
        let mut hh = None;
        if info.is_some() {
            let (h, act) = self.make_raw_task(self.shared.help);
            hh = Some(h);
            let task = Task {
                act,
                info: self.make_child_info(Kind::Prod),
            };

            self.add_task(task);
        }
        self.shared.parsers.borrow_mut().push(self.visited);

        // Save wakeup_reason so that nested executor runs don't leak
        // their modifications back to the parent.
        let saved_reason =
            std::mem::replace(&mut *self.shared.wakeup_reason.borrow_mut(), Reason::Pass);
        let mut r = Executor::new(self.clone(), Id(scope_start), info.is_some()).execute();
        *self.shared.wakeup_reason.borrow_mut() = saved_reason;

        self.shared.parsers.borrow_mut().pop();

        if let Some(h) = hh {
            r = match h.take() {
                Ok(help) => Err(Error::Final(self.render_help(help))),
                Err(Error::Silent(_) | Error::Problem(_, _) | Error::Missing(_)) => r,
                Err(e @ (Error::Final(_) | Error::CompReply(_) | Error::CompValue(_))) => Err(e),
            }
        };

        if lazy && matches!(r, Err(Error::Problem(_, Problem::Unconsumed { .. }))) {
            r = Ok(());
        }
        r
    }
}

impl Mixer {
    fn populate_short_arg(&mut self, name: char, triggers: &RefCell<Triggers>) {
        self.pecking_push(triggers.borrow().args.get(&Name::Short(name)));
    }

    fn populate_short_flag(&mut self, name: char, triggers: &RefCell<Triggers>) {
        self.pecking_push(triggers.borrow().flags.get(&Name::Short(name)));
    }

    fn populate(&mut self, front: &OsStr, arg: &Arg, ctx: &Ctx, global: bool) {
        let strict_pos = ctx.strict_pos.get();
        let triggers = if global {
            &ctx.shared.global_triggers.borrow()
        } else {
            &ctx.triggers.borrow()
        };

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
            Arg::Named { name, value } => {
                self.pecking_push(triggers.args.get(name));
                let flags = triggers.flags.get(name);
                if let Some((adj, value)) = value
                    && let Some(po) = flags
                    && let Some(id) = po.first()
                    && let Name::Short(n) = name
                {
                    let problem = Problem::ExpectedFlag {
                        name: Name::Short(*n),
                        adj: *adj,
                        value: value.to_string_lossy().to_string(),
                    };
                    let msg = problem.to_string();

                    let conflict = Conflict::Caught {
                        pos: ctx.cursor().get(),
                        msg,
                        id,
                        global,
                    };
                    ctx.shared.conflicts.borrow_mut().push(conflict);
                } else {
                    self.pecking_push(flags);
                }
            }
            Arg::Pos { value } => {
                if let Some(name) = arg::as_name(value)
                    && !strict_pos
                {
                    self.pecking_push(triggers.literal.get(&name));
                    // Single-char bare word could be a command registered as
                    // Name::Long (e.g. command("a")), so also check that variant
                    if let Lit(Name::Short(_)) = &name
                        && let Some(s) = value.to_str()
                    {
                        let long = Lit(Name::Long(Cow::Borrowed(s)));
                        self.pecking_push(triggers.literal.get(&long));
                    }
                }
                self.pecking_push(Some(&triggers.pos));
            }
        }
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

#[cfg(test)]
mod tests {
    mod adjacent;
    mod algebra;
    mod any;
    mod anywhere;
    mod argument;
    mod command;
    mod complete;
    mod construct;
    mod custom_help;
    mod errors;
    mod fallback;
    mod flag;
    mod git;
    mod global;
    mod help;
    mod helpers;
    mod leftovers;
    mod nested;
    mod offset;
    mod osstring;
    mod parse;
    mod positionals;
    mod pure_with;
    mod repeat;
    mod sum;
    mod unsorted;
    mod usage;
    mod vault;
}
