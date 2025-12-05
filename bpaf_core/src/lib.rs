mod arg;
mod args;
mod complete;
mod consumers;
mod core_consumers;
mod error;
mod os_str;
mod utils;
mod visitors;

use crate::{
    arg::{Adjacency, Arg, lex_os_arg},
    args::Args,
    complete::CompleteReq,
    utils::{Vec1, reuse_vec},
    visitors::{BetterName, IsAcceptedOnce, ValidCommand},
};
#[doc(inline)]
pub use crate::{consumers::*, error::*, traits::*};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::{OsStr, OsString},
    pin::Pin,
    rc::Rc,
    task::Poll,
};

pub(crate) struct JoinHandle<T> {
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn take(self) -> Result<T, Error> {
        self.result
            .take()
            .unwrap_or(Err(Error::Silent("Empty JoinHandle?")))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Bp<I>(I);

mod adapters;

mod traits;

pub mod __private {
    pub use crate::{Alt, Con, Ctx, Error, Kind};
}

#[macro_export]
macro_rules! construct {
    // === capture initial shape of the query

    // `construct!(Enum::Cons { a, b, c })`
    ($ns:ident $(:: $con:ident)* { $($rest:tt)* }) =>
        {{ $crate::construct!(@prepare [named $ns $(:: $con)*] [] $($rest)*) }};

    // `construct!(Enum::Cons ( a, b, c ))`
    ($ns:ident $(:: $con:ident)* ( $($rest:tt)* )) =>
        {{ $crate::construct!(@prepare [pos $ns $(:: $con)*] [] $($rest)*) }};

    // construct!( a, b, c )
    ($first:ident $($rest:tt)*) => // first to make sure we have at least one item
        {{ $crate::construct!(@prepare [pos] [] $first $($rest)*) }};

    // construct!([a, b, c])
    ([$first:ident $($rest:tt)*]) => // first - to make sure we have at lest one item
        {{ $crate::construct!(@prepare [alt] [] $first $($rest)*) }};

    // === expand function calls in argument lists, if any
    // this is done for both prod and sum type constructors

    // instantiate field from a function call with possible arguments
    (@prepare $ty:tt [$($fields:tt)*] $field:ident ($($param:tt)*) $(, $($rest:tt)*)? ) => {{
        let $field = $field($($param)*);
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)*)?)
    }};
    // field is already a variable - we can use it as is.
    (@prepare $ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)* )?)
    }};

    // === fields are done (no 4th argument), can start constructing parsers

    // All the logic for sum parser sits inside of Alt datatype
    (@prepare [alt] [ $($field:ident)*]) => {
        $crate::__private::Alt { items: ::std::vec![ $($field.into_rc()),*] }
    };

    // For product type the logic is a bit more complicated - do one more step
    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::construct!(@fin $ty [ $($fields)* ])
    };

    // === Making a body for the product parser

    // Two special cases where we construct something with no fields, use `Parser::pure` for that
    (@fin [named $($con:tt)+] []) => { $crate::pure($($con)+ { })};
    (@fin [pos   $($con:tt)+] []) => { $crate::pure($($con)+ ( ))};

    (@fin $ty:tt [$($fields:ident)*]) => {{
        $( let $fields = $fields.into_rc(); )*

        // This allocates...
        let visits = vec![ $($crate::Visited::into_box($fields.clone()) ),* ];

        let run:  ::std::boxed::Box<dyn ::std::ops::Fn($crate::__private::Ctx) ->
        ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::std::result::Result<_, $crate::__private::Error>>> =
            ::std::boxed::Box::new(move |ctx: $crate::__private::Ctx| {
                $( let $fields = ctx.spawn($crate::__private::Kind::Prod, $fields.clone());)*
                ::std::boxed::Box::new(||{
                    let mut err = ::std::option::Option::None;
                    $( let $fields = $fields.take().map_err(|e| e.append_to(&mut err)); )*

                    if let ::std::option::Option::Some(err) = err {
                        return ::std::result::Result::Err(err);
                    }

                    ::std::result::Result::Ok::<_, $crate::__private::Error>
                        ($crate::construct!(@make $ty [$($fields)*]))
                })
            });

        $crate::__private::Con { run, visits }

    }};

    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    (@make [named $($con:tt)+] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields?),* } };
    // for positional - (), if there's no constructor - we are making a tuple
    (@make [pos   $($con:tt)*] [$($fields:ident)*]) => { $($con)* ( $($fields?),* ) };
}

