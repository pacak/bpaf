use crate::{
    ctx::Ctx,
    error::{Error, Message},
    named::Name,
    pecking::Pecking,
    split::{split_param, Arg, OsOrStr},
    visitor::ExplainUnparsed,
    Parser,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    future::Future,
    pin::{pin, Pin},
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

// Redesign
// make executor more aware of tasks
//
//
// problem - when parent gets dropped - we need to terminate all the children since
// flag and other non future triggers don't register this signal
// better names for join/spark handles?
//
// TODO:
// - factor out shared bits of argument parser so that getting an argument value from Ctx is not
//   duplicated across all the possible T
// - switch from atomics to Cell<u32> -
// - centralize task context management
// - if we did nothing at all for the whole loop - panic
// - proper support for --
// - don't run "explain" for --help or --version parsers?
// - subparser visitor should tell the difference between command and plain subparser?
// - simplify branchid - can be simple a pair of two ids - actually just one id for branch
// - in help have proper usage line
// - support for prefix parsing only - chained subcommands, etc
// - support for subparsers
// - in --version have proper version from either top level parser or current level parser
// - port missing items
// - port tests
// - Message or Error can have 'ctx lifetime?
//
// - shell completion
// - rename Cx into something meaningful
// - colorschemes
// - documentation
// - markdown/manpage

pub(crate) mod futures;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id {
    branch: u32,
    id: u32,
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}:{}", self.branch, self.id)
    }
}

impl Id {
    pub(crate) fn new(branch: u32, id: u32) -> Self {
        Id { branch, id }
    }
}

impl Id {
    const ZERO: Self = Id { branch: 0, id: 0 };

    pub(crate) fn next_branch(&self) -> Self {
        let mut id = *self;
        id.branch += 1;
        id
    }
    pub(crate) fn succ(&self) -> Self {
        Self {
            branch: self.branch,
            id: self.id + 1,
        }
    }
}

pub(crate) enum Action<'a> {
    Raw { action: RawAction<'a>, waker: Waker },
    Trigger(Trigger<'a>),
}

pub(crate) enum Trigger<'a> {
    Flag {
        names: &'a [Name<'static>],
        action: Box<dyn Fn(Option<(Name, Option<OsOrStr>)>) + 'a>,
    },
    Arg {
        names: &'a [Name<'static>],
        action: Box<dyn Fn(Option<(Name, Option<OsOrStr>)>) + 'a>,
    },
    Positional {
        action: Box<dyn Fn(Option<OsOrStr>) + 'a>,
    },
    Literal {
        names: &'a [Name<'static>],
        action: Box<dyn Fn(bool) + 'a>,
    },
    Any {
        action: Box<dyn Fn(Option<&OsOrStr>) -> bool + 'a>,
    },
}

pub type RawAction<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;
pub(crate) struct Task<'a> {
    pub(crate) action: Action<'a>,

    pub(crate) parent: Id,

    pub(crate) consumed: u32,

    pub(crate) done: bool,
}

impl<'a> Task<'a> {
    /// Run a task in a context, return number of items consumed an a result
    ///
    /// does not advance the pointer
    fn poll(
        &mut self,
        id: Id,
        front_named_arg: Option<OsOrStr<'a>>,
        ctx: &Ctx,
    ) -> (Poll<()>, usize) {
        let before = ctx.cur();
        ctx.items_consumed.set(self.consumed);
        *ctx.current_task.borrow_mut() = Some(id);
        let poll = match &mut self.action {
            Action::Raw { action: act, waker } => {
                println!("Waking raw action {id:?}");
                let mut cx = Context::from_waker(waker);
                act.as_mut().poll(&mut cx)
            }
            Action::Trigger(trigger) => Poll::Ready(match trigger {
                Trigger::Literal { names: _, action } => action(!ctx.is_term()),
                _ => todo!("shouldn't poll triggers here"),
                // Trigger::Flag { names: _, action } => todo!(), // action(!ctx.is_term()),
                // Trigger::Arg { names: _, action } => todo!(),  // action(front_named_arg),
                // Trigger::Positional { action } => todo!(),     // action(!ctx.is_term()),
                // Trigger::Any { action } => todo!(), // match action(!ctx.is_term()) {
                //                                     //     Some(r) => todo!(),
                //                                     //     None => return (Poll::Pending, 0),
                //                                     // },
            }),
        };
        *ctx.current_task.borrow_mut() = None;
        let after = ctx.cur();
        ctx.set_cur(before);
        (poll, after - before)
    }
}

// TODO - make private and remove from __private reexport!
#[derive(Debug, Copy, Clone)]
pub enum IdStrat {
    /// Use next available Id, keep this branch
    ///
    /// Regular spawn, most of tasks should be using this
    KeepBranch,
    /// Use next available Id, make it a new branch
    ///
    /// Any sum type should be using this
    NewBranch,
    /// This task and all of the children use Id starting from this one
    ///
    /// Used by variants of .many combinator
    KeepId,
}

