use crate::{
    ctx::Ctx,
    error::Error,
    named::Name,
    pecking::Pecking,
    split::{split_param, Arg, OsOrStr},
    visitor::{ExplainUnparsed, Group},
    Metavisit, Parser,
};
use std::{
    any::Any,
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    marker::PhantomData,
    pin::{pin, Pin},
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

// TODO:
// - make so tests are all passing
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

//pub(crate) mod family;
pub(crate) mod futures;

use self::futures::{ErrorHandle, JoinHandle};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(u32);
impl Id {
    pub(crate) const ZERO: Self = Self(0);
    const ROOT: Self = Self(1);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum NodeKind {
    Sum,
    Prod,
}

impl Id {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    pub(crate) fn sum(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Sum,
            id: self,
            field,
        }
    }

    pub fn prod(self, field: u32) -> Parent {
        Parent {
            kind: NodeKind::Prod,
            id: self,
            field,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Parent {
    pub(crate) kind: NodeKind,
    pub(crate) id: Id,
    pub(crate) field: u32,
}

// TODO - it should be possible to replace this with id of the first child attached
// to the field - they are always spawned tasks
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BranchId {
    pub(crate) parent: Id,
    pub(crate) field: u32,
}

impl std::fmt::Debug for BranchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B({}:{})", self.parent.0, self.field)
    }
}

impl BranchId {
    pub(crate) const ZERO: Self = Self {
        parent: Id(0),
        field: 0,
    };
    pub(crate) const ROOT: Self = Self {
        parent: Id::ROOT,
        field: 0,
    };
    pub(crate) fn succ(&self) -> Self {
        Self {
            parent: self.parent,
            field: self.field + 1,
        }
    }
}

pub type Action<'a> = Pin<Box<dyn Future<Output = ErrorHandle> + 'a>>;
pub(crate) struct Task<'a> {
    pub(crate) action: Action<'a>,
    // TODO - do I need "field" here?
    pub(crate) parent: Parent,
    // TODO - this can be a simple Id
    pub(crate) branch: BranchId,
    pub(crate) waker: Waker,
    pub(crate) consumed: u32,

    pub(crate) done: bool,
}

impl Task<'_> {
    fn poll(&mut self, id: Id, ctx: &Ctx) -> Poll<ErrorHandle> {
        *ctx.current_task.borrow_mut() = Some((self.branch, id));
        let mut cx = Context::from_waker(&self.waker);
        let poll = self.action.as_mut().poll(&mut cx);
        *ctx.current_task.borrow_mut() = None;
        poll
    }
}

pub(crate) enum Op<'a> {
    SpawnTask {
        parent: Parent,
        action: Action<'a>,
        keep_id: bool,
    },
    WakeTask {
        id: Id,
        error: Option<Error>,
    },
    /// Parent is no longer interested
    RemoveTask {
        id: Id,
    },
    AddNamedListener {
        flag: bool,
        names: &'a [Name<'static>],
        branch: BranchId,
        id: Id,
    },
    RemoveNamedListener {
        flag: bool,
        names: &'a [Name<'static>],
        branch: BranchId,
        id: Id,
    },
    AddFallback {
        branch: BranchId,
        id: Id,
    },
    RemoveFallback {
        branch: BranchId,
        id: Id,
    },
    AddPositionalListener {
        branch: BranchId,
        id: Id,
    },
    RemovePositionalListener {
        branch: BranchId,
        id: Id,
    },
    RestoreIdCounter {
        id: u32,
    },

    AddExitListener {
        parent: Id,
    },
    RemoveExitListener {
        parent: Id,
    },
    AddLiteral {
        branch: BranchId,
        id: Id,
        values: &'a [Name<'static>],
    },
    RemoveLiteral {
        branch: BranchId,
        id: Id,
        values: &'a [Name<'static>],
    },
    AddAny {
        branch: BranchId,
        id: Id,
    },

    RemoveAny {
        branch: BranchId,
        id: Id,
    },
}

pub type Fragment<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + 'a>>;

struct WakeTask {
    id: Id,
    pending: Arc<Mutex<Vec<Id>>>,
}

impl Wake for WakeTask {
    fn wake(self: std::sync::Arc<Self>) {
        // println!("Waking up {:?}", self.id);
        self.pending.lock().expect("poison").push(self.id);
    }
}

