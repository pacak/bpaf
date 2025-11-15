use std::{
    borrow::Cow,
    cell::{Cell, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    pin::Pin,
    rc::Rc,
    task::Poll,
};

pub struct JoinHandle<T> {
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.result.take() {
            Some(r) => Poll::Ready(r),
            None => Poll::Pending,
        }
    }
}

impl<T> JoinHandle<T> {
    fn take(self) -> Result<T, Error> {
        self.result.take().unwrap_or(Err(Error::Killed))
    }
}

mod traits {
    //! [`Parser`] trait and related private helper traits

    use crate::{Ctx, Error};
    use std::{pin::Pin, rc::Rc};

    pub trait Parser<T: 'static> {
        fn run(&self, ctx: Ctx) -> impl Future<Output = Result<T, Error>>;

        /// Convert the parser into a boxed, reference counted version
        fn into_rc(self) -> RcParser<T>
        where
            Self: Sized + Parser<T> + 'static,
        {
            RcParser(Rc::new(self))
        }
    }

    /// Helper trait that allows shoving non-dyn compatible trait [`Parser`] into an [`Rc`]
    trait DynParser<T: 'static> {
        fn dyn_run(&self, ctx: Ctx) -> Pin<Box<dyn Future<Output = Result<T, Error>> + '_>>;
    }

    impl<T: 'static, P: Parser<T>> DynParser<T> for P {
        fn dyn_run(&self, ctx: Ctx) -> Pin<Box<dyn Future<Output = Result<T, Error>> + '_>> {
            Box::pin(<Self as Parser<T>>::run(self, ctx))
        }
    }

    impl<T: 'static> Parser<T> for RcParser<T> {
        async fn run(&self, ctx: Ctx) -> Result<T, Error> {
            self.0.as_ref().dyn_run(ctx).await
        }
    }

    /// Reference counted boxed [`Parser<T>`](Parser) - it is cheap to clone
    pub struct RcParser<T>(Rc<dyn DynParser<T>>);

    impl<T> Clone for RcParser<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
}

pub mod __private {
    pub use crate::{Con, Ctx, Error, Kind};
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

        $crate::__private::Con { run }

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
}

impl<T: 'static> Parser<T> for Con<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        // TODO - explain
        let closure = (*self.run)(ctx);
        r#yield().await;
        closure()
    }
}

#[doc(inline)]
pub use traits::*;

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

#[derive(Debug)]
pub enum Error {
    MissingItem(&'static str),
    ParseError,
    Killed,
    Internal,
}
impl Error {
    fn combine(self, e2: Error) -> Error {
        match (self, e2) {
            (Error::Internal, _e) | (_e, Error::Internal) => Error::Internal,
            (Error::Killed, e) | (e, Error::Killed) => e,
            (Error::MissingItem(_), Error::MissingItem(_)) => todo!(),
            (Error::MissingItem(_), Error::ParseError) => todo!(),
            (Error::ParseError, Error::MissingItem(_)) => todo!(),
            (Error::ParseError, Error::ParseError) => todo!(),
        }
    }

    fn can_catch(&self) -> bool {
        match self {
            Error::MissingItem(_) => true,
            Error::ParseError => false,
            Error::Killed => true,
            Error::Internal => false,
        }
    }