pub(crate) enum Op<'a> {
    /// Spawn a "raw" task
    ///
    /// It might contain children or produce a result right away, but it cannot parse anything by
    /// itself. They can use custom id strategy
    SpawnTask {
        parent: Id,
        strat: IdStrat,
        action: RawAction<'a>,
    },

    /// Spawn a "trigger" task
    ///
    /// Executor will wake up a trigger task when their condition is met, they can't have children
    /// in a context of this executor so consumption must be atomic. They can spawn executors of
    /// their own - command parser uses a literal trigger to do exactly that
    SpawnTrigger {
        parent: Id,
        action: Trigger<'a>,
    },
    /// Wake up task Id and advance it
    WakeTask {
        id: Id,
        ///
        ///
        consumed: u32,
    },
    /// Parent is no longer interested
    RemoveTask {
        id: Id,
    },
    AddFallback {
        id: Id,
    },
    RemoveFallback {
        id: Id,
    },
    RestoreIdCounter {
        id: u32,
    },
}

pub type Fragment<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

struct WakeTask {
    id: Id,
    pending: Arc<Mutex<Vec<Id>>>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        self.pending.lock().expect("poison").push(self.id);
    }
}

impl<'ctx> Runner<'ctx> {
    pub(crate) fn new(ctx: Ctx<'ctx>) -> Self {
        Self {
            tasks: BTreeMap::new(),
            pending: Default::default(),
            next_task_id: 0,
            // flags: Default::default(),
            // args: Default::default(),
            // fallback: Default::default(),
            // positional: Default::default(),
            // literal: Default::default(),
            // any: Default::default(),
            ctx,
        }
    }
}

pub(crate) struct Runner<'ctx> {
    next_task_id: u32,
    ctx: Ctx<'ctx>,
    tasks: BTreeMap<Id, Task<'ctx>>,

    // pub(crate) flags: HashMap<Name<'ctx>, Pecking>,
    // pub(crate) args: HashMap<Name<'ctx>, Pecking>,
    // fallback: Pecking,
    // positional: Pecking,
    //
    // /// Parsers for "anything" they behave similar to positional parsers, but
    // /// unlike regular positional items they can opt not to consume a value
    // any: Pecking,
    //
    // literal: HashMap<Name<'ctx>, Pecking>,
    /// Shared with Wakers,
    ///
    /// contains a vector [`Id`] for tasks to wake up.
    pending: Arc<Mutex<Vec<Id>>>,
}

fn remove_names_from_pecking<'a>(
    id: Id,
    names: &[Name<'a>],
    pecking: &mut HashMap<Name<'a>, Pecking>,
) {
    for name in names.iter() {
        let std::collections::hash_map::Entry::Occupied(mut entry) = pecking.entry(name.clone())
        else {
            continue;
        };
        entry.get_mut().remove(id);
        if entry.get().is_empty() {
            entry.remove();
        }
    }
}