impl<'ctx> Runner<'ctx> {
    pub(crate) fn new(ctx: Ctx<'ctx>) -> Self {
        Self {
            tasks: BTreeMap::new(),
            pending: Default::default(),
            next_task_id: 0,
            parent_ids: Default::default(),
            wake_on_child_exit: Default::default(),
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

    /// Prod type items want to be notified about children exiting, in order
    /// they exit instead of order they are defined
    wake_on_child_exit: HashSet<Id>,

    /// For those tasks that asked us to retain the id according to the parent
    parent_ids: HashMap<Parent, u32>,

    // TODO - use HashMap?
    pub(crate) flags: HashMap<Name<'ctx>, Pecking>,
    pub(crate) args: HashMap<Name<'ctx>, Pecking>,
    fallback: Pecking,
    positional: Pecking,

    /// Parsers for "anything" they behave similar to positional parsers, but
    /// unlike regular positional items they can opt not to consume a value
    any: Pecking,

    literal: BTreeMap<Name<'ctx>, Pecking>,

    /// Shared with Wakers,
    ///
    /// contains a vector [`Id`] for tasks to wake up.
    pending: Arc<Mutex<Vec<Id>>>,
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
                        .map(|id| Op::WakeTask { id, error: None }),
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
                Op::SpawnTask {
                    parent,
                    action,
                    mut keep_id,
                } => {
                    let original_id = self.next_task_id;
                    if keep_id {
                        if let Some(prev_id) = self.parent_ids.get(&parent) {
                            self.next_task_id = *prev_id;
                            keep_id = false;
                        } else {
                            self.parent_ids.insert(parent, self.next_task_id);
                        }
                    }

                    let (id, waker) = self.next_id();

                    let branch = match parent.kind {
                        NodeKind::Sum => BranchId {
                            parent: parent.id,
                            field: parent.field,
                        },
                        NodeKind::Prod => self
                            .tasks
                            .get(&parent.id)
                            .map_or(BranchId::ROOT, |t| t.branch),
                    };
                    let mut task = Task {
                        action,
                        waker,
                        parent,
                        consumed: 0,
                        branch,
                        done: false,
                    };
                    self.ctx.items_consumed.set(0);
                    // try to immediately poll the task to handle task that produce
                    // the result without consuming anything from the input or spawning
                    // children: parsers such are `fail` or `pure` create those.
                    // If not handled here - they will be never woken up
                    match task.poll(id, &self.ctx) {
                        Poll::Ready(_r) => {
                            task.done = true;
                            todo!("Exited immediately");
                        }
                        Poll::Pending => {
                            // Only keep tasks that are not immediately resolved
                            assert!(self.tasks.insert(id, task).is_none());
                            println!("Spawned task {id:?} with parent {parent:?}");
                        }
                    }

                    // To execute parsers in depth first order we must
                    // execute children of the tasks as well as anything they
                    // need (adding listeners) before the siblings. Easiest way
                    // to do that is to rotate the queue by exact amount added - forward
                    let mut shared = self.ctx.shared.borrow_mut();
                    let mut after = shared.len();
                    if keep_id {
                        after += 1;
                        shared.push_back(Op::RestoreIdCounter { id: original_id });
                    }
                    // before:
                    // T S1 S2 S3
                    // task T is consumed and it spawns some children: C1/C2
                    // S1 S2 S3 C1 C2
                    // rotate right by 2:
                    // C1 C2 S1 S2 S3
                    shared.rotate_right(after - before);
                }
                Op::AddNamedListener {
                    flag,
                    names,
                    branch,
                    id,
                } => {
                    let map = if flag {
                        &mut self.flags
                    } else {
                        &mut self.args
                    };
                    for name in names.iter() {
                        map.entry(name.clone()).or_default().insert(branch, id);
                    }
                }
                Op::RemoveNamedListener {
                    flag,
                    names,
                    branch,
                    id,
                } => {
                    println!("{id:?}: Remove listener for {names:?}");

                    let map = if flag {
                        &mut self.flags
                    } else {
                        &mut self.args
                    };
                    for name in names {
                        let std::collections::hash_map::Entry::Occupied(mut entry) =
                            map.entry(name.clone())
                        else {
                            continue;
                        };
                        entry.get_mut().remove(branch, id);
                        if entry.get().is_empty() {
                            entry.remove();
                        }
                    }
                }
                Op::AddPositionalListener { branch, id } => {
                    println!("{id:?}: Add positional listener {id:?}");
                    self.positional.insert(branch, id);
                }
                Op::RemovePositionalListener { branch, id } => {
                    println!("{id:?}: Remove positional listener");
                    self.positional.remove(branch, id);
                }
                Op::RemoveTask { id } => {
                    println!("{id:?}: Removing task");
                    if let Some(task) = self.tasks.remove(&id) {
                        if let Some(parent) = self.tasks.get_mut(&task.parent.id) {
                            parent.consumed += task.consumed;
                        }
                    }
                }
                Op::WakeTask { id, error } => {
                    let Some(task) = self.tasks.get_mut(&id) else {
                        println!("waking up removed task {id:?}");
                        continue;
                    };
                    if task.done {
                        println!("waking up done task {id:?}");
                        continue;
                    }
                    println!("Waking {id:?} - consumed count is {:?}", task.consumed);
                    self.ctx.items_consumed.set(task.consumed);
                    self.ctx.child_exit.set(error);
                    let poll = task.poll(id, &self.ctx);
                    self.handle_task_poll(id, poll);
                }
                Op::RestoreIdCounter { id } => {
                    self.next_task_id = id + 1;
                }
                Op::AddExitListener { parent } => {
                    self.wake_on_child_exit.insert(parent);
                }
                Op::RemoveExitListener { parent } => {
                    self.wake_on_child_exit.remove(&parent);
                }
                Op::AddFallback { branch, id } => {
                    println!("Adding exit fallback to {id:?}");
                    self.fallback.insert(branch, id);
                }
                Op::RemoveFallback { branch, id } => {
                    println!("Removing exit fallback from {id:?}");
                    self.fallback.remove(branch, id);
                }
                Op::AddLiteral { branch, id, values } => {
                    for val in values {
                        self.literal
                            .entry(val.clone())
                            .or_default()
                            .insert(branch, id);
                    }
                }
                Op::RemoveLiteral { branch, id, values } => {
                    for val in values {
                        if let Some(e) = self.literal.get_mut(val) {
                            e.remove(branch, id);
                        }
                    }
                }
                Op::AddAny { branch, id } => {
                    self.any.insert(branch, id);
                }
                Op::RemoveAny { branch, id } => {
                    self.any.remove(branch, id);
                }
            }
        }
    }

    /// Pick one or more parsers that can handle front argument
    fn pick_parsers(&self, front: &mut Arg, ids: &mut Vec<(BranchId, Id)>) {
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
            if let Some((branch, id)) = self.fallback.heads().next() {
                let f = self.first_child(id);
                // Not the right branch, but the right one is not needed - it is only used
                // for deduplication. Should `ids` be `Vec<(Option<BranchId>, Id)>` instead?
                // let branch = self.tasks.get(&id).unwrap().branch;
                ids.push((branch, f));
                self.ctx.set_term(true);
            }
        } else {
            ids.sort(); // sort by branch
        }
    }

    fn consume(
        &mut self,
        front: &mut Arg<'a>,
        ids: &mut Vec<(BranchId, Id)>,
        out: &mut Vec<(Id, Poll<ErrorHandle>, usize)>,
    ) -> Result<bool, Error> {
        // actual feed consumption happens here
        let mut max_consumed = 0;

        assert!(!ids.is_empty());
        if let Arg::Named { value, .. } = front {
            *self.ctx.front_value.borrow_mut() = value.clone()
        }

        let mut last_branch = BranchId::ZERO;
        for (branch, id) in ids.drain(..) {
            if branch == last_branch {
                println!("skipping {id:?}");
                continue;
            }
            // each scheduled task gets a chance to run,
            if let Some(task) = self.tasks.get_mut(&id) {
                let (poll, consumed) = self.ctx.run_task(task);
                if poll.is_ready() {
                    last_branch = branch;
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
                todo!("got already terminated parser");
            }
        }

        for (id, poll, consumed) in out.drain(..) {
            if let Poll::Ready(eh) = &poll {
                if consumed < max_consumed {
                    // TODO  - custom error
                    eh.set(Some(Error::fail("terminated due to low priority")));
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
    /// don't care
    pub(crate) fn run_parser<P, T>(mut self, parser: &'a P, primary: bool) -> Result<T, Error>
    where
        P: Parser<T> + ?Sized,
        T: 'static,
    {
        let (root_id, root_waker) = self.next_id();

        // first - shove parser into a task so wakers can work
        // as usual. Since we care about the result - output type
        // must be T so it can't go into tasks directly.
        // We spawn it as a task instead and keep the handle
        let mut handle = pin!(self.ctx.spawn(root_id.prod(0), parser, false));
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
                message: crate::error::Message::Missing(vec),
            }) => Some(vec),
            e => return e,
        };

        let parsed = &self.ctx.args[0..self.ctx.cur()];
        let unparsed_raw = self.ctx.args[self.ctx.cur()].str();
        let mut v = ExplainUnparsed::new(missing, unparsed, unparsed_raw, parsed);
        parser.visit(&mut v);
        Err(v.explain())
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
                            prev_arg = split_param(val, &self.args, &self.flags)?;
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
        for (_branch, id) in ids.drain(..) {
            if let Some(task) = self.tasks.get_mut(&id) {
                println!("Need to terminate {id:?}");
                let (poll, consumed) = self.ctx.run_task(task);
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
    fn first_child(&self, mut parent_id: Id) -> Id {
        for (child_id, task) in self.tasks.range(parent_id..).skip(1) {
            if task.parent.id == parent_id {
                parent_id = *child_id;
            } else {
                return parent_id;
            }
        }
        parent_id
    }

    #[inline(never)]
    /// Drain all consumers so they can be polled to emit default values
    fn prepare_consumers_for_draining(&mut self, ids: &mut Vec<(BranchId, Id)>) {
        ids.extend(self.flags.values().flat_map(|v| v.iter()));
        ids.extend(self.positional.iter());
        ids.extend(self.args.values().flat_map(|v| v.iter()));
        ids.extend(self.literal.values().flat_map(|v| v.iter()));
        // TODO: any

        ids.sort_by_key(|(_b, id)| *id);
        ids.dedup_by_key(|(_b, id)| *id);
        ids.sort();
    }

    #[inline(never)]
    /// Handle potential task completion
    /// - notify parent if it is interested in error codes
    /// - queue task termination if task is complete
    ///
    /// It cannot run the task as well since to be able to handle sum types
    /// we need to be able to decide which tasks to terminate based on consumed length
    fn handle_task_poll(&mut self, id: Id, poll: Poll<ErrorHandle>) {
        let Poll::Ready(handle) = poll else {
            return;
        };
        println!("Handling exit for {id:?}");
        let task = self.tasks.get_mut(&id).expect("We know it's there");
        task.done = true;
        if self.wake_on_child_exit.contains(&task.parent.id) {
            let error = handle.take();
            handle.set(error.clone());
            self.ctx.shared.borrow_mut().push_front(Op::WakeTask {
                id: task.parent.id,
                error,
            });
        }
        self.ctx
            .shared
            .borrow_mut()
            .push_back(Op::RemoveTask { id });
    }

    /// Allocate next task [`Id`] and a [`Waker`] for that task.
    fn next_id(&mut self) -> (Id, Waker) {
        let id = self.next_task_id;
        self.next_task_id += 1;
        let id = Id::new(id);
        let pending = self.pending.clone();
        let waker = Waker::from(Arc::new(WakeTask { id, pending }));
        (id, waker)
    }
}

pub struct Alt<T: 'static> {
    pub items: Vec<Box<dyn Parser<T>>>,
}

impl<T> Parser<T> for Box<dyn Parser<T>>
where
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.as_ref().run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.as_ref().visit(visitor)
    }
}

impl<T> Parser<T> for Rc<dyn Parser<T>>
where
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.as_ref().run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.as_ref().visit(visitor)
    }
}