pub struct Con<T> {
    #[allow(clippy::type_complexity)]
    run: Box<dyn Fn(Ctx) -> Box<dyn FnOnce() -> Result<T, Error>>>,
    // TODO - this is a whole lot of allocations, we can achieve the same results
    // by adding a Visited implementation for (A, B, ...) then recursively folding $fields
    // into (A, (B, (... )))
    visits: Vec<Box<dyn Visited>>,
}

impl<T: 'static> Parser<T> for Con<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        // TODO - explain
        let closure = (*self.run)(ctx);
        r#yield().await;
        closure()
    }
}

impl<T: 'static> Visited for Con<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(traits::Group::Prod);
        for item in &self.visits {
            item.visit(visitor);
        }
        visitor.pop_group();
    }
}

pub struct Alt<T> {
    items: Vec<Bp<RcParser<T>>>,
}

impl std::ops::Add for Error {
    type Output = Error;

    fn add(self, rhs: Self) -> Self::Output {
        self.combine(rhs)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Scope {
    start: Id,
    end: Id,
}

impl Scope {
    const ALL: Scope = Scope {
        start: Id(0),
        end: Id(u32::MAX),
    };
    fn contains(&self, id: Id) -> bool {
        self.start <= id && id < self.end
    }
}

#[derive(Debug)]
enum Op {
    Spawn(Task),
    RegisterSum { id: Id, scope: Scope },
    DeregisterSum { id: Id },
    KillScope { cursor: u32, scope: Scope },
}

impl<T: 'static> Visited for Alt<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(traits::Group::Sum);
        for i in &self.items {
            i.0.visit(visitor);
        }
        visitor.pop_group();
    }
}

impl<T: 'static> Parser<T> for Alt<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let id = ctx.current_task.borrow().id;
        let mut scopes = Vec::new();
        let handles = self
            .items
            .iter()
            .map(|parser| {
                let (h, scope) = ctx.scoped_spawn(parser.clone().0, Kind::Sum);
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
        r#yield().await;
        ctx.trim_children(scopes).await;

        let mut acc = Error::Silent("Empty Alt?");
        for h in handles {
            match h.take() {
                Err(err) => acc = acc + err,
                v => return v,
            }
        }
        Err(acc)
    }
}

struct Task {
    act: Pin<Box<dyn Future<Output = ()>>>,
    info: TaskInfo,
}