impl<'a> Runner<'a> {
    fn process_wakers(&mut self) {
        self.ctx.shared.borrow_mut().extend(
            self.pending
                .lock()
                .expect("poison")
                .drain(..)
                .map(|id| Op::WakeTask { id, consumed: 0 }),
        )
    }
    /// Handle scheduled operations
    ///
    /// This should advance all the tasks as far as possible without consuming the input
    fn propagate(&mut self, reactor: &mut Reactor) {
        loop {
            self.process_wakers();
            let mut shared = self.ctx.shared.borrow_mut();
            // get the next operation, or populate them from tasks to wake
            let Some(item) = shared.pop_front() else {
                return;
            };
            let before = shared.len();
            // tasks are going to borrow from shared when running
            drop(shared);

            match item {
                Op::SpawnTrigger { parent, action } => {
                    let id = self.next_id(parent.branch);
                    reactor.register(id, &action);
                    let task = Task {
                        action: Action::Trigger(action),
                        parent,
                        consumed: 0,
                        done: false,
                    };
                    assert!(self.tasks.insert(id, task).is_none());
                }
                Op::SpawnTask {
                    parent,
                    action,
                    strat,
                } => {
                    let mut restore_id = None;
                    let id = match strat {
                        IdStrat::KeepBranch => self.next_id(parent.branch),
                        IdStrat::NewBranch => {
                            let mut id = self.next_id(parent.branch);
                            id.branch = id.id;
                            id
                        }
                        IdStrat::KeepId => {
                            // We use KeepId strategy to be able to relaunch (inside of .many())
                            // parsers with the same id - since priority system uses id
                            // this means parsing priority isn't changing between different
                            // invocations.
                            //
                            // Tricky part is that we don't want to restore the original id
                            // on the first invocation of the KeepId strategy - so there's
                            // no overlap in ids in the children of the KeepId parser and
                            // whatever parser goes next
                            if parent.id + 1 < self.next_task_id {
                                restore_id = Some(self.next_task_id);
                                self.next_task_id = parent.id + 1;
                            }
                            self.next_id(parent.branch)
                        }
                    };
                    let waker = self.waker_for(id);
                    let mut task = Task {
                        action: Action::Raw { action, waker },
                        parent,
                        consumed: 0,
                        done: false,
                    };
                    self.ctx.items_consumed.set(0);

                    // Poll a task once. It might exit immediately if task implements
                    // something like `pure` or `fail`. This is fine, we still want to keep
                    // them around - I'm planning to move ownership of the result from JoinHandle
                    // and into ExitHandle so we can effectively "poison" shoter tasks by dropping
                    // them.
                    task.done = task.poll(id, None, &self.ctx).0.is_ready();
                    assert!(self.tasks.insert(id, task).is_none());
                    println!("Spawned task {id:?} with parent {parent:?}");

                    // To execute parsers in depth first order we must
                    // execute children of the tasks as well as anything they
                    // need (adding listeners) before the siblings. Easiest way
                    // to do that is to rotate the queue by exact amount added - forward
                    let mut shared = self.ctx.shared.borrow_mut();
                    let mut after = shared.len();

                    // for KeepId we might need to restore the original counter
                    if let Some(id) = restore_id {
                        shared.push_back(Op::RestoreIdCounter { id });
                        after += 1;
                    }
                    // before:
                    // T S1 S2 S3
                    // task T is consumed and it spawns some children: C1/C2
                    // S1 S2 S3 C1 C2
                    // rotate right by 2:
                    // C1 C2 S1 S2 S3
                    shared.rotate_right(after - before);
                }
                Op::RemoveTask { id } => {
                    if id == Id::new(0, 1) {
                        println!("Refusing to delete the root task");
                        return;
                    }
                    println!("Removing task {id:?} and it's children");
                    let mut consumed = 0;
                    if let Some(task) = self.tasks.remove(&id) {
                        while let Some((child_id, child)) = self
                            .first_deepest_child(id)
                            .and_then(|child_id| Some((child_id, self.tasks.remove(&child_id)?)))
                        {
                            if let Action::Trigger(ref action) = child.action {
                                reactor.unregister(child_id, action);
                            } else {
                                consumed += child.consumed;
                            }
                        }

                        if let Action::Trigger(ref action) = task.action {
                            reactor.unregister(id, action);
                        } else {
                            consumed += task.consumed;
                        }

                        if let Some(parent) = self.tasks.get_mut(&task.parent) {
                            parent.consumed += consumed
                        }
                    }
                }
                Op::WakeTask { id, consumed } => {
                    println!("Waking {id:?}");
                    let Some(task) = self.tasks.get_mut(&id) else {
                        println!("waking up removed task {id:?}");
                        continue;
                    };
                    if task.done {
                        println!("waking up done task {id:?}");
                        continue;
                    }
                    task.consumed += consumed;
                    self.ctx.items_consumed.set(task.consumed);
                    let (poll, _consumed) = task.poll(id, None, &self.ctx);
                    self.handle_task_poll(id, reactor, poll);
                }
                Op::RestoreIdCounter { id } => {
                    self.next_task_id = id + 1;
                }
                Op::AddFallback { id } => {
                    println!("Adding exit fallback to {id:?}");
                    reactor.fallback.insert(id);
                }
                Op::RemoveFallback { id } => {
                    println!("Removing exit fallback from {id:?}");
                    reactor.fallback.remove(id);
                }
            }
        }
    }

    // /// Pick one or more parsers that can handle front argument
    // fn pick_parsers(&self, front: &mut Arg, ids: &mut Vec<Id>) {
    //     debug_assert!(ids.is_empty());
    //     match front {
    //         Arg::Named { name, value: _ } => {
    //             if let Some(p) = self.args.get(name) {
    //                 ids.extend(p.heads());
    //             }
    //             if let Some(p) = self.flags.get(name) {
    //                 ids.extend(p.heads());
    //             }
    //         }
    //         Arg::ShortSet { current, names } => {
    //             if *current == 0 {
    //                 println!("pick any here");
    //                 if ids.is_empty() {
    //                     *current += 1;
    //                 } else {
    //                     return;
    //                 }
    //             }
    //             println!("Picking for things in ShortSet {current:?} {names:?} ");
    //             let name = Name::Short(names[*current - 1]);
    //             if let Some(p) = self.flags.get(&name) {
    //                 ids.extend(p.heads());
    //             }
    //         }
    //         Arg::Positional { value } => {
    //             if let OsOrStr::Str(name) = &value {
    //                 let mut cs = name.chars();
    //                 if let (Some(c), None) = (cs.next(), cs.next()) {
    //                     let name = Name::Short(c);
    //                     if let Some(p) = self.literal.get(&name) {
    //                         ids.extend(p.heads());
    //                     }
    //                 } else if let Some(p) = self.literal.get(&Name::Long(Cow::Borrowed(name))) {
    //                     ids.extend(p.heads());
    //                 }
    //             }
    //             ids.extend(self.positional.heads());
    //         }
    //     }
    //     ids.extend(self.any.iter());
    //
    //     // TODO - include Any here
    //     if ids.is_empty() {
    //         if let Some(id) = self.fallback.heads().next() {
    //             let f = self.first_child(id);
    //             // Not the right branch, but the right one is not needed - it is only used
    //             // for deduplication. Should `ids` be `Vec<(Option<BranchId>, Id)>` instead?
    //             // let branch = self.tasks.get(&id).unwrap().branch;
    //             ids.push(f);
    //             self.ctx.set_term(true);
    //         }
    //     } else {
    //         ids.sort(); // sort by branch
    //     }
    // }

