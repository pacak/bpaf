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
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::{pin, Pin},
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

// redesign
// make executor more aware of tasks
//
//
// problem - when parent gets dropped - we need to terminate all the children since
// flag and other non future triggers don't register this signal
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

// enum Ac<'a> {
//     Fut(Box<dyn Future<Output = PoisonHandle> + 'a>),
//     Flag {
//         names: &'a [Name<'a>],
//         handle: futures::ExitHandle<'a, bool>,
//     },
//     Arg {
//         names: &'a [Name<'a>],
//         handle: futures::ExitHandle<'a, OsOrStr<'a>>,
//     },
//     Pos {
//         handle: futures::ExitHandle<'a, OsOrStr<'a>>,
//     },
//     Any {
//         foo: Box<dyn Pass>,
//     },
// }

//fn any(metavar: Metavar) -> Parser<T>

// trait Pass {
//     fn try_parse(&self, val: OsOrStr) -> Option<PoisonHandle>;
// }
//
// impl<P, T> Pass for (P, futures::ExitHandle<'_, T>)
// where
//     P: Fn(OsOrStr) -> Option<T>,
// {
//     fn try_parse(&self, val: OsOrStr<'_>) -> Option<PoisonHandle> {
//         (self.0)(val).map(|t| self.1.exit_task(Ok(t)))
//     }
// }

// TODO - newtype?
pub type PoisonHandle = Rc<Cell<bool>>;

pub(crate) enum Action<'a> {
    Raw { action: RawAction<'a>, waker: Waker },
    Trigger(Trigger<'a>),
}

pub(crate) enum Trigger<'a> {
    Flag {
        names: &'a [Name<'a>],
        action: Box<dyn Fn(bool) -> PoisonHandle + 'a>,
    },
    Arg {
        names: &'a [Name<'a>],
        action: Box<dyn Fn(Option<OsOrStr<'a>>) -> PoisonHandle + 'a>,
    },
    Positional {
        action: Box<dyn Fn(bool) -> PoisonHandle + 'a>,
    },
    Literal {
        names: &'a [Name<'a>],
        action: Box<dyn Fn(bool) -> PoisonHandle + 'a>,
    },
    Any {
        action: Box<dyn Fn(bool) -> Option<PoisonHandle> + 'a>,
    },
}