impl std::fmt::Debug for Task {
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
#[derive(Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct Metavar(&'static str);

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Id(u32);

pub type Fragment<T> = Pin<Box<dyn Future<Output = Result<T, Error>>>>;

pub struct RawCtx {
    /// Scheduled ops
    ///
    /// Active tasks are living outside of the `RawCtx`: we need to borrow them to poll
    /// anyway but also need to borrow to spawn new tasks from polled tasks. Having them
    /// outside makes the code a bit cleaner
    ///
    /// Holds other operations that might need access to tasks or other internal structures
    pending_ops: RefCell<VecDeque<Op>>,
    /// Reference to triggers used by the executor
    ///
    /// Not available (borrowed by executor) during the first stage
    triggers: RefCell<Triggers>,
    /// Early exit ranges
    ///
    /// When there's no matching triggers executor will try to terminate anything inside of
    /// the each pair. This allows is necessary for things like `.optional()` and `.many()` to work
    early_exit: RefCell<BTreeSet<Scope>>,
    /// Next free Id
    ///
    /// Tasks can use it to allocate Ids for children tasks including overriding
    next_free: Cell<u32>,
    args: Args,
    cursor: Cell<usize>,
    /// When task is woken up this contains a reason for it
    wakeup_reason: RefCell<Reason>,
    /// Reference to [`TaskInfo`] for the current task
    ///
    /// Executor sets it to the right value when polling a task
    current_task: RefCell<TaskInfo>,
    /// We keep track of all the early terminated branches, saving each termination along with
    /// the position - from that we'll deduce conflict info
    conflicts: RefCell<Vec<Conflict>>,
}
pub type Ctx = Rc<RawCtx>;

#[derive(Debug)]
enum Conflict {
    /// Named item - flag, argument
    Named { pos: u32, name: Name<'static> },
    /// Literal - command
    Lit { pos: u32, name: Cow<'static, str> },
    /// Positional item
    Pos { pos: u32 },
}

#[derive(Debug)]
enum Reason {
    Arg(Option<Arg<'static>>),
    Pass,
    NoPass,
    Push,
    ChildProgress(Vec1<Id>),
    Complete(CompleteReq),
}

fn view_reactor_flags(
    reactor: &mut Triggers,
) -> (
    &mut HashMap<char, PeckingOrder>,
    &mut HashMap<Cow<'static, str>, PeckingOrder>,
) {
    (&mut reactor.short_flags, &mut reactor.long_flags)
}

fn view_reactor_args(
    reactor: &mut Triggers,
) -> (
    &mut HashMap<char, PeckingOrder>,
    &mut HashMap<Cow<'static, str>, PeckingOrder>,
) {
    (&mut reactor.short_args, &mut reactor.long_args)
}

type LongNamePeckingOrder = HashMap<Cow<'static, str>, PeckingOrder>;
type ShortNamePeckingOrder = HashMap<char, PeckingOrder>;

type NamedPeckingOrderSelector =
    fn(&mut Triggers) -> (&mut ShortNamePeckingOrder, &mut LongNamePeckingOrder);

fn make_handle<T: 'static>() -> (Rc<Cell<Option<Result<T, Error>>>>, JoinHandle<T>) {
    let result = Rc::new(Cell::new(None));
    (result.clone(), JoinHandle { result })
}

impl RawCtx {
    fn task_parent_and_id(&self) -> (Parent, Id) {
        let cur = self.current_task.borrow();
        (cur.parent_id, cur.id)
    }

    /// After consumption when called from a leaf node returns what was consumed
    pub(crate) fn leaf_range(&self) -> Option<(u32, u32)> {
        if !matches!(&*self.wakeup_reason.borrow(), Reason::Pass) {
            return None;
        }
        let start = self.cursor.get() as u32;
        let end = start + self.current_task.borrow().consumed;
        (end > start).then_some((start, end))
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
    //
    // TODO - Do I really need Bp wrapper here?
    fn make_raw_task<T: 'static>(
        self: &Rc<Self>,
        parser: Bp<RcParser<T>>,
    ) -> (JoinHandle<T>, Pin<Box<impl Future<Output = ()> + 'static>>) {
        let ctx = self.clone();
        let (out, handle) = make_handle();
        let action = Box::pin(async move { out.set(Some(parser.0.run(ctx).await)) });
        (handle, action)
    }
    fn make_act<T: 'static>(
        self: &Rc<Self>,
        out: Rc<Cell<Option<Result<T, Error>>>>,
        parser: RcParser<T>,
    ) -> Pin<Box<impl Future<Output = ()> + 'static>> {
        let ctx = self.clone();
        Box::pin(async move { out.set(Some(parser.run(ctx).await)) })
    }

    fn spawn<T: 'static>(self: &Rc<RawCtx>, kind: Kind, parser: Bp<RcParser<T>>) -> JoinHandle<T> {
        let (handle, act) = self.make_raw_task(parser);
        let info = self.make_child_info(kind);
        let task = Task { act, info };
        self.add_task(task);
        handle
    }