    // The parser has failed to consume everything, we need
    // to prepare an explanation for the user. This method
    // prepares the explainer.
    #[inline(never)]
    fn mk_explainer(
        &self,
        unparsed_raw: &'a OsOrStr,
        failure: Option<Error>,
    ) -> Result<ExplainUnparsed<'a>, Error> {
        let missing = match failure {
            None => None,
            Some(Error {
                message: Message::Missing(vec),
                offset: _,
            }) => Some(vec),
            Some(err) => return Err(err),
        };
        let parsed_start = self.ctx.ctx_start.get() as usize;
        let parsed_end = self.ctx.cur();
        let parsed = &self.ctx.args[parsed_start..parsed_end];
        let m = HashMap::new();
        // TODO - don't do this here!
        // Instead either try to guess the right value inside of the
        // explainer or pass the actual unparsed value from the parser
        let unparsed = split_param(unparsed_raw, &m, &m).unwrap();
        let unparsed_raw = unparsed_raw.str();
        Ok(ExplainUnparsed::new(
            missing,
            unparsed,
            unparsed_raw,
            parsed,
        ))
    }

    pub(crate) fn run_parser<T, P>(mut self, parser: &'a P, compact: bool) -> Result<T, Error>
    where
        P: Parser<T> + ?Sized,
        T: 'static,
    {
        let root_id = self.next_id(0);
        let root_waker = self.waker_for(root_id);
        let mut handle = pin!(self.ctx.spawn(root_id, IdStrat::KeepBranch, parser));
        let mut root_cx = Context::from_waker(&root_waker);
        let _ = handle.as_mut().poll(&mut root_cx);

        self.inner()?;

        println!("done with inner, checking the result");
        println!("{:?}", self.tasks.keys().collect::<Vec<_>>());
        let Poll::Ready(result) = handle.as_mut().poll(&mut root_cx) else {
            unreachable!("bpaf internal error: Failed to produce result");
        };
        if compact {
            if let Some(unparsed_raw) = self.ctx.args.get(self.ctx.cur()) {
                let mut explainer = self.mk_explainer(unparsed_raw, result.err())?;
                parser.visit(&mut explainer);
                let message = explainer.explain();
                return Err(Error {
                    message,
                    offset: self.ctx.cur(),
                });
            }
        }

        result
    }

    fn inner(&mut self) -> Result<(), Error> {
        let mut reactor = Reactor::default();
        self.propagate(&mut reactor);
        let mut strict_pos = false;
        while let Some(front) = self.ctx.args.get(self.ctx.cur()) {
            self.propagate(&mut reactor);

            // TODO - move strict_pos into runner or ctx?? It might
            // be useful to produce a better error message.
            if front == "--" && !strict_pos {
                strict_pos = true;
                self.ctx.advance(1);
                continue;
            }

            let param =
                split_param(front, &reactor.args, &reactor.flags).map_err(|message| Error {
                    message,
                    offset: self.ctx.cur(),
                })?;
            println!("Need to consume {param:?}");
            let consumed = match &param {
                Arg::ShortSet { current, names } => {
                    let first_name = Name::Short(names[0]);
                    let items = reactor.named(first_name);
                    let r = self.run_items(front, &param, items);
                    match r {
                        Some(0) => {
                            for name in names.iter().skip(1) {
                                let name = Name::Short(*name);
                                let items = reactor.shorts(name);
                                let r = self.run_items(front, &param, items);
                                if r != Some(0) {
                                    todo!();
                                }
                            }
                        }
                        Some(xx) => {
                            self.ctx.advance(xx);
                            todo!("Advance {xx:?}");
                        }
                        None => todo!(),
                    }

                    self.ctx.advance(1);
                    Some(1)
                }
                Arg::Named { name, value: _ } => {
                    let items = reactor.named(name.as_ref());
                    self.run_items(front, &param, items)
                }
                Arg::Positional { value: _ } => {
                    let items = reactor.positionals();
                    self.run_items(front, &param, items)
                }
            };

            println!("Consumption of {front:?} {param:?}, got {consumed:?}");
            if consumed.is_none() {
                if let Some(id) = reactor
                    .fallback
                    .inner()
                    .first()
                    .copied()
                    .and_then(|parent| self.first_deepest_child(parent))
                {
                    self.ctx
                        .shared
                        .borrow_mut()
                        .push_back(Op::RemoveTask { id });
                } else {
                    break;
                }
            }
        }
        self.propagate(&mut reactor);
        println!("Terminating already?");
        for (id, task) in self.tasks.iter_mut() {
            if let Action::Trigger(action) = &task.action {
                match action {
                    Trigger::Flag { names: _, action } => action(None),
                    Trigger::Arg { names: _, action } => action(None),
                    Trigger::Positional { action } => action(None),
                    Trigger::Literal { names: _, action } => todo!(), // action(None),
                    Trigger::Any { action } => {
                        action(None);
                    }
                }
                reactor.unregister(*id, action);
            }
        }
        self.propagate(&mut reactor);
        Ok(())
    }

    fn run_items(
        &mut self,
        front: &OsOrStr,
        param: &Arg,
        mut items: impl BranchIterator,
    ) -> Option<usize> {
        let mut max = 0;
        // TODO - move `progressed` and `alive` into executor
        let mut progressed = Vec::new();
        let mut alive = BTreeSet::new();

        while let Some(id) = items.next() {
            println!("Waking up trigger {id:?} to consume {param:?}");

            let Some(len) = self.run_trigger_task(id, front, param) else {
                items.same_branch();
                continue;
            };
            max = max.max(len);
            progressed.push((id, len));
        }

        for (mut id, len) in progressed.iter().copied() {
            if len == max {
                alive.insert(id.branch);
                while let Some(task) = self.tasks.get(&id) {
                    alive.insert(task.parent.branch);
                    id = task.parent;
                }
            }
        }
        self.tasks.retain(|id, task| {
            if alive.contains(&id.branch) {
                true
            } else {
                self.ctx
                    .shared
                    .borrow_mut()
                    .push_back(Op::RemoveTask { id: *id });
                !matches!(task.action, Action::Trigger(_))
            }
        });

        progressed.clear();
        self.ctx.advance(max);
        if alive.is_empty() {
            None
        } else {
            Some(max)
        }
    }

    fn run_trigger_task(&mut self, id: Id, raw: &OsOrStr, arg: &Arg) -> Option<usize> {
        let task = self.tasks.get_mut(&id)?;
        let Action::Trigger(trigger) = &mut task.action else {
            return None;
        };
        let before = self.ctx.cur();
        match (trigger, arg) {
            (Trigger::Flag { names: _, action }, Arg::Named { name, value }) => {
                action(Some((name.as_ref(), value.as_ref().map(|v| v.as_ref()))));
                // TODO - as_deref()
            }
            (Trigger::Flag { names: _, action }, Arg::ShortSet { current, names }) => {
                action(Some((Name::Short(names[*current]), None)));
                // this rewinds the cursor making it so task consumed length will be 0
                // since there's usually more to consume
                self.ctx.set_cur(before);
            }
            (Trigger::Flag { names: _, action }, Arg::Positional { value }) => todo!(),
            (Trigger::Arg { names: _, action }, Arg::Named { name, value }) => {
                action(Some((name.as_ref(), value.as_ref().map(|v| v.as_ref()))));
            }
            (Trigger::Arg { names: _, action }, Arg::ShortSet { current, names }) => todo!(),
            (Trigger::Arg { names: _, action }, Arg::Positional { value }) => todo!(),
            (Trigger::Positional { action }, Arg::Named { name, value }) => todo!(),
            (Trigger::Positional { action }, Arg::ShortSet { current, names }) => todo!(),
            (Trigger::Positional { action }, Arg::Positional { value }) => {
                action(Some(value.as_ref()));
            }
            (Trigger::Literal { names: _, action }, Arg::Named { name, value }) => todo!(),
            (Trigger::Literal { names: _, action }, Arg::ShortSet { current, names }) => todo!(),
            (Trigger::Literal { names: _, action }, Arg::Positional { value }) => todo!(),
            (Trigger::Any { action }, _) => action(Some(raw)).then_some(())?,
        };
        let after = self.ctx.cur();
        self.ctx.set_cur(before);

        // task.consumed = (after - before) as u32;

        println!("consumed: {}", task.consumed);
        self.ctx.shared.borrow_mut().push_back(Op::WakeTask {
            id: task.parent,
            consumed: (after - before) as u32,
        });
        Some(after - before)
    }

    // /// Execute the parser
    // ///
    // /// `primary` is set to false for parsers like `--version` or `--help`, they
    // /// don't care about improving the error message.
    // ///
    // /// TODO - `primary` seem like an optimization. Do I need it?
    // pub(crate) fn run_parser<P, T>(mut self, parser: &'a P, primary: bool) -> Result<T, Error>
    // where
    //     P: Parser<T> + ?Sized,
    //     T: 'static,
    // {
    //     let root_id = self.next_id(1);
    //     let root_waker = self.waker_for(root_id);
    //
    //     // first - shove parser into a task so wakers can work
    //     // as usual. Since we care about the result - output type
    //     // must be T so it can't go into tasks directly.
    //     // We spawn it as a task instead and keep the handle
    //     let mut handle = pin!(self.ctx.spawn(root_id, IdStrat::KeepBranch, parser));
    //     let mut root_cx = Context::from_waker(&root_waker);
    //     debug_assert!(handle.as_mut().poll(&mut root_cx).is_pending());
    //
    //     let unparsed = self.inner_loop()?;
    //
    //     let Poll::Ready(result) = handle.as_mut().poll(&mut root_cx) else {
    //         unreachable!("bpaf internal error: Failed to produce result");
    //     };
    //
    //     let Some(unparsed) = unparsed else {
    //         // parsed everything - can't improve error message if it is an error
    //         return result;
    //     };
    //     if !primary {
    //         return result;
    //     }
    //
    //     // TODO
    //     // if prefix_only {
    //     //     return result;
    //     // }
    //
    //     let missing = match result {
    //         Ok(_) => None,
    //         Err(Error {
    //             message: Message::Missing(vec),
    //             offset: _,
    //         }) => Some(vec),
    //         e => return e,
    //     };
    //
    //     let parsed = &self.ctx.args[0..self.ctx.cur()];
    //     let unparsed_raw = self.ctx.args[self.ctx.cur()].str();
    //     let mut v = ExplainUnparsed::new(missing, unparsed, unparsed_raw, parsed);
    //     parser.visit(&mut v);
    //     let message = v.explain();
    //     Err(Error {
    //         message,
    //         offset: self.ctx.cur(),
    //     })
    // }
    //
    // /// Run configured parser for as long as possible,
    // fn inner_loop(&mut self) -> Result<Option<Arg<'a>>, Error> {
    //     // mostly to avoid allocations
    //     let mut ids = Vec::new();
    //     let mut out = Vec::new();
    //
    //     let mut prev_arg: Arg<'a> = Arg::DUMMY;
    //
    //     let mut strict_pos = false;
    //
    //     let mut prev_consumed = false;
    //     let no_parse = loop {
    //         self.ctx.set_term(false);
    //         self.propagate();
    //         println!("============= Propagate done");
    //
    //         debug_assert!(self.ctx.shared.borrow().is_empty());
    //         debug_assert!(self.pending.lock().expect("poison").is_empty());
    //
    //         let front = match &mut prev_arg {
    //             Arg::ShortSet { .. } if !prev_consumed => &mut prev_arg,
    //             Arg::ShortSet { current, names } if *current < names.len() => {
    //                 *current += 1;
    //                 &mut prev_arg
    //             }
    //             _ => {
    //                 if matches!(prev_arg, Arg::ShortSet { .. }) {
    //                     self.ctx.advance(1);
    //                 }
    //                 if let Some(val) = self.ctx.args.get(self.ctx.cur()) {
    //                     if val == "--" && !strict_pos {
    //                         strict_pos = true;
    //                         self.ctx.advance(1);
    //                         continue;
    //                     }
    //                     if strict_pos {
    //                         prev_arg = Arg::Positional {
    //                             value: val.as_ref(),
    //                         };
    //                     } else {
    //                         prev_arg =
    //                             split_param(val, &self.args, &self.flags).map_err(|message| {
    //                                 Error {
    //                                     message,
    //                                     offset: self.ctx.cur(),
    //                                 }
    //                             })?;
    //                     }
    //                     &mut prev_arg
    //                 } else {
    //                     println!("nothing to consume");
    //                     break false;
    //                 }
    //             }
    //         };
    //
    //         self.pick_parsers(front, &mut ids);
    //         if ids.is_empty() {
    //             println!("No parsers for {front:?}, exiting");
    //             break true;
    //         }
    //
    //         prev_consumed = self.consume(front, &mut ids, &mut out)?;
    //         println!("============= Consuming part done");
    //     };
    //
    //     if no_parse {
    //         println!("==== loop done, no valid parser to handle {prev_arg:?}, cleaning up");
    //     }
    //
    //     // at this point we are done consuming, either there's nothing more left or we don't know
    //     // how to consume the rest., let's run existing tasks to completion
    //     // first by waking them up all the consuming events (most of them fail with "not found")
    //     // and then by processing all the non-consuming events - this would either create some
    //     // errors or or those "not found" errors are going to be converted into something useful
    //
    //     self.propagate();
    //     self.ctx.set_term(true);
    //
    //     self.prepare_consumers_for_draining(&mut ids);
    //
    //     *self.ctx.front_value.borrow_mut() = None;
    //
    //     for id in ids.drain(..) {
    //         if let Some(task) = self.tasks.get_mut(&id) {
    //             println!("Need to terminate {id:?}");
    //             let (poll, consumed) = task.poll(id, None, &self.ctx);
    //             debug_assert_eq!(consumed, 0, "Consumed during termination?");
    //             debug_assert!(poll.is_ready(), "Termination left task stil pending?");
    //             self.handle_task_poll(id, poll);
    //         }
    //     }
    //     println!("Final propagation");
    //     self.propagate();
    //     Ok(no_parse.then_some(prev_arg))
    // }

    // fn first_child(&self, mut parent_id: Id) -> Id {
    //     for (child_id, task) in self.tasks.range(parent_id..).skip(1) {
    //         if task.parent == parent_id {
    //             parent_id = *child_id;
    //         } else {
    //             return parent_id;
    //         }
    //     }
    //     parent_id
    // }

    /// Find the deepest left most child
    ///
    /// Assuming we did non consuming prcessing prior to that - it will be
    /// a consuming parser
    fn first_deepest_child(&self, parent: Id) -> Option<Id> {
        let mut current = parent;
        for (child, task) in self.tasks.range(parent..).skip(1) {
            if task.parent == current {
                current = *child;
            } else {
                return (current != parent).then_some(current);
            }
        }
        None
    }

    // #[inline(never)]
    // /// Drain all consumers so they can be polled to emit default values
    // fn prepare_consumers_for_draining(&mut self, ids: &mut Vec<Id>) {
    //     ids.extend(self.flags.values().flat_map(|v| v.iter()));
    //     ids.extend(self.positional.iter());
    //     ids.extend(self.args.values().flat_map(|v| v.iter()));
    //     ids.extend(self.literal.values().flat_map(|v| v.iter()));
    //     // TODO: any
    //
    //     ids.sort();
    //     ids.dedup();
    // }

    #[inline(never)]
    /// Handle potential task completion
    /// - notify parent if it is interested in error codes
    /// - queue task termination if task is complete
    ///
    /// It cannot run the task as well since to be able to handle sum types
    /// we need to be able to decide which tasks to terminate based on consumed length
    fn handle_task_poll(&mut self, id: Id, reactor: &mut Reactor, poll: Poll<()>) {
        if poll.is_pending() {
            return;
        }
        println!("Handling exit for {id:?}");
        let task = self.tasks.get_mut(&id).expect("We know it's there");
        task.done = true;
        if matches!(task.action, Action::Raw { .. }) {
            // this was a raw task, if it is done - it won't be polling any children - we can
            // terminate them all here
            while let Some(child) = self.first_deepest_child(id) {
                // todo!("Terminated {child:?}");
                if let Some(ctask) = self.tasks.remove(&child) {
                    if let Action::Trigger(trigger) = &ctask.action {
                        reactor.unregister(child, trigger)
                    }
                }
            }
        }
        // TODO - Can we remove the task immediately?
        self.ctx
            .shared
            .borrow_mut()
            .push_back(Op::RemoveTask { id });
    }

    /// Allocate next task [`Id`]
    fn next_id(&mut self, branch: u32) -> Id {
        let id = self.next_task_id;
        self.next_task_id += 1;
        Id::new(branch, id)
    }
    /// Make a [`Waker`] for a task with this [`Id`].
    fn waker_for(&self, id: Id) -> Waker {
        let pending = self.pending.clone();
        Waker::from(Arc::new(WakeTask { id, pending }))
    }
}