struct AltFuture<'a, T> {
    handles: Vec<JoinHandle<'a, T>>,
}

impl<T> Future for AltFuture<'_, T> {
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(!self.as_ref().handles.is_empty());
        for (ix, mut h) in self.as_mut().handles.iter_mut().enumerate() {
            if let Poll::Ready(r) = pin!(h).poll(cx) {
                // This future can be called multiple times, as long as there
                // are handles to be consumed
                self.handles.remove(ix);
                return Poll::Ready(r);
            }
        }
        Poll::Pending
    }
}

impl<T> Parser<T> for Alt<T>
where
    T: Clone + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let (_branch, id) = ctx.current_id();

            // Spawn a task for all the branches
            let handles = self
                .items
                .iter()
                .enumerate()
                .map(|(ix, p)| ctx.spawn(id.sum(ix as u32), p, false))
                .collect::<Vec<_>>();

            let mut fut = AltFuture { handles };
            // TODO: this should be some low priority error
            let mut res = Error::empty();

            // must collect results as they come. If

            // return first succesful result or the best error
            while !fut.handles.is_empty() {
                let hh = (&mut fut).await;
                res = match (res, hh) {
                    (ok @ Ok(_), _) | (Err(_), ok @ Ok(_)) => return ok,
                    (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
                }
            }
            res
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Sum);
        for i in &self.items {
            i.visit(visitor);
        }
        visitor.pop_group();
    }
}

// [`downcast`] + [`hint`] are used to smuggle type of the field inside the [`Con`]
pub fn downcast<T: 'static>(_: PhantomData<T>, parser: &Box<dyn Any>) -> &Rc<dyn Parser<T>> {
    parser.downcast_ref().expect("Can't downcast")
}
pub fn hint<T: 'static>(_: impl Parser<T>) -> PhantomData<T> {
    PhantomData
}

pub struct Con<T> {
    pub visitors: Vec<Box<dyn Metavisit>>,
    pub parsers: Vec<Box<dyn Any>>,

    #[allow(clippy::type_complexity)] // And who's fault is that?
    pub run: Box<dyn for<'a> Fn(&'a [Box<dyn Any>], Ctx<'a>) -> Fragment<'a, T>>,
}

impl<T> Parser<T> for Con<T>
where
    T: std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        (self.run)(&self.parsers, ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Prod);
        for v in &self.visitors {
            v.visit(visitor);
        }
        visitor.pop_group();
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