    /// Consume current error and append it to a growing collection in dst
    ///
    /// It exists to collect errors from multiple handles and designed to work with
    /// [`Result::map_err`]. We aggregate the best possible error inside an Option
    /// and fail with that if it is present
    fn append_to(self, dst: &mut Option<Error>) -> Self {
        *dst = Some(match dst {
            Some(e) => todo!("{e:?} {self:?}"),
            None => self,
        });
        Error::Killed
    }
}

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
    fn as_parent(&self) -> Parent {
        Parent(self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Id(u32);

pub type Fragment<T> = Pin<Box<dyn Future<Output = Result<T, Error>>>>;

pub struct RawCtx {
    /// Storage for to-be-spawned tasks
    ///
    /// Active tasks are living outside of the `RawCtx`
    new_tasks: RefCell<Vec<Task>>,
    triggers: RefCell<Triggers>,
    next_free: Cell<u32>,
    args: RefCell<Vec<String>>,
    cursor: Cell<usize>,
    wakeup_reason: RefCell<Reason>,
    current_task: RefCell<TaskInfo>,
}
pub type Ctx = Rc<RawCtx>;

enum Reason {
    Term(bool),
    Positional { value: String },
    Named { name: Name, value: Option<String> },
}

struct DummyFlag(char);

impl Parser<bool> for DummyFlag {
    async fn run(&self, ctx: Ctx) -> Result<bool, Error> {
        let names = [Name::Short(self.0)];
        let r = ctx.parse_flag(&names[..]).await;
        println!("{r:?} / {names:?} {:?}", ctx.current_task.borrow().id);
        r
    }
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

impl RawCtx {
    fn task_parent_and_id(&self) -> (Parent, Id) {
        let cur = self.current_task.borrow();
        (cur.parent_id, cur.id)
    }

    fn add_names(&self, names: &[Name], selector: NamedPeckingOrderSelector) -> Result<(), Error> {
        let (parent, id) = self.task_parent_and_id();
        let (mut short, mut long) = RefMut::map_split(self.triggers.borrow_mut(), selector);
        for name in names {
            match name {
                Name::Short(c) => short.entry(*c).or_default().insert(parent, id),
                Name::Long(s) => long.entry(s.clone()).or_default().insert(parent, id),
            }
        }
        Ok(())
        //
    }

    fn remove_names(
        &self,
        names: &[Name],
        selector: NamedPeckingOrderSelector,
    ) -> Result<(), Error> {
        let (parent, id) = self.task_parent_and_id();
        let (mut short, mut long) = RefMut::map_split(self.triggers.borrow_mut(), selector);

        use std::collections::hash_map::Entry;
        for name in names {
            match name {
                Name::Short(c) => {
                    if let Entry::Occupied(mut e) = short.entry(*c) {
                        if e.get_mut().remove(parent, id) {
                            e.remove();
                        }
                    } else if cfg!(debug_assertions) {
                        panic!("Tried to remove missing {name:?}");
                    }
                }
                Name::Long(s) => {
                    if let Entry::Occupied(mut e) = long.entry(s.clone()) {
                        if e.get_mut().remove(parent, id) {
                            e.remove();
                        }
                    } else if cfg!(debug_assertions) {
                        panic!("Tried to remove missing {name:?}");
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert a parser into a task that saves its output to a [`JoinHandle`]
    fn make_raw_task<T: 'static>(
        self: &Rc<Self>,
        parser: RcParser<T>,
    ) -> (JoinHandle<T>, Pin<Box<impl Future<Output = ()> + 'static>>) {
        let ctx = self.clone();
        let result_out = Rc::new(Cell::new(None));
        let result = result_out.clone();
        let action = Box::pin(async move { result_out.set(Some(parser.run(ctx).await)) });
        (JoinHandle { result }, action)
    }

    fn spawn<T: 'static>(self: &Rc<Self>, kind: Kind, parser: RcParser<T>) -> JoinHandle<T> {
        let (handle, act) = self.make_raw_task(parser);
        let info = self.make_child_info(kind);
        let task = Task { act, info };
        self.add_task(task);
        handle
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
        if self.poll_in_context(&mut task) {
            let mut cur = self.current_task.borrow_mut();
            cur.pending -= 1;
            debug_assert_eq!(task.info.consumed, 0);
        } else {
            self.new_tasks.borrow_mut().push(task);
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

    fn consume(&self, cnt: u32) {
        self.current_task.borrow_mut().consumed += cnt;
    }

    /// Parse a flag by any one of the given names
    ///
    /// - `Ok(true)` when encounters a name
    /// - `Ok(false)` when it gets terminated by optional logic
    /// - `Err(Error::Killed)` when it gets out-consumed by something else
    async fn parse_flag(&self, names: &[Name]) -> Result<bool, Error> {
        self.add_names(names, view_reactor_flags)?;
        r#yield().await; // ------------------------------------------
        let mut term = false;
        let mut res = match *self.wakeup_reason.borrow() {
            Reason::Term(true) => {
                term = true;
                Ok(false)
            }
            Reason::Term(false) => unreachable!(),
            Reason::Named {
                name: _,
                value: None,
            } => Ok(true),
            Reason::Named { .. } | Reason::Positional { .. } => unreachable!(),
        };
        if term {
            self.remove_names(names, view_reactor_flags)?;
            return res;
        }
        self.consume(1);
        r#yield().await; // ------------------------------------------
        self.remove_names(names, view_reactor_flags)?;
        if matches!(*self.wakeup_reason.borrow(), Reason::Term(true)) {
            res = Err(Error::Killed);
        }
        res
    }

    // async fn parse_arg(self: Rc<Self>, names: &[Name]) -> Result<Error, Option<String>> {
    //     self.add_names(names, reactor_flags)?;
    //     r#yield().await;
    //     let res = match *self.wakeup_reason.borrow() {
    //         Reason::Term => Ok(None),
    //         Reason::Named {
    //             name: _,
    //             value: None,
    //         } => Ok(true),
    //         Reason::Named { .. } | Reason::Positional { .. } => unreachable!(),
    //     };
    //     self.remove_names(names, reactor_flags)?;
    //     r#yield().await;
    //     res
    //
    // }
    //
    // async fn wake_on_literal(self: Rc<Self>, value: &str) -> bool {
    //     todo!()
    // }
    //
    // async fn wake_on_positional(self: Rc<Self>) -> Result<Error, String> {
    //     todo!()
    // }
    //
    // async fn wake_on_any(
    //     self: Rc<Self>,
    //     check: Box<dyn Fn(&str) -> Box<dyn Any>>,
    // ) -> Option<Box<dyn Any>> {
    //     todo!()
    // }
}

struct Executor {
    ctx: Ctx,
    tasks: BTreeMap<Id, Task>,
    to_wake: Vec<Id>,
    to_wake2: VecDeque<(Id, u32)>,
}

impl Executor {
    fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            tasks: Default::default(),
            to_wake: Default::default(),
            to_wake2: Default::default(),
        }
    }

    fn execute(&mut self) -> Result<(), Error> {
        let mut changed = true;

        // Keep running as long as we are making progress:
        // - consuming new items
        // -
        let mut mixer_capacity = Mixer::default();
        while changed {
            changed = false;

            // Spawn new tasks. They been polled once inside the `Self::add_task`
            // so triggers should be all in place and none would exit immediately.
            for task in self.ctx.new_tasks.borrow_mut().drain(..) {
                let old = self.tasks.insert(task.info.id, task);
                debug_assert!(old.is_none(), "Dupliated task id?");
            }

            let ctx = self.ctx.clone();
            let args = ctx.args.borrow();
            let Some(front) = args.get(self.ctx.cursor.get()) else {
                break;
            };

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
            let (best_size, mixer) = self.stage_1(front, mixer_capacity);
            mixer_capacity = mixer;

            let ambiguity = false;

            if ambiguity {
                // create new context, new executor? run stuff in the new context?
            }

            if best_size > 0 {
                self.ctx.cursor.update(|c| c + best_size as usize);
                changed = true;
            } else {
                todo!("Didn't consume anything, complain about {front:?} and prepare to exit?");
            }
            self.stage_2(best_size);

            self.propagate();
        }

        *self.ctx.wakeup_reason.borrow_mut() = Reason::Term(true);
        while let Some((id, mut task)) = self.tasks.pop_last() {
            if self.tasks.is_empty() {
                *self.ctx.wakeup_reason.borrow_mut() = Reason::Term(false);
            }
            let exited = self.ctx.poll_in_context(&mut task);
            debug_assert!(exited, "Task {id:?} {task:?} failed to exit");
        }

        println!("somehow we are done");

        Ok(())
    }

    /// Run the first stage of the trigger
    ///
    /// Each task indicates how much it will consume if allowed to run till the end
    fn stage_1(&mut self, front: &str, mixer_capacity: Mixer<'static>) -> (u32, Mixer<'static>) {
        let mut mixer = mixer_capacity.reuse_capacity();
        let reactor = self.ctx.triggers.borrow();
        let arg = mixer.populate(front, &reactor);
        *self.ctx.wakeup_reason.borrow_mut() = match arg {
            Arg::Named { name, value } => Reason::Named { name, value },
            Arg::Pos { value } => Reason::Positional { value },
        };

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

        (best_size, mixer.reuse_capacity())
    }

    /// Run the second stage of the trigger
    ///
    /// Depending on how much a task is consuming relative to other tasks
    /// task are either allowed to run or are being terminated
    fn stage_2(&mut self, best_size: u32) {
        use std::collections::btree_map::Entry;
        for id in self.to_wake.drain(..) {
            // TODO - I think `entry` API can be replaced with just removing
            let Entry::Occupied(mut task) = self.tasks.entry(id) else {
                todo!("Second stage got a task id that isn't there?");
            };
            let task_ref = task.get_mut();
            let term = task_ref.info.consumed < best_size;
            *self.ctx.wakeup_reason.borrow_mut() = Reason::Term(term);

            if self.ctx.poll_in_context(task_ref) {
                let task = task.remove();
                let size = if term { 0 } else { task.info.consumed };
                self.to_wake2.push_back((task.info.parent_id.as_id(), size))
            } else {
                todo!("I think this should never run");
            }
        }
    }

    // Propagate trigger results to parents recursively
    fn propagate(&mut self) {
        while let Some((id, consumed)) = self.to_wake2.pop_front() {
            let std::collections::btree_map::Entry::Occupied(mut task) = self.tasks.entry(id)
            else {
                todo!("Unknown task in propagete?")
            };
            let task_ref = task.get_mut();
            task_ref.info.pending -= 1;
            task_ref.info.consumed += consumed;
            if task_ref.info.pending > 0 {
                continue;
            }
            if self.ctx.poll_in_context(task_ref) {
                let task = task.remove();
                if task.info.parent_id.is_root() {
                    // the parent is root = there's no task to wake up
                    continue;
                }
                self.to_wake2
                    .push_back((task.info.parent_id.as_id(), task.info.consumed));
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

        pub(crate) fn front(&self) -> Option<&Item> {
            match &self.0 {
                Pecking::Empty => None,
                Pecking::Single { item } => Some(item),
                Pecking::Set { set } => set.first(),
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

#[derive(Default)]
struct RCache {
    items: BTreeSet<(pecking::Item, PeckingOrder)>,
    seen: BTreeSet<Parent>,
}

impl RCache {
    fn clear(&mut self) {
        self.items.clear();
        self.seen.clear();
    }
}

impl RawCtx {
    fn new(args: Vec<String>) -> Rc<RawCtx> {
        Rc::new(RawCtx {
            new_tasks: Default::default(),
            triggers: Default::default(),
            next_free: Cell::new(1),
            args: args.into(),
            wakeup_reason: RefCell::new(Reason::Term(false)),
            current_task: Default::default(),
            cursor: Cell::new(0),
            // term: Default::default(),
        })
    }

    fn execute(self: &Ctx) -> Result<(), Error> {
        Executor::new(self.clone()).execute()
    }
}

#[derive(Default, Debug)]
struct Mixer<'a> {
    pecking: Vec<&'a PeckingOrder>,
    to_wake: Vec<Id>,
    tracker: Tracker,
    same_family: bool,
    prev: Option<pecking::Item>,
}

impl<'a> Mixer<'a> {
    fn populate(&mut self, front: &str, triggers: &'a Triggers) -> Arg {
        self.pecking_push(&triggers.any);
        let parsed = split(front);
        match &parsed {
            Arg::Named {
                name: Name::Long(name),
                value: _,
            } => {
                if let Some(order) = triggers.long_args.get(name) {
                    self.pecking_push(order);
                }
                if let Some(order) = triggers.long_flags.get(name) {
                    self.pecking_push(order);
                }
            }
            Arg::Named {
                name: Name::Short(name),
                value: _,
            } => {
                if let Some(order) = triggers.short_args.get(name) {
                    self.pecking_push(order);
                }
                if let Some(order) = triggers.short_flags.get(name) {
                    self.pecking_push(order);
                }
            }
            Arg::Pos { value: _ } => {
                self.pecking_push(&triggers.pos);
            }
        }
        parsed
    }

    fn pecking_push(&mut self, order: &'a PeckingOrder) {
        if !order.is_empty() {
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

    // fn next_item(&self, prev: pecking::Item) -> Option<pecking::Item> {
    //     // TODO - apply filtering and throw out peckings we can't possibly match
    //     let mut res = None;
    //     for order in self
    //         .pecking
    //         .iter()
    //         .filter_map(|v| v.po_next_item(prev))
    //         .copied()
    //     {
    //         res = Some(res.map_or(order, |old| order.min(old)));
    //     }
    //     res
    // }
    //
    // fn next_family(
    //     &mut self,
    //     tasks: &BTreeMap<Id, Task>,
    //     mut prev: pecking::Item,
    // ) -> Option<pecking::Item> {
    //     println!("Checking next family to {prev:?}");
    //     loop {
    //         let min = self
    //             .pecking
    //             .iter()
    //             .filter_map(|v| v.po_next_family(prev))
    //             .copied()
    //             .min()?;
    //
    //         // In most cases we expect to have just a single trigger for an item so we can skip all this
    //         // walking business and do it only when we actually need to decide if two parsers can run
    //         // concurrently. This is just an optimization.
    //         if self.tracker.is_empty() {
    //             self.tracker.walk_tasks_up(prev.id, tasks);
    //         }
    //         if self.tracker.walk_tasks_up(min.id, tasks) {
    //             println!("Got {min:?}");
    //             return Some(min);
    //         } else {
    //             prev = min.next_family_item();
    //         }
    //     }
    // }
}

impl<'a> Mixer<'a> {
    /// Reuse the capacity inside the executor while decoupling the lifetimes
    fn reuse_capacity<'b>(mut self) -> Mixer<'b> {
        self.to_wake.clear();
        self.tracker.clear();
        Mixer {
            pecking: reuse_vec(self.pecking),
            to_wake: self.to_wake,
            tracker: self.tracker,
            same_family: false,
            prev: None,
        }
    }

    fn pass(&mut self) {
        self.same_family = true;
    }

    fn consume_next_item(&mut self, tasks: &BTreeMap<Id, Task>) -> Option<pecking::Item> {
        println!("Prev: {:?}", self.prev);
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
                println!("Prev is {prev:?}, next best item is {best_candidate:?}");

                if self.tracker.is_empty() {
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

        println!("Produced => {:?}\n", self.prev);
        self.prev
    }

    // fn consume_next(
    //     &mut self,
    //     tasks: &BTreeMap<Id, Task>,
    //     prev: &mut Option<(pecking::Item, Kind)>,
    //     same_family: bool,
    // ) -> Option<pecking::Item> {
    //     println!("Need to consume next to {prev:?} with same family {same_family:?}");
    //     match prev {
    //         Some((prev, kind)) => {
    //             if same_family
    //             /* || *kind == Kind::Sum  */
    //             {
    //                 println!("Looking at next item");
    //                 self.next_item(*prev)
    //             } else {
    //                 println!("Looking at next family");
    //                 self.next_family(tasks, *prev)
    //             }
    //         }
    //         None => self.first(),
    //     }
    // }
}

fn reuse_vec<'a, 'b, T>(mut v: Vec<&'a T>) -> Vec<&'b T> {
    v.clear();
    v.into_iter().map(|_| unreachable!()).collect()
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum Name {
    Short(char),
    Long(Cow<'static, str>),
}

impl From<char> for Name {
    fn from(value: char) -> Self {
        Name::Short(value)
    }
}

impl From<&'static str> for Name {
    fn from(value: &'static str) -> Self {
        Name::Long(Cow::Borrowed(value))
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name::Long(Cow::Owned(value))
    }
}

#[derive(Debug)]
enum Arg {
    Named { name: Name, value: Option<String> },
    Pos { value: String },
}

fn split(val: &str) -> Arg {
    if let Some(name) = val.strip_prefix("--") {
        match name.split_once('=') {
            Some((name, val)) => {
                let name = name.to_owned().into();
                let value = Some(val.to_owned());
                Arg::Named { name, value }
            }
            None => {
                let name = name.to_owned().into();
                let value = None;
                Arg::Named { name, value }
            }
        }
    } else if let Some(name) = val.strip_prefix("-") {
        match name.split_once('=') {
            Some((name, val)) => {
                let name = name.chars().next().unwrap().into();
                let value = Some(val.to_owned());
                Arg::Named { name, value }
            }
            None => {
                let name = name.chars().next().unwrap().into();
                let value = None;
                Arg::Named { name, value }
            }
        }
    } else {
        Arg::Pos {
            value: val.to_owned(),
        }
    }
}

pub struct Args(pub(crate) Vec<String>);
impl<const N: usize> From<[&'static str; N]> for Args {
    fn from(value: [&'static str; N]) -> Self {
        Args(value.into_iter().map(String::from).collect())
    }
}

fn run<T: 'static>(parser: impl Parser<T> + 'static, args: &[&str]) -> Result<T, Error> {
    let args = args.iter().copied().map(|a| a.to_owned()).collect();
    let ctx = RawCtx::new(args);

    let (handle, act) = ctx.make_raw_task(parser.into_rc());
    let info = ctx.make_child_info(Kind::Prod);
    let task = Task { act, info };
    ctx.add_task(task);
    ctx.execute()?;
    handle.take()
}

#[test]
fn parse_simple_works_1() {
    let a = DummyFlag('a');
    let b = DummyFlag('b');
    let parser = construct!(a, b);
    let r = run(parser, &["-a", "-b"]).unwrap();
    assert_eq!(r, (true, true));
}

#[test]
fn parse_simple_works_2() {
    let a = DummyFlag('a');
    let b = DummyFlag('a');
    let parser = construct!(a, b);
    let r = run(parser, &["-a", "-a"]).unwrap();
    assert_eq!(r, (true, true));
}

#[cfg(test)]
mod wake_tests {
    //! This module contains tests for scenarios when we want to wake up multiple parsers on the same
    //! trigger, potentially from several different pecking sets.

    use super::*;

    macro_rules! arranged {
        (($a:ident $ab:tt $b:ident) $abcd:tt ($c:ident $cd:tt $d:ident)) => {
            arrange(
                [$a.clone(), $b.clone(), $c.clone(), $d.clone()],
                arranged!($abcd),
                arranged!($ab),
                arranged!($cd),
            )
        };
        (+) => {
            Kind::Sum
        };
        (*) => {
            Kind::Prod
        };
    }

    fn run_arranged(
        parser: impl Parser<[Result<bool, Error>; 4]> + 'static,
        input: &[&str],
    ) -> [bool; 4] {
        let r = run(parser, input).unwrap();
        let v = r
            .into_iter()
            .map(|v| v.unwrap_or(false))
            .collect::<Vec<_>>();
        v.try_into().unwrap()
    }

    #[test]
    fn short_flags_only() {
        let a = DummyFlag('a').into_rc();

        assert_eq!(
            run_arranged(arranged!((a * a) * (a * a)), &["-a"]),
            [true, false, false, false]
        );

        assert_eq!(
            run_arranged(arranged!((a + a) + (a + a)), &["-a"]),
            [true, true, true, true]
        );

        assert_eq!(
            run_arranged(arranged!((a * a) + (a + a)), &["-a"]),
            [true, false, true, true]
        );

        assert_eq!(
            run_arranged(arranged!((a + a) + (a * a)), &["-a"]),
            [true, true, true, false]
        );

        assert_eq!(
            run_arranged(arranged!((a + a) * (a + a)), &["-a"]),
            [true, true, false, false]
        );
    }

    fn arrange<T: 'static>(
        ps: [RcParser<T>; 4],
        kind_1234: Kind,
        kind_12: Kind,
        kind_34: Kind,
    ) -> impl Parser<[Result<T, Error>; 4]> {
        #![allow(clippy::type_complexity)]

        let run: Box<dyn Fn(Ctx) -> Box<dyn FnOnce() -> Result<[Result<T, Error>; 4], Error>>> =
            Box::new(move |ctx: Ctx| {
                let cur_kind = ctx.current_task.borrow().parent_kind;
                let info1234 = ctx.make_child_info(cur_kind);
                let info12 = ctx.make_child_info(kind_1234);
                let info34 = ctx.make_child_info(kind_1234);

                let (ha, act) = ctx.make_raw_task(ps[0].clone());
                let mut info = ctx.make_child_info(kind_12);
                info.parent_id = info12.id.as_parent();
                ctx.add_task(Task { act, info });

                let (hb, act) = ctx.make_raw_task(ps[1].clone());
                let mut info = ctx.make_child_info(kind_12);
                info.parent_id = info12.id.as_parent();
                ctx.add_task(Task { act, info });

                let (hc, act) = ctx.make_raw_task(ps[2].clone());
                let mut info = ctx.make_child_info(kind_34);
                info.parent_id = info34.id.as_parent();
                ctx.add_task(Task { act, info });

                let (hd, act) = ctx.make_raw_task(ps[3].clone());
                let mut info = ctx.make_child_info(kind_34);
                info.parent_id = info34.id.as_parent();
                ctx.add_task(Task { act, info });

                ctx.add_task(Task {
                    act: Box::pin(async { r#yield().await }),
                    info: TaskInfo {
                        pending: 2,
                        parent_id: info1234.id.as_parent(),
                        ..info12
                    },
                });

                ctx.add_task(Task {
                    act: Box::pin(async { r#yield().await }),
                    info: TaskInfo {
                        pending: 2,
                        parent_id: info1234.id.as_parent(),
                        ..info34
                    },
                });

                ctx.add_task(Task {
                    act: Box::pin(async { r#yield().await }),
                    info: TaskInfo {
                        pending: 2,
                        ..info1234
                    },
                });
                ctx.current_task.borrow_mut().pending -= 6;

                Box::new(|| Ok([ha.take(), hb.take(), hc.take(), hd.take()]))
            });
        Con { run }
    }
}