// For every Sum, as soon as we start making any progress with any branch, no matter how deep - we
// must terminate all branches that don't make progress
//
// ways to imlement it:
// - user land and executor - user land subscribes to advances and reports upstream, deals with
//   child termination
//   + the same logic can be reused in many-unadjacent
//   - lots of back and forth across the boundary
//
// - executor only with "sets" - propagates and terminates tasks... mark'n'sweep?
//   every child that finishes marks child node of a sum parent up to the top most branch
//   then second pass goes though marked nodes,
//   removes marks and kills children without marks
//   + simple
//   - many-unadjacent is still an open question...
//
// - every sum task has some notion of cursor position, children making progress
//
//
// For every Prod positional items go sequentially - just put them in a vector
// Every sibling of an alt can consume a separate instance ... positional
// items go into a set of queues keyed by (Id, i32) and we can run one positional item from each
// queue :)

// several named items with the same name in a product
// go sequentially

// For 'Any' - same idea as PosPrio, they just get to
#[derive(Debug, Default)]
struct Reactor {
    pub(crate) flags: HashMap<Name<'static>, Pecking>,
    pub(crate) args: HashMap<Name<'static>, Pecking>,

    named: BTreeSet<(Name<'static>, Id)>,
    fallback: Pecking,
    positional: Pecking,

    /// Parsers for "anything" they behave similar to positional parsers, but
    /// unlike regular positional items they can opt not to consume a value
    any: BTreeSet<Id>,

    literal: HashMap<Name<'static>, Pecking>,
}

impl Reactor {
    fn register(&mut self, id: Id, action: &Trigger) {
        match action {
            Trigger::Flag { names, action: _ } => {
                for name in names.iter().cloned() {
                    self.flags.entry(name.clone()).or_default().insert(id);
                    self.named.insert((name, id));
                }
            }
            Trigger::Arg { names, action: _ } => {
                for name in names.iter().cloned() {
                    self.args.entry(name.clone()).or_default().insert(id);
                    self.named.insert((name, id));
                }
            }
            Trigger::Positional { action: _ } => {
                self.positional.insert(id);
            }
            Trigger::Literal { names, action: _ } => {
                for name in names.iter().cloned() {
                    self.literal.entry(name).or_default().insert(id);
                }
            }
            Trigger::Any { action: _ } => {
                self.any.insert(id);
            }
        }
    }