    /// Spawn a parser and collect the range it occupies
    fn scoped_spawn<T: 'static>(
        self: &Rc<RawCtx>,
        parser: RcParser<T>,
        kind: Kind,
    ) -> (JoinHandle<T>, Scope) {
        let start = self.next_free.get();
        let h = self.spawn(kind, Bp(parser));
        let end = self.next_free.get();
        (
            h,
            Scope {
                start: Id(start),
                end: Id(end),
            },
        )
    }

    fn spawn_with_early_exit<T: 'static>(
        self: &Rc<RawCtx>,
        parser: RcParser<T>,
    ) -> (JoinHandle<T>, Scope) {
        let start = Id(self.next_free.get());
        let h = self.spawn(Kind::Prod, Bp(parser));
        let end = Id(self.next_free.get());
        let scope = Scope { start, end };
        self.early_exit.borrow_mut().insert(scope);
        (h, scope)
    }

    fn remove_early_exit(&self, scope: Scope) {
        let removed = self.early_exit.borrow_mut().remove(&scope);
        assert!(removed);
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
    fn add_task(&self, mut task: Task) {
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
        use std::{
            ops::DerefMut,
            task::{Context, Waker},
        };
        const NOOP: Context<'static> = Context::from_waker(Waker::noop());
        std::mem::swap(&mut task.info, self.current_task.borrow_mut().deref_mut());
        #[expect(const_item_mutation)]
        let done = task.act.as_mut().poll(&mut NOOP).is_ready();
        std::mem::swap(&mut task.info, self.current_task.borrow_mut().deref_mut());
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
                                cursor: self.cursor.get() as u32,
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

struct Executor {
    ctx: Ctx,
    tasks: BTreeMap<Id, Task>,
    to_wake: Vec<Id>,
    to_propagate: VecDeque<(Id, Parent, u32)>,
    sums: BTreeMap<Id, Scope>,
}

impl Executor {
    fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            tasks: BTreeMap::new(),
            to_wake: Vec::new(),
            to_propagate: VecDeque::new(),
            sums: BTreeMap::new(),
        }
    }

    // try to parse a set of short flags `-vvv` as separate flags `-v -v -v`
    fn execute_group(&mut self, group: Group) -> Result<(), Error> {
        let res = self.execute_group_inner(&group.0);
        if res.is_ok() {
            self.ctx.cursor.update(|c| c + 1);
        } else {
            self.kill_in_scope(Scope::ALL);
        }
        res
    }

    fn execute_group_inner(&mut self, names: &[char]) -> Result<(), Error> {
        let mut mixer_capacity = Mixer::default();
        for (ix, name) in names.iter().copied().enumerate() {
            // set a wakeup reason, just in case
            *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(Some(Arg::Named {
                name: Name::Short(name),
                value: None,
            }));

            let mut mixer = mixer_capacity.reuse_capacity();
            let triggers = self.ctx.triggers.borrow();
            mixer.populate_short_flag(&name, &triggers);
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
                return Err(Error::Problem(problem));
            }
            mixer_capacity = mixer.reuse_capacity();
            drop(triggers);
            self.stage_2(1);
            self.propagate(Reason::Push);
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
                Op::KillScope { scope, cursor } => {
                    let old = self.ctx.cursor.replace(cursor as usize);
                    self.kill_in_scope(scope);
                    self.ctx.cursor.set(old);
                }
                Op::DeregisterSum { id } => {
                    self.sums.remove(&id);
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

        let triggers = self.ctx.triggers.borrow();
        // TODO - populate_prefix doesn't really have to be a method on Mixer
        // it can be a standalone method
        let (reason, mgroup) = mixer.populate_prefix(&arg, &triggers);
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
        drop(triggers);

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
        self.kill_in_scope(Scope::ALL);
        self.propagate(Reason::Push);
        Some(Ok(()))
    }

    fn execute(&mut self, parser: &dyn Visited) -> Result<(), Error> {
        // Keep running as long as we are making progress:
        // - consuming new items
        // -
        let mut mixer_capacity = Mixer::default();
        loop {
            assert!(self.to_propagate.is_empty());
            self.process_scheduled();

            let ctx = self.ctx.clone();
            let Some(front) = ctx.args.get(self.ctx.cursor.get()) else {
                break;
            };

            if let Some(out) = self.check_autocomplete(front) {
                return out;
            }

            // Waking up tasks is done in two stages: during the first stage we wake up
            // all the tasks with matching triggers, during the second stage tasks that don't
            // consume the biggest amount from the first stage are terminated.
            //
            // This allows to parsing things as both flag and an argument (`--foo` | `--foo BAR`)
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
                self.propagate(Reason::Push);
                continue;
            }

            if best_size == 0
                && let Some(scope) = { self.ctx.early_exit.borrow().last().copied() }
            {
                self.kill_in_scope(scope);
                continue;
            }

            if best_size == 0 {
                self.kill_in_scope(Scope::ALL);
                return Err(self.complain_about(front, parser).into());
            }

            self.stage_2(best_size);

            self.propagate(Reason::Push);

            self.ctx.cursor.update(|c| c + best_size as usize);
        }

        self.kill_in_scope(Scope::ALL);

        println!("somehow we are done, {:?}", self.tasks);
        assert!(
            self.tasks.is_empty(),
            "All tasks should be terminated when exiting execution"
        );
        assert!(self.ctx.pending_ops.borrow().is_empty());
        // assert!(
        //     self.sums.is_empty(),
        //     "All sums should be removed, {:?}",
        //     self.sums
        // );

        Ok(())
    }

    fn complain_about(&self, unexpected: &OsString, parser: &dyn Visited) -> Problem {
        match lex_os_arg(unexpected) {
            Arg::Named {
                name: unexpected,
                value: _,
            } => {
                // is it a conflict?
                for conflict in self.ctx.conflicts.borrow().iter() {
                    if let Conflict::Named { pos, name: dropped } = conflict
                        && &unexpected == dropped
                    {
                        return Problem::Conflict {
                            accepted: self.ctx.args[*pos as usize].clone(),
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
                let mut visitors = (once, better);
                parser.visit(&mut visitors);
                if let Some(problem) = visitors.0.into_problem() {
                    return problem;
                }
                if let Some(problem) = visitors.1.and_then(|v| v.into_problem()) {
                    return problem;
                }
            }
            Arg::Pos { value } => {
                if let Some(target) = value.to_str() {
                    let mut is_command = ValidCommand::new(target);
                    parser.visit(&mut is_command);
                    if let Some(problem) = is_command.into_problem() {
                        return problem;
                    }
                }
            }
        }
        Problem::Unconsumed {
            value: unexpected.clone(),
        }
    }

    fn kill_in_scope(&mut self, scope: Scope) {
        *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(None);
        for (_, mut task) in self
            .tasks
            .extract_if(scope.start..scope.end, |_, t| t.info.pending == 0)
        {
            let done = self.ctx.poll_in_context(&mut task);
            assert!(done);
            if task.info.parent_id == Parent(0) {
                continue;
            }
            self.to_propagate
                .push_back((task.info.id, task.info.parent_id, task.info.consumed));
        }
        self.propagate(Reason::Arg(None));
    }

    /// Run the first stage of the trigger
    ///
    /// Each task indicates how much it will consume if allowed to run till the end
    fn stage_1(
        &mut self,
        front: &OsStr,
        mixer_capacity: Mixer<'static>,
    ) -> (u32, Mixer<'static>, Option<Group>) {
        let mut mixer = mixer_capacity.reuse_capacity();
        let reactor = self.ctx.triggers.borrow();
        let arg = lex_os_arg(front);
        let mgroup = mixer.populate(&arg, &reactor);

        *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(Some(arg.into_owned()));

        let mut best_size = 0;
        while let Some(next) = mixer.consume_next_item(&self.tasks) {
            let task = &mut self.tasks.get_mut(&next.id).unwrap();
            let r = self.ctx.poll_in_context(task);
            assert!(!r);
            if task.info.consumed > 0 {
                self.to_wake.push(next.id);
            } else {
                mixer.pass();
            }

            best_size = best_size.max(task.info.consumed);
        }

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
            *self.ctx.wakeup_reason.borrow_mut() = if term { Reason::NoPass } else { Reason::Pass };

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
    fn propagate(&mut self, reason: Reason) {
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
        *self.ctx.wakeup_reason.borrow_mut() = reason;
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

mod pecking {
    //! # Pecking order - what parsers get to run
    //!
    //! Crate allows having arbitrary algebraic trees for parsers composed of sums (enums) and
    //! products (structs), it also allows to have multiple parsers matching the same trigger -
    //! name or position.
    //!
    //! At the same time we must be careful about parsing:
    //! 1. In the final result each CLI gets consumed by exactly one parser
    //! 2. All the CLI items are consumed.
    //!
    //! During the execution we might consume the same item from multiple parsers - as long as they
    //! go into parallel branches of a sum and gets disambiguated later.
    //!
    //! Rules for running as follow:
    //!
    //! 1. If a parser from a product runs - no other parser from the same product can consume this
    //!    item - all the items from this product go into the final result
    //! 2. Parsers from the same sum can run in parallel
    //!
    //! Some examples:
    //!
    //! - `A + B` - Both can run
    //! - `A * B` - Only `A` can run
    //! - `(A + B) + C` - Run all 3
    //! - `(A + B) * (C + D)` - Run `A` and `B`
    //! - `(A * B) + (C * D)` - Run `A` and `C`

    use crate::{Id, Parent};
    use std::collections::BTreeSet;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub(crate) struct Item {
        pub(crate) parent: Parent,
        pub(crate) id: Id,
    }

    impl Item {
        pub(crate) fn next_item(&self) -> Item {
            Item {
                id: Id(self.id.0 + 1),
                ..*self
            }
        }
        pub(crate) fn next_family_item(&self) -> Item {
            Item {
                parent: Parent(self.parent.0 + 1),
                id: Id(0),
            }
        }
    }

    #[derive(Debug, Default)]
    pub(crate) struct PeckingOrder(Pecking);
    impl PeckingOrder {
        pub(crate) fn is_empty(&self) -> bool {
            match &self.0 {
                Pecking::Empty => true,
                Pecking::Single { item: _ } => false,
                Pecking::Set { set } => set.is_empty(),
            }
        }
        pub(crate) fn peek(&self) -> Option<Item> {
            match &self.0 {
                Pecking::Empty => None,
                Pecking::Single { item } => Some(*item),
                Pecking::Set { set } => set.first().copied(),
            }
        }
    }

    #[derive(Debug, Default)]
    enum Pecking {
        #[default]
        Empty,
        Single {
            item: Item,
        },
        Set {
            set: BTreeSet<Item>,
        },
    }

    impl PeckingOrder {
        pub(crate) fn insert(&mut self, parent: Parent, id: Id) {
            let new_item = Item { parent, id };
            match &mut self.0 {
                Pecking::Empty => self.0 = Pecking::Single { item: new_item },
                Pecking::Single { item } => {
                    let mut set = BTreeSet::new();
                    set.insert(*item);
                    set.insert(new_item);
                    self.0 = Pecking::Set { set };
                }
                Pecking::Set { set } => {
                    set.insert(new_item);
                }
            }
        }

        /// Remove an item from the pecking order, returns `false` if PO is now empty
        pub(crate) fn remove(&mut self, parent: Parent, id: Id) -> bool {
            let to_remove = Item { parent, id };
            let ok = match &mut self.0 {
                Pecking::Empty => false,
                Pecking::Single { item } => {
                    let ok = *item == to_remove;
                    self.0 = Pecking::Empty;
                    ok
                }
                Pecking::Set { set } => set.remove(&to_remove),
            };
            debug_assert!(ok, "Couldn't remove absent {to_remove:?} from {self:?}!");
            match &self.0 {
                Pecking::Empty => true,
                Pecking::Single { .. } => false,
                Pecking::Set { set } => set.is_empty(),
            }
        }

        /// Get the next item that might or might not belong to this family
        pub(crate) fn item_after(&self, prev: Item) -> Option<&Item> {
            match &self.0 {
                Pecking::Empty => None,
                Pecking::Single { item } => (prev < *item).then_some(item),
                Pecking::Set { set } => set.range(prev.next_item()..).next(),
            }
        }
    }
}

#[derive(Debug, Default)]
struct Tracker {
    seen: BTreeSet<Parent>,
    stack: Vec<Parent>,
}

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
    /// returns if current value can run and if
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
                    (true, Kind::Sum) => WalkResult::PickAndSame,
                    (true, Kind::Prod) => WalkResult::PickAndNext,
                    (false, Kind::Sum) => WalkResult::PickAndSame,
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
    fn mk_dummy_task(id: u32, parent: u32, parent_kind: Kind) -> (Id, Task) {
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

#[derive(Default, Debug)]
struct Triggers {
    // -f
    short_flags: HashMap<char, PeckingOrder>,
    // --foo
    long_flags: HashMap<Cow<'static, str>, PeckingOrder>,
    // -f=bar
    short_args: HashMap<char, PeckingOrder>,
    // `--foo=bar`
    long_args: HashMap<Cow<'static, str>, PeckingOrder>,
    // `foo`
    pos: PeckingOrder,
    // `-1`
    any: PeckingOrder,
    // `a`, `alpha`
    literal: HashMap<Cow<'static, str>, PeckingOrder>,
}

// reactor - listens for readiness, notifies tasks
// executor - runs ready tasks

impl RawCtx {
    /// Create a copy of a context suitable to run an executor
    fn fork(&self) -> Rc<RawCtx> {
        Self::make(self.args.clone(), self.cursor.get())
    }

    fn new(args: Args) -> Ctx {
        Self::make(args, 0)
    }

    fn make(args: Args, cursor: usize) -> Rc<RawCtx> {
        Rc::new(RawCtx {
            args,
            current_task: Default::default(),
            cursor: Cell::new(cursor),
            early_exit: Default::default(),
            pending_ops: Default::default(),
            next_free: Cell::new(1),
            triggers: Default::default(),
            wakeup_reason: RefCell::new(Reason::Pass),
            conflicts: Default::default(),
        })
    }

    fn execute(self: &Ctx, parser: &dyn Visited) -> Result<(), Error> {
        Executor::new(self.clone()).execute(parser)
    }
}

#[derive(Default, Debug)]
struct Mixer<'a> {
    pecking: Vec<&'a PeckingOrder>,
    tracker: Tracker,
    same_family: bool,
    prev: Option<pecking::Item>,
}
#[must_use]
#[derive(Debug)]
struct Group(Vec<char>);

impl<'a> Mixer<'a> {
    fn populate_short_flag(&mut self, name: &char, triggers: &'a Triggers) {
        self.pecking_push(triggers.short_flags.get(name));
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
    ) -> (CompleteReq, Option<Group>) {
        self.pecking_push(Some(&triggers.any));
        match arg {
            Arg::Named {
                name: nn @ Name::Long(name),
                value: None,
            } => {
                for (n, po) in triggers.long_args.iter() {
                    if n.starts_with(name.as_ref()) {
                        self.pecking_push(Some(po));
                    }
                }
                for (n, po) in triggers.long_flags.iter() {
                    if n.starts_with(name.as_ref()) {
                        self.pecking_push(Some(po));
                    }
                }
                let prefix = Some(name.as_ref().into());
                (CompleteReq::Name { prefix }, None)
            }
            Arg::Named {
                name: Name::Short(s),
                value: None,
            } => {
                self.pecking_push(triggers.short_args.get(s));
                self.pecking_push(triggers.short_flags.get(s));
                (CompleteReq::Name { prefix: None }, None)
            }
            Arg::Named {
                name,
                value: Some((adj, val)),
            } => {
                self.pecking.clear(); // wipe triggers.any since they can't fire // TODO - why?
                self.pecking_push(match name {
                    Name::Short(s) => triggers.short_args.get(s),
                    Name::Long(l) => triggers.long_args.get(l.as_ref()),
                });
                let req = match val.to_str() {
                    Some(p) => CompleteReq::Literal { prefix: p.into() },
                    None => todo!(),
                };
                (req, None)
            }
            Arg::Pos { value } => {
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
                if short {
                    for f in triggers.short_args.values() {
                        self.pecking_push(Some(f));
                    }
                    for f in triggers.short_flags.values() {
                        self.pecking_push(Some(f));
                    }
                }
                if long {
                    for f in triggers.long_args.values() {
                        self.pecking_push(Some(f));
                    }
                    for f in triggers.long_flags.values() {
                        self.pecking_push(Some(f));
                    }
                }
                if lit {
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
    fn populate(&mut self, arg: &Arg<'a>, triggers: &'a Triggers) -> Option<Group> {
        self.pecking_push(Some(&triggers.any));
        match arg {
            Arg::Named {
                name: Name::Long(name),
                value: _,
            } => {
                self.pecking_push(triggers.long_args.get(name));
                self.pecking_push(triggers.long_flags.get(name));
            }
            Arg::Named {
                name: Name::Short(name),
                value: None | Some((Adjacency::WithEq, _)),
            } => {
                self.pecking_push(triggers.short_args.get(name));
                self.pecking_push(triggers.short_flags.get(name));
            }
            Arg::Named {
                name: Name::Short(name),
                value: Some((Adjacency::Immediate, val)),
            } => {
                let prev_len = self.pecking.len();
                self.pecking_push(triggers.short_args.get(name));
                if let Some(_) = triggers.short_flags.get(name)
                    && self.pecking.len() == prev_len
                    && let Some(chars) = val.to_str()
                    && chars.chars().all(|k| triggers.short_flags.contains_key(&k))
                {
                    // There's no way to parse it as a single argument, but it might work
                    // if we treat it as a merged group of single letter flags

                    let mut group = vec![*name];
                    group.extend(chars.chars());
                    return Some(Group(group));
                } else {
                    self.pecking_push(triggers.short_flags.get(name));
                }
            }
            Arg::Pos { value } => {
                if let Some(name) = value.to_str() {
                    self.pecking_push(triggers.literal.get(name));
                }
                self.pecking_push(Some(&triggers.pos));
            }
        }
        None
    }

    /// Add a non-empty pecking order to the mix
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

    fn pass(&mut self) {
        self.same_family = true;
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

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Name<'a> {
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

#[derive(Debug, Clone)]
pub(crate) enum ShortLong {
    Short(char),
    Long(Cow<'static, str>),
    Both(char, Cow<'static, str>),
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

#[cfg(test)]
mod tests {
    mod complete;
    mod errors;
    mod osstring;
    mod unsorted;
    mod wake;
}