pub type RawAction<'a> = Pin<Box<dyn Future<Output = PoisonHandle> + 'a>>;
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
    ) -> (Poll<PoisonHandle>, usize) {
        let before = ctx.cur();
        ctx.items_consumed.set(self.consumed);
        *ctx.current_task.borrow_mut() = Some(id);
        let poll = match &mut self.action {
            Action::Raw { action: act, waker } => {
                let mut cx = Context::from_waker(waker);
                act.as_mut().poll(&mut cx)
            }
            Action::Trigger(trigger) => Poll::Ready(match trigger {
                Trigger::Flag { names: _, action } => action(!ctx.is_term()),
                Trigger::Arg { names: _, action } => action(front_named_arg),
                Trigger::Positional { action } => action(!ctx.is_term()),
                Trigger::Literal { names: _, action } => action(!ctx.is_term()),
                Trigger::Any { action } => match action(!ctx.is_term()) {
                    Some(r) => r,
                    None => return (Poll::Pending, 0),
                },
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
    WakeTask {
        id: Id,
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
            flags: Default::default(),
            args: Default::default(),
            fallback: Default::default(),
            positional: Default::default(),
            literal: Default::default(),
            any: Default::default(),
            ctx,
        }
    }
}

pub(crate) struct Runner<'ctx> {
    next_task_id: u32,
    ctx: Ctx<'ctx>,
    tasks: BTreeMap<Id, Task<'ctx>>,

    pub(crate) flags: HashMap<Name<'ctx>, Pecking>,
    pub(crate) args: HashMap<Name<'ctx>, Pecking>,
    fallback: Pecking,
    positional: Pecking,

    /// Parsers for "anything" they behave similar to positional parsers, but
    /// unlike regular positional items they can opt not to consume a value
    any: Pecking,

    literal: HashMap<Name<'ctx>, Pecking>,

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
    /// Handle scheduled operations
    ///
    /// This should advance all the tasks as far as possible without consuming the input
    fn propagate(&mut self) {
        let before = self.ctx.cur();
        loop {
            let mut shared = self.ctx.shared.borrow_mut();
            // get the next operation, or populate them from tasks to wake
            let Some(item) = shared.pop_front() else {
                shared.extend(
                    self.pending
                        .lock()
                        .expect("poison")
                        .drain(..)
                        .map(|id| Op::WakeTask { id }),
                );
                if shared.is_empty() {
                    assert_eq!(
                        before,
                        self.ctx.cur(),
                        "propagation should not consume items"
                    );
                    return;
                } else {
                    continue;
                }
            };
            let before = shared.len();
            // tasks are going to borrow from shared when running
            drop(shared);

            match item {
                Op::SpawnTrigger { parent, action } => {
                    let id = self.next_id(parent.branch);
                    match action {
                        Trigger::Flag { names, action: _ } => {
                            for name in names.iter() {
                                self.flags.entry(name.clone()).or_default().insert(id);
                            }
                        }
                        Trigger::Arg { names, action: _ } => {
                            for name in names.iter() {
                                self.args.entry(name.clone()).or_default().insert(id);
                            }
                        }
                        Trigger::Positional { action: _ } => {
                            println!("Adding positional parser {id:?}");
                            self.positional.insert(id);
                        }
                        Trigger::Literal { names, action: _ } => {
                            for name in names.iter() {
                                self.literal.entry(name.clone()).or_default().insert(id);
                            }
                        }
                        Trigger::Any { action: _ } => {
                            self.any.insert(id);
                        }
                    }
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
                    println!("{id:?}: Removing task");
                    if let Some(task) = self.tasks.remove(&id) {
                        if let Some(parent) = self.tasks.get_mut(&task.parent) {
                            parent.consumed += task.consumed;
                        }
                    }
                    use std::collections::btree_map::Entry;
                    let cid = id.succ();
                    let Entry::Occupied(child) = self.tasks.entry(cid) else {
                        continue;
                    };
                    let child_task = child.get();
                    if child_task.parent != id {
                        continue;
                    }
                    if let Action::Trigger(ref action) = child_task.action {
                        match action {
                            Trigger::Flag { names, action: _ } => {
                                remove_names_from_pecking(cid, names, &mut self.flags);
                            }
                            Trigger::Arg { names, action: _ } => {
                                remove_names_from_pecking(cid, names, &mut self.args);
                            }
                            Trigger::Positional { action: _ } => self.positional.remove(cid),
                            Trigger::Literal { names, action: _ } => {
                                remove_names_from_pecking(cid, names, &mut self.literal);
                            }
                            Trigger::Any { action: _ } => {
                                self.any.remove(cid);
                            }
                        }
                        child.remove();
                    }
                }
                Op::WakeTask { id } => {
                    let Some(task) = self.tasks.get_mut(&id) else {
                        println!("waking up removed task {id:?}");
                        continue;
                    };
                    if task.done {
                        println!("waking up done task {id:?}");
                        continue;
                    }
                    self.ctx.items_consumed.set(task.consumed);
                    let (poll, _consumed) = task.poll(id, None, &self.ctx);
                    self.handle_task_poll(id, poll);
                }
                Op::RestoreIdCounter { id } => {
                    self.next_task_id = id + 1;
                }
                Op::AddFallback { id } => {
                    println!("Adding exit fallback to {id:?}");
                    self.fallback.insert(id);
                }
                Op::RemoveFallback { id } => {
                    println!("Removing exit fallback from {id:?}");
                    self.fallback.remove(id);
                }
            }
        }
    }

    /// Pick one or more parsers that can handle front argument
    fn pick_parsers(&self, front: &mut Arg, ids: &mut Vec<Id>) {
        debug_assert!(ids.is_empty());
        match front {
            Arg::Named { name, value: _ } => {
                if let Some(p) = self.args.get(name) {
                    ids.extend(p.heads());
                }
                if let Some(p) = self.flags.get(name) {
                    ids.extend(p.heads());
                }
            }
            Arg::ShortSet { current, names } => {
                if *current == 0 {
                    println!("pick any here");
                    if ids.is_empty() {
                        *current += 1;
                    } else {
                        return;
                    }
                }
                println!("Picking for things in ShortSet {current:?} {names:?} ");
                let name = Name::Short(names[*current - 1]);
                if let Some(p) = self.flags.get(&name) {
                    ids.extend(p.heads());
                }
            }
            Arg::Positional { value } => {
                if let OsOrStr::Str(name) = &value {
                    let mut cs = name.chars();
                    if let (Some(c), None) = (cs.next(), cs.next()) {
                        let name = Name::Short(c);
                        if let Some(p) = self.literal.get(&name) {
                            ids.extend(p.heads());
                        }
                    } else if let Some(p) = self.literal.get(&Name::Long(Cow::Borrowed(name))) {
                        ids.extend(p.heads());
                    }
                }
                ids.extend(self.positional.heads());
            }
        }
        ids.extend(self.any.iter());

        // TODO - include Any here
        if ids.is_empty() {
            if let Some(id) = self.fallback.heads().next() {
                let f = self.first_child(id);
                // Not the right branch, but the right one is not needed - it is only used
                // for deduplication. Should `ids` be `Vec<(Option<BranchId>, Id)>` instead?
                // let branch = self.tasks.get(&id).unwrap().branch;
                ids.push(f);
                self.ctx.set_term(true);
            }
        } else {
            ids.sort(); // sort by branch
        }
    }

    fn consume(
        &mut self,
        front: &mut Arg<'a>,
        ids: &mut Vec<Id>,
        out: &mut Vec<(Id, Poll<PoisonHandle>, usize)>,
    ) -> Result<bool, Error> {
        // actual feed consumption happens here
        let mut max_consumed = 0;

        assert!(!ids.is_empty());
        if let Arg::Named { value, .. } = front {
            *self.ctx.front_value.borrow_mut() = value.clone()
        }

        let mut last_branch = 0;
        for id in ids.drain(..) {
            if id.branch == last_branch {
                println!("skipping {id:?}");
                continue;
            }
            // each scheduled task gets a chance to run,
            if let Some(task) = self.tasks.get_mut(&id) {
                let f = match front {
                    Arg::Named { value, .. } => value.clone(),
                    _ => None,
                };
                let (poll, consumed) = task.poll(id, f, &self.ctx);
                if poll.is_ready() {
                    last_branch = id.branch;
                } else {
                    continue;
                }
                task.consumed += consumed as u32;

                println!(
                    "{id:?} consumed {consumed}, is ready? {:?}!",
                    poll.is_ready()
                );
                out.push((id, poll, consumed));

                max_consumed = consumed.max(max_consumed);
            } else {
                todo!("got already terminated parser: {id:?}");
            }
        }

        for (id, poll, consumed) in out.drain(..) {
            if let Poll::Ready(eh) = &poll {
                if consumed < max_consumed {
                    // TODO  - custom error
                    eh.set(true);
                }
            }
            self.handle_task_poll(id, poll);
        }
        if let Arg::ShortSet { current, .. } = front {
            // This branch covers two cases - parsing a short set with "any" parser
            // and parsing a single flag out of the short set. With current set to 0
            // we are running "any" parsers only, if any of them succeed - no need to run any of
            // the flag parsers since "any" consumed more items.
            //
            // For individual flags we'll have current > 0
            if *current == 0 && max_consumed > 0 {
                *front = Arg::DUMMY;
                self.ctx.advance(max_consumed);
            }
        } else {
            println!("Max consumed from {front:?} {max_consumed:?}");
            self.ctx.advance(max_consumed)
        }

        Ok(max_consumed > 0)
    }

    /// Execute the parser
    ///
    /// `primary` is set to false for parsers like `--version` or `--help`, they
    /// don't care about improving the error message.
    ///
    /// TODO - `primary` seem like an optimization. Do I need it?
    pub(crate) fn run_parser<P, T>(mut self, parser: &'a P, primary: bool) -> Result<T, Error>
    where
        P: Parser<T> + ?Sized,
        T: 'static,
    {
        let root_id = self.next_id(1);
        let root_waker = self.waker_for(root_id);

        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead and keep the handle
        let mut handle = pin!(self.ctx.spawn(root_id, IdStrat::KeepBranch, parser));
        let mut root_cx = Context::from_waker(&root_waker);
        debug_assert!(handle.as_mut().poll(&mut root_cx).is_pending());

        let unparsed = self.inner_loop()?;

        let Poll::Ready(result) = handle.as_mut().poll(&mut root_cx) else {
            unreachable!("bpaf internal error: Failed to produce result");
        };

        let Some(unparsed) = unparsed else {
            // parsed everything - can't improve error message if it is an error
            return result;
        };
        if !primary {
            return result;
        }

        // TODO
        // if prefix_only {
        //     return result;
        // }

        let missing = match result {
            Ok(_) => None,
            Err(Error {
                message: Message::Missing(vec),
                offset: _,
            }) => Some(vec),
            e => return e,
        };

        let parsed = &self.ctx.args[0..self.ctx.cur()];
        let unparsed_raw = self.ctx.args[self.ctx.cur()].str();
        let mut v = ExplainUnparsed::new(missing, unparsed, unparsed_raw, parsed);
        parser.visit(&mut v);
        let message = v.explain();
        Err(Error {
            message,
            offset: self.ctx.cur(),
        })
    }

    /// Run configured parser for as long as possible,
    fn inner_loop(&mut self) -> Result<Option<Arg<'a>>, Error> {
        // mostly to avoid allocations
        let mut ids = Vec::new();
        let mut out = Vec::new();

        let mut prev_arg: Arg<'a> = Arg::DUMMY;

        let mut strict_pos = false;

        let mut prev_consumed = false;
        let no_parse = loop {
            self.ctx.set_term(false);
            self.propagate();
            println!("============= Propagate done");

            debug_assert!(self.ctx.shared.borrow().is_empty());
            debug_assert!(self.pending.lock().expect("poison").is_empty());

            let front = match &mut prev_arg {
                Arg::ShortSet { .. } if !prev_consumed => &mut prev_arg,
                Arg::ShortSet { current, names } if *current < names.len() => {
                    *current += 1;
                    &mut prev_arg
                }
                _ => {
                    if matches!(prev_arg, Arg::ShortSet { .. }) {
                        self.ctx.advance(1);
                    }
                    if let Some(val) = self.ctx.args.get(self.ctx.cur()) {
                        if val == "--" && !strict_pos {
                            strict_pos = true;
                            self.ctx.advance(1);
                            continue;
                        }
                        if strict_pos {
                            prev_arg = Arg::Positional {
                                value: val.as_ref(),
                            };
                        } else {
                            prev_arg =
                                split_param(val, &self.args, &self.flags).map_err(|message| {
                                    Error {
                                        message,
                                        offset: self.ctx.cur(),
                                    }
                                })?;
                        }
                        &mut prev_arg
                    } else {
                        println!("nothing to consume");
                        break false;
                    }
                }
            };

            self.pick_parsers(front, &mut ids);
            if ids.is_empty() {
                println!("No parsers for {front:?}, exiting");
                break true;
            }

            prev_consumed = self.consume(front, &mut ids, &mut out)?;
            println!("============= Consuming part done");
        };

        if no_parse {
            println!("==== loop done, no valid parser to handle {prev_arg:?}, cleaning up");
        }

        // at this point we are done consuming, either there's nothing more left or we don't know
        // how to consume the rest., let's run existing tasks to completion
        // first by waking them up all the consuming events (most of them fail with "not found")
        // and then by processing all the non-consuming events - this would either create some
        // errors or or those "not found" errors are going to be converted into something useful

        self.propagate();
        self.ctx.set_term(true);

        self.prepare_consumers_for_draining(&mut ids);

        *self.ctx.front_value.borrow_mut() = None;

        for id in ids.drain(..) {
            if let Some(task) = self.tasks.get_mut(&id) {
                println!("Need to terminate {id:?}");
                let (poll, consumed) = task.poll(id, None, &self.ctx);
                debug_assert_eq!(consumed, 0, "Consumed during termination?");
                debug_assert!(poll.is_ready(), "Termination left task stil pending?");
                self.handle_task_poll(id, poll);
            }
        }
        println!("Final propagation");
        self.propagate();
        Ok(no_parse.then_some(prev_arg))
    }

    /// Find the deepest left most child
    ///
    /// Assuming we did non consuming prcessing prior to that - it will be
    /// a consuming parser
    ///
    /// TODO - drop, it would produce invalid value if called on an item with no children
    fn first_child(&self, mut parent_id: Id) -> Id {
        for (child_id, task) in self.tasks.range(parent_id..).skip(1) {
            if task.parent == parent_id {
                parent_id = *child_id;
            } else {
                return parent_id;
            }
        }
        parent_id
    }

    #[inline(never)]
    /// Drain all consumers so they can be polled to emit default values
    fn prepare_consumers_for_draining(&mut self, ids: &mut Vec<Id>) {
        ids.extend(self.flags.values().flat_map(|v| v.iter()));
        ids.extend(self.positional.iter());
        ids.extend(self.args.values().flat_map(|v| v.iter()));
        ids.extend(self.literal.values().flat_map(|v| v.iter()));
        // TODO: any

        ids.sort();
        ids.dedup();
    }

    #[inline(never)]
    /// Handle potential task completion
    /// - notify parent if it is interested in error codes
    /// - queue task termination if task is complete
    ///
    /// It cannot run the task as well since to be able to handle sum types
    /// we need to be able to decide which tasks to terminate based on consumed length
    fn handle_task_poll(&mut self, id: Id, poll: Poll<PoisonHandle>) {
        if poll.is_pending() {
            return;
        }
        println!("Handling exit for {id:?}");
        let task = self.tasks.get_mut(&id).expect("We know it's there");
        if let Action::Trigger(trigger) = &task.action {
            match trigger {
                Trigger::Flag { names, action: _ } => {
                    remove_names_from_pecking(id, names, &mut self.flags);
                }
                Trigger::Arg { names, action: _ } => {
                    remove_names_from_pecking(id, names, &mut self.args);
                }
                Trigger::Positional { action: _ } => {
                    self.positional.remove(id);
                }
                Trigger::Literal { names, action: _ } => {
                    remove_names_from_pecking(id, names, &mut self.literal);
                }
                Trigger::Any { action: _ } => {
                    self.any.remove(id);
                }
            }
        }
        task.done = true;
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