    fn unregister(&mut self, id: Id, action: &Trigger) {
        match action {
            Trigger::Flag { names, action: _ } => {
                for name in names.iter() {
                    self.named.remove(&(name.clone(), id));
                }
                remove_names_from_pecking(id, names, &mut self.flags);
            }
            Trigger::Arg { names, action: _ } => {
                for name in names.iter() {
                    self.named.remove(&(name.clone(), id));
                }
                remove_names_from_pecking(id, names, &mut self.args);
            }
            Trigger::Positional { action: _ } => {
                self.positional.remove(id);
            }
            Trigger::Literal { names, action: _ } => {
                remove_names_from_pecking(id, names, &mut self.literal);
            }
            Trigger::Any { action: _ } => {
                self.any.remove(&id);
            }
        }
    }

    fn positionals(&self) -> PosItems {
        PosItems {
            any: &self.any,
            pos: self.positional.inner(),
            branch: BranchCursor::new(),
        }
    }

    fn named<'a>(&'a self, name: Name<'a>) -> NamedItems<'a> {
        NamedItems {
            any: &self.any,
            named: &self.named,
            name,
            branch: BranchCursor::new(),
        }
    }

    fn any(&self) -> AnyItems<'_> {
        AnyItems {
            any: &self.any,
            branch: BranchCursor::new(),
        }
    }

    fn shorts<'a>(&'a self, name: Name<'a>) -> ShortItems<'a> {
        ShortItems {
            name,
            named: &self.named,
            branch: BranchCursor::new(),
        }
    }
}

struct BranchCursor {
    prev: Id,
    same_branch: bool,
}

impl BranchCursor {
    fn new() -> Self {
        Self {
            prev: Id::ZERO,
            same_branch: true,
        }
    }
    fn next(&mut self) -> Id {
        if self.same_branch {
            self.same_branch = false;
            self.prev.succ()
        } else {
            self.prev.next_branch()
        }
    }
    fn same_branch(&mut self) {
        self.same_branch = true;
    }
}

trait BranchIterator {
    fn next(&mut self) -> Option<Id>;
    fn same_branch(&mut self);
}

impl BranchIterator for AnyItems<'_> {
    fn next(&mut self) -> Option<Id> {
        let start = self.branch.next();
        self.any.range(start..).next().copied()
    }

    fn same_branch(&mut self) {
        self.branch.same_branch();
    }
}

struct ShortItems<'a> {
    name: Name<'a>,
    named: &'a BTreeSet<(Name<'a>, Id)>,
    branch: BranchCursor,
}

impl BranchIterator for ShortItems<'_> {
    fn next(&mut self) -> Option<Id> {
        let start = self.branch.next();
        self.named
            .range((self.name.as_ref(), start)..)
            .next()
            .and_then(|(name, id)| (name == &self.name).then_some(*id))
    }

    fn same_branch(&mut self) {
        self.branch.same_branch();
    }
}

struct AnyItems<'a> {
    any: &'a BTreeSet<Id>,
    branch: BranchCursor,
}

struct NamedItems<'a> {
    name: Name<'a>,
    any: &'a BTreeSet<Id>,
    named: &'a BTreeSet<(Name<'a>, Id)>,
    branch: BranchCursor,
}

impl BranchIterator for NamedItems<'_> {
    fn next(&mut self) -> Option<Id> {
        let start = self.branch.next();

        // multiple lookups seem expensive, but in practice in absense of any we
        // will be making a single lookup per branch
        //
        match (
            self.any.range(start..).next().copied(),
            self.named
                .range((self.name.as_ref(), start)..)
                .next()
                .and_then(|(name, id)| (name == &self.name).then_some(*id)),
        ) {
            (None, None) => None,
            (None, Some(id)) | (Some(id), None) => {
                self.branch.prev = id;
                Some(id)
            }
            (Some(a), Some(b)) => {
                let id = a.min(b);
                self.branch.prev = id;
                Some(id)
            }
        }
    }

    fn same_branch(&mut self) {
        self.branch.same_branch();
    }
}

struct PosItems<'a> {
    any: &'a BTreeSet<Id>,
    pos: &'a BTreeSet<Id>,
    branch: BranchCursor,
}

impl BranchIterator for PosItems<'_> {
    fn next(&mut self) -> Option<Id> {
        let start = self.branch.next();

        // multiple lookups seem expensive, but in practice in absense of any we
        // will be making a single lookup per branch
        //
        match (
            self.any.range(start..).next().copied(),
            self.pos.range(start..).next().copied(),
        ) {
            (None, None) => None,
            (None, Some(id)) | (Some(id), None) => {
                self.branch.prev = id;
                Some(id)
            }
            (Some(a), Some(b)) => {
                let id = a.min(b);
                self.branch.prev = id;
                Some(id)
            }
        }
    }
    fn same_branch(&mut self) {
        self.branch.same_branch();
    }
}
