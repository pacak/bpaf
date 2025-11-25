mod arg;
mod args;
mod consumers;
mod os_str;
mod utils;
mod core_consumers {
    //! This module contains the basic consumers for the arguments
    //!
    //! They know how to interact with executor, deal with early exit when necessary, etc.
    //! Anything else gets built on top of those consumers

    pub(crate) type AnyCheck = Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>;
    pub(crate) type AnyResult = Option<Box<dyn std::any::Any>>;
    use crate::*;

    impl RawCtx {
        #[inline(never)]
        pub(crate) fn add_names(
            &self,
            names: &[Name<'static>],
            selector: NamedPeckingOrderSelector,
        ) -> Result<(), Error> {
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

        #[inline(never)]
        pub(crate) fn remove_names(
            &self,
            names: &[Name<'static>],
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

        pub(crate) fn consume(&self, cnt: u32) {
            self.current_task.borrow_mut().consumed += cnt;
        }

        // pub(crate) async fn parse_any(&self, check: check: Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>) -> Result<Option<Box<dyn std::any::Any>>, Error> {
        //
        // }

        fn add_to_po(&self, lens: fn(&mut Triggers) -> &mut PeckingOrder) {
            let (parent, id) = self.task_parent_and_id();
            lens(&mut self.triggers.borrow_mut()).insert(parent, id);
        }
        fn remove_from_po(&self, lens: fn(&mut Triggers) -> &mut PeckingOrder) {
            let (parent, id) = self.task_parent_and_id();
            lens(&mut self.triggers.borrow_mut()).remove(parent, id);
        }

        pub(crate) async fn parse_pos(&self) -> Result<Option<OsString>, Error> {
            self.add_to_po(|triggers| &mut triggers.pos);
            r#yield().await;
            let res = self
                .handle_early_exit(None, |arg| self.parse_pos_consume(arg))
                .await;
            self.remove_from_po(|triggers| &mut triggers.pos);
            self.managed_to_survive()?;
            res
        }

        pub(self) fn parse_pos_consume(&self, arg: &Arg) -> Result<Option<OsString>, Error> {
            match arg {
                Arg::Named { .. } => unreachable!(),
                Arg::Pos { value } => {
                    self.consume(1);
                    Ok(Some(value.clone().into_owned()))
                }
            }
        }

        pub(crate) async fn parse_any(&self, check: AnyCheck) -> Result<AnyResult, Error> {
            self.add_to_po(|triggers| &mut triggers.any);
            r#yield().await;
            let res = loop {
                println!("Trying to run Any");
                let res = self
                    .handle_early_exit(None, |_arg| self.parse_any_consume(&check))
                    .await;

                if !matches!(res, Ok(None)) {
                    break res;
                }
            };
            self.remove_from_po(|triggers| &mut triggers.any);
            self.managed_to_survive()?;
            res
        }

        pub(self) fn parse_any_consume(&self, check: &AnyCheck) -> Result<AnyResult, Error> {
            let cursor = self.cursor.get();
            let args = self.args.borrow();
            let res = check(&args[cursor]);
            println!(
                "Tried to parse {:?}, got {:?} ",
                args[cursor],
                res.is_some()
            );
            if res.is_some() {
                self.consume(1);
            }
            Ok(res)
        }

        pub(crate) async fn parse_arg(
            &self,
            names: &[Name<'static>],
        ) -> Result<Option<OsString>, Error> {
            self.add_names(names, view_reactor_args)?;
            r#yield().await;
            let res = self
                .handle_early_exit(None, |arg| self.parse_arg_consume(arg))
                .await;
            self.remove_names(names, view_reactor_args)?;
            self.managed_to_survive()?;
            res
        }

        pub(self) fn parse_arg_consume(&self, arg: &Arg) -> Result<Option<OsString>, Error> {
            match arg {
                Arg::Pos { .. } => unreachable!(),
                Arg::Named { name, value: None } => {
                    let cursor = self.cursor.get() + 1;
                    let args = self.args.borrow();
                    let Some(arg) = args.get(cursor) else {
                        todo!("{name:?} expects a value");
                    };
                    match lex_os_arg(arg) {
                        Arg::Named { .. } => {
                            todo!("{name:?} got {arg:?}, try {name:?}={arg:?}")
                        }
                        Arg::Pos { value } => {
                            self.consume(2);
                            Ok(Some(value.clone().into_owned()))
                        }
                    }
                }
                Arg::Named {
                    name: _,
                    value: Some((_adj, val)),
                } => {
                    self.consume(1);
                    Ok(Some(val.clone().into_owned()))
                }
            }
        }

        /// Parse a flag by any one of the given names
        ///
        /// - `Ok(true)` when encounters a name
        /// - `Ok(false)` when it gets terminated by "no such item"
        /// - `Err(Error::Killed)` when it gets out-consumed by something else
        pub(crate) async fn parse_flag(&self, names: &[Name<'static>]) -> Result<bool, Error> {
            self.add_names(names, view_reactor_flags)?;
            r#yield().await;
            let res = self
                .handle_early_exit(false, |arg| self.parse_flag_consume(arg))
                .await;
            self.remove_names(names, view_reactor_flags)?;
            self.managed_to_survive()?;
            res
        }

        /// Consume a present argument, advance
        #[inline(always)]
        pub(self) fn parse_flag_consume(&self, arg: &Arg) -> Result<bool, Error> {
            match arg {
                Arg::Named { name, value } => match value {
                    Some(val) => {
                        todo!("Expected flag {name:?}, got value {val:?}");
                    }
                    None => {
                        self.consume(1);
                        Ok(true)
                    }
                },
                Arg::Pos { value: _ } => unreachable!(),
            }
        }

        /// Handle early exit conditions
        #[inline(always)]
        pub(self) async fn handle_early_exit<T>(
            &self,
            fallback: T,
            act: impl Fn(&Arg) -> Result<T, Error>,
        ) -> Result<T, Error> {
            let res = {
                let reason = self.wakeup_reason.borrow();
                let Reason::Arg(arg) = &*reason else {
                    return Err(Error::Killed);
                };
                let Some(arg_ref) = arg.as_ref() else {
                    return Ok(fallback);
                };
                act(arg_ref)
            };
            r#yield().await;
            res
        }

        fn managed_to_survive(&self) -> Result<(), Error> {
            if matches!(*self.wakeup_reason.borrow(), Reason::NotConsumedEnough) {
                Err(Error::Killed)
            } else {
                Ok(())
            }
        }
    }
}

use crate::{
    arg::{Adjacency, Arg, lex_os_arg},
    args::Args,
    core_consumers::{AnyCheck, AnyResult},
    utils::reuse_vec,
};
#[doc(inline)]
pub use consumers::*;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    ffi::{OsStr, OsString},
    pin::Pin,
    rc::Rc,
    task::Poll,
};
#[doc(inline)]
pub use traits::*;

pub(crate) struct JoinHandle<T> {
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn take(self) -> Result<T, Error> {
        self.result.take().unwrap_or(Err(Error::Killed))
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Bp<I>(I);

mod adapters {
    //! Adapters that implement functionality used by the [`Parser`] trait

    use crate::{
        Bp, Error, Kind, Name, Parser, RawCtx, RcParser, Reason, Task, args::Args, r#yield,
    };
    use std::{borrow::Cow, marker::PhantomData};

    pub(crate) struct Map<P, F, T, R> {
        pub(crate) inner: P,
        pub(crate) map: F,
        pub(crate) ctx: PhantomData<(T, R)>,
    }

    impl<T: 'static, P: Parser<T>, R: 'static, F: Fn(T) -> R> Parser<R> for Map<P, F, T, R> {
        async fn run(&self, ctx: crate::Ctx) -> Result<R, crate::Error> {
            let t = self.inner.run(ctx).await?;
            Ok((self.map)(t))
        }
    }

    pub(crate) struct Optional<T> {
        pub(crate) inner: RcParser<T>,
    }
    impl<T: 'static> Parser<Option<T>> for Optional<T> {
        async fn run(&self, ctx: crate::Ctx) -> Result<Option<T>, Error> {
            // TODO - use scoped spawn, Scope and get rid of spawn_with_early_exit
            let (h, pair) = ctx.spawn_with_early_exit(self.inner.clone());
            r#yield().await;
            ctx.remove_early_exit(pair);
            match h.take() {
                Ok(v) => Ok(Some(v)),
                Err(err) => {
                    if err.can_catch() && ctx.current_task.borrow().consumed == 0 {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                }
            }
        }
    }

    pub struct OptionParser<T> {
        pub(crate) inner: RcParser<T>,
    }

    impl<T: 'static> Bp<OptionParser<T>> {
        pub fn run_inner(&self, args: impl Into<Args>) -> Result<T, Error> {
            let items = args.into().items;

            let ctx = RawCtx::new(items);

            let (handle, act) = ctx.make_raw_task(Bp(self.0.inner.clone()));
            let info = ctx.make_child_info(Kind::Prod);
            let task = Task { act, info };
            ctx.add_task(task);
            ctx.execute()?;
            handle.take()
        }

        pub fn command(self, name: impl Into<Cow<'static, str>>) -> Bp<Command<T>> {
            Bp(Command {
                names: vec![Name::Long(name.into())],
                inner: self.0,
            })
        }
    }

    impl<T: 'static> Parser<T> for Bp<Command<T>> {
        async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
            todo!();
        }
    }

    pub struct Many<T> {
        pub(crate) inner: RcParser<T>,
    }

    impl<T: 'static> Parser<Vec<T>> for Bp<Many<T>> {
        async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
            let mut res = Vec::new();
            let start = ctx.next_free.get();
            let mut consumed_before = 0;
            while matches!(&*ctx.wakeup_reason.borrow(), Reason::Pass) {
                ctx.next_free.set(start);
                let (h, pair) = ctx.spawn_with_early_exit(self.0.inner.clone());

                r#yield().await;
                ctx.remove_early_exit(pair);

                let consumed_after = ctx.current_task.borrow().consumed;
                let advanced = consumed_after > consumed_before;
                consumed_before = consumed_after;

                let val = h.take();
                println!("Parsed something, ok? {:?}", val.is_ok());

                match (advanced, val) {
                    (true, Ok(v)) => res.push(v),
                    (true, Err(e)) => return Err(e),
                    (false, Ok(v)) => {
                        if res.is_empty() {
                            res.push(v);
                        }
                        break;
                    }
                    (false, Err(e)) if e.can_catch() => break,
                    (false, Err(e)) => return Err(e),
                }
            }
            Ok(res)
        }
    }

    pub struct Command<T> {
        names: Vec<Name<'static>>,
        inner: OptionParser<T>,
    }
}

mod traits {
    //! [`Parser`] trait and related private helper traits

    use crate::{Bp, Ctx, Error};
    use std::{marker::PhantomData, pin::Pin, rc::Rc};

    use crate::adapters::*;
    pub trait Parser<T: 'static> {
        fn run(&self, ctx: Ctx) -> impl Future<Output = Result<T, Error>>;

        /// Convert the parser into a boxed, reference counted version
        fn into_rc(self) -> Bp<RcParser<T>>
        where
            Self: Sized + Parser<T> + 'static,
        {
            Bp(RcParser(Rc::new(self)))
        }

        fn map<F, R>(self, map: F) -> impl Parser<R>
        where
            Self: Sized,
            F: Fn(T) -> R + 'static,
            R: 'static,
        {
            Map {
                inner: self,
                ctx: PhantomData,
                map,
            }
        }

        fn optional(self) -> impl Parser<Option<T>>
        where
            Self: Sized + 'static,
        {
            Optional {
                inner: self.into_rc().0,
            }
        }

        fn many(self) -> Bp<Many<T>>
        where
            Self: Sized + 'static,
        {
            Bp(Many {
                inner: self.into_rc().0,
            })
        }

        fn to_options(self) -> Bp<OptionParser<T>>
        where
            Self: Sized + 'static,
        {
            Bp(OptionParser {
                inner: self.into_rc().0,
            })
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
    #[repr(transparent)]
    pub struct RcParser<T>(Rc<dyn DynParser<T>>);

    impl<T> Clone for RcParser<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
}

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
    KillScope { scope: Scope },
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
        ctx.pending_ops.borrow_mut().push(Op::RegisterSum {
            id,
            scope: Scope {
                start: id,
                end: scopes.last().unwrap().end,
            },
        });
        r#yield().await;
        ctx.trim_children(scopes).await;

        // // TODO for what
        // let task = {
        //     let ctx = ctx.clone();
        //     let info = ctx.make_child_info(Kind::Prod);
        //     let act = Box::pin(async move {
        //         ctx.trim_children(scopes).await;
        //     });
        //     Task { act, info }
        // };
        //
        // ctx.add_task(task);
        // r#yield().await;

        #[expect(clippy::manual_try_fold)] // this is not a try_fold
        handles
            .into_iter()
            .fold(Err(Error::Killed), |res, h| match (h.take(), res) {
                (v @ Ok(_), _) | (Err(_), v @ Ok(_)) => v,
                (Err(e1), Err(e2)) => Err(e1 + e2),
            })
    }
}

#[derive(Debug, Default)]
struct Scopes {
    items: BTreeMap<Scope, Id>,
}

impl Scopes {
    fn insert(&mut self, key: Scope, val: Id) {
        self.items.insert(key, val);
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
#[derive(Debug, Copy, Clone)]
pub struct Metavar(&'static str);
mod error {
    use std::borrow::Cow;

    use crate::{Metavar, Name};

    #[derive(Debug)]
    pub enum Error {
        MissingItem(Missing),
        ParseError,
        Killed,
        Internal,
    }

    #[derive(Debug, Clone)]
    pub enum Missing {
        Single(MissingItem),
        Items(Vec<MissingItem>),
    }
    impl Missing {
        fn combine(self, m: Missing) -> Self {
            match (self, m) {
                (Missing::Single(m1), Missing::Single(m2)) => Self::Items(vec![m1, m2]),
                (Missing::Single(m), Missing::Items(mut ms))
                | (Missing::Items(mut ms), Missing::Single(m)) => {
                    ms.push(m);
                    Self::Items(ms)
                }
                (Missing::Items(mut ms), Missing::Items(mut ms2)) => {
                    ms.append(&mut ms2);
                    Self::Items(ms)
                }
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum MissingItem {
        Named {
            name: Name<'static>,
            meta: Option<Metavar>,
        },
        Pos {
            meta: Metavar,
        },
        Lit {
            value: Cow<'static, str>,
        },
        Command,
    }

    impl Error {
        pub(crate) fn combine(self, e2: Error) -> Error {
            match (self, e2) {
                (Error::Internal, _e) | (_e, Error::Internal) => Error::Internal,
                (Error::Killed, e) | (e, Error::Killed) => e,
                (Error::MissingItem(i1), Error::MissingItem(i2)) => {
                    Error::MissingItem(i1.combine(i2))
                }
                (Error::MissingItem(_), Error::ParseError) => todo!(),
                (Error::ParseError, Error::MissingItem(_)) => todo!(),
                (Error::ParseError, Error::ParseError) => todo!(),
            }
        }

        pub(crate) fn can_catch(&self) -> bool {
            match self {
                Error::MissingItem(_) => true,
                Error::ParseError => false,
                Error::Killed => true,
                Error::Internal => false,
            }
        }
        pub(crate) fn missing(item: MissingItem) -> Self {
            Self::MissingItem(Missing::Single(item))
        }

        /// Consume current error and append it to a growing collection in dst
        ///
        /// It exists to collect errors from multiple handles and designed to work with
        /// [`Result::map_err`]. We aggregate the best possible error inside an Option
        /// and fail with that if it is present
        pub(crate) fn append_to(self, dst: &mut Option<Error>) -> Self {
            *dst = Some(match dst.take() {
                Some(e) => e.combine(self),
                None => self,
            });
            Error::Killed
        }
    }
}
pub use error::*;

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
    /// Scheduled ops
    ///
    /// Active tasks are living outside of the `RawCtx`: we need to borrow them to poll
    /// anyway but also need to borrow to spawn new tasks from polled tasks. Having them
    /// outside makes the code a bit cleaner
    ///
    /// Holds other operations that might need access to tasks or other internal structures
    pending_ops: RefCell<Vec<Op>>,
    triggers: RefCell<Triggers>,
    /// Early exit ranges
    ///
    /// When there's no matching triggers executor will try to terminate anything inside of
    /// the each pair. This allows is necessary for things like `.optional()` and `.many()` to work
    early_exit: RefCell<BTreeSet<Scope>>,
    next_free: Cell<u32>,
    args: RefCell<Vec<OsString>>,
    cursor: Cell<usize>,
    wakeup_reason: RefCell<Reason>,
    current_task: RefCell<TaskInfo>,
}
pub type Ctx = Rc<RawCtx>;

#[derive(Debug)]
enum Reason {
    NotConsumedEnough,
    Pass,
    Arg(Option<Arg<'static>>),
    ChildProgress(Vec1<Id>),
}

/// A small vector that can hold 1 item without heap allocation
#[derive(Debug, Clone)]
enum Vec1<T> {
    Small(Option<T>),
    Big(Vec<T>),
}

impl<T> Default for Vec1<T> {
    fn default() -> Self {
        Vec1::Small(None)
    }
}

impl<T: Copy> Vec1<T> {
    fn push(&mut self, val: T) {
        match self {
            Vec1::Small(None) => *self = Vec1::Small(Some(val)),
            Vec1::Small(Some(prev)) => *self = Vec1::Big(vec![*prev, val]),
            Vec1::Big(items) => items.push(val),
        }
    }
    fn as_slice(&self) -> &[T] {
        match self {
            Vec1::Small(x) => x.as_slice(),
            Vec1::Big(items) => items.as_slice(),
        }
    }
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

struct DummyArg(char);
impl Parser<Option<OsString>> for DummyArg {
    async fn run(&self, ctx: Ctx) -> Result<Option<OsString>, Error> {
        let names = [Name::Short(self.0)];
        let r = ctx.parse_arg(&names[..]).await;
        println!("{r:?} / {names:?} {:?}", ctx.current_task.borrow().id);
        r
    }
}

struct DummyPos;
impl Parser<Option<OsString>> for DummyPos {
    async fn run(&self, ctx: Ctx) -> Result<Option<OsString>, Error> {
        let r = ctx.parse_pos().await;
        println!("{r:?} / {:?}", ctx.current_task.borrow().id);
        r
    }
}

struct DummyAnyOs<T>(Rc<dyn Fn(&OsStr) -> Option<T>>);
struct DummyAny<T>(Rc<dyn Fn(&str) -> Option<T>>);

fn any<T: 'static>(check: impl Fn(&str) -> Option<T> + 'static) -> impl Parser<T> {
    DummyAny(Rc::new(check))
}

impl<T: 'static> Parser<T> for DummyAny<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let parser = self.0.clone();
        let check = Box::new(move |os: &OsStr| -> Option<Box<dyn std::any::Any>> {
            Some(Box::new(parser(os.to_str()?)?))
        });
        Ok(*ctx
            .parse_any(check)
            .await?
            .unwrap()
            .downcast()
            .expect("It should match"))
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

    /// Convert a parser into a task that saves its output to a [`JoinHandle`]
    //
    // TODO - Do I really need Bp wrapper here?
    fn make_raw_task<T: 'static>(
        self: &Rc<Self>,
        parser: Bp<RcParser<T>>,
    ) -> (JoinHandle<T>, Pin<Box<impl Future<Output = ()> + 'static>>) {
        let ctx = self.clone();
        let result_out = Rc::new(Cell::new(None));
        let result = result_out.clone();
        let action = Box::pin(async move { result_out.set(Some(parser.0.run(ctx).await)) });
        (JoinHandle { result }, action)
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
            self.pending_ops.borrow_mut().push(Op::Spawn(task));
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

    async fn trim_children(&self, mut scopes: Vec<Scope>) {
        loop {
            println!("Trim children, got {:?} / {scopes:?}", self.wakeup_reason);
            match *self.wakeup_reason.borrow() {
                Reason::ChildProgress(ref ids) => {
                    scopes.retain(|scope| {
                        let lives = ids.as_slice().iter().any(|id| scope.contains(*id));
                        if !lives {
                            let op = Op::KillScope { scope: *scope };
                            self.pending_ops.borrow_mut().push(op);
                        }
                        lives
                    });
                    if scopes.len() == 1 {
                        self.pending_ops.borrow_mut().push(Op::DeregisterSum {
                            id: self.current_task.borrow().id,
                        });
                    }
                }
                Reason::Pass => {
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
        match self.execute_group_inner(&group.0) {
            Ok(()) => {
                self.ctx.cursor.update(|c| c + 1);
                Ok(())
            }
            Err(_) => todo!("Can't parse {group:?}"),
        }
    }

    fn execute_group_inner(&mut self, names: &[char]) -> Result<(), Error> {
        let mut mixer_capacity = Mixer::default();
        for name in names {
            // set a wakeup reason, just in case
            *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(Some(Arg::Named {
                name: (*name).into(),
                value: None,
            }));

            let mut mixer = mixer_capacity.reuse_capacity();
            let triggers = self.ctx.triggers.borrow();
            mixer.populate_short_flag(name, &triggers);
            while let Some(next) = mixer.consume_next_item(&self.tasks) {
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
            mixer_capacity = mixer.reuse_capacity();
            drop(triggers);
            self.stage_2(1);
            self.propagate(Reason::Pass);
            self.process_scheduled();
        }

        Ok(())
    }

    /// Process scheduled operations
    fn process_scheduled(&mut self) {
        for op in self.ctx.pending_ops.borrow_mut().drain(..) {
            match op {
                Op::Spawn(task) => {
                    let old = self.tasks.insert(task.info.id, task);
                    debug_assert!(old.is_none(), "Duplicated task id?");
                }
                Op::RegisterSum { id, scope } => {
                    self.sums.insert(id, scope);
                }
                Op::KillScope { scope } => {
                    // self.kill_in_scope(scope); // TODO <- want this!
                    *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(None);
                    for (id, mut task) in self
                        .tasks
                        .extract_if(scope.start..scope.end, |_, task| task.info.pending == 0)
                    {
                        let r = self.ctx.poll_in_context(&mut task);
                        assert_eq!(task.info.consumed, 0);
                        self.to_propagate
                            .push_back((id, task.info.parent_id, task.info.consumed));
                        assert!(r);
                    }
                }
                Op::DeregisterSum { id } => {
                    self.sums.remove(&id);
                }
            }
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
            self.process_scheduled();

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
            let (best_size, mixer, mgroup) = self.stage_1(front, mixer_capacity);
            mixer_capacity = mixer;

            if let Some(group) = mgroup
                && best_size == 0
            {
                self.execute_group(group)?;
                self.propagate(Reason::Pass);
                continue;
            }

            if best_size == 0 {
                let maybe_scope = self.ctx.early_exit.borrow_mut().pop_last();
                if let Some(scope) = maybe_scope {
                    self.kill_in_scope(scope);
                    continue;
                }
            }

            if best_size > 0 {
                self.ctx.cursor.update(|c| c + best_size as usize);
                changed = true;
            } else {
                todo!("Didn't consume anything, complain about {front:?} and prepare to exit?");
            }
            self.cleanup_siblings(best_size);
            self.stage_2(best_size);

            self.propagate(Reason::Pass);
        }

        println!("There's nothing to do");
        for t in self.tasks.values() {
            println!("{t:?}");
        }
        println!();

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

    fn kill_in_scope(&mut self, scope: Scope) {
        *self.ctx.wakeup_reason.borrow_mut() = Reason::Arg(None);
        for (_, mut task) in self
            .tasks
            .extract_if(scope.start..scope.end, |_, t| t.info.pending == 0)
        {
            let done = self.ctx.poll_in_context(&mut task);
            assert!(done);
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

    fn cleanup_siblings(&mut self, best_size: u32) {
        for id in self.to_wake.iter() {}
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
                Reason::NotConsumedEnough
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
                    // println!("Waking up {sid:?}, tasks we have {:?}", self.tasks);
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
        println!("{advancing:?}\n",);

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
            println!("After {parent:?} {consumed:?} task is niow {task:?}");
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
    fn new(args: Vec<OsString>) -> Rc<RawCtx> {
        Rc::new(RawCtx {
            args: args.into(),
            current_task: Default::default(),
            cursor: Cell::new(0),
            early_exit: Default::default(),
            pending_ops: Default::default(),
            next_free: Cell::new(1),
            triggers: Default::default(),
            wakeup_reason: RefCell::new(Reason::Pass),
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
#[must_use]
#[derive(Debug)]
struct Group(Vec<char>);

impl<'a> Mixer<'a> {
    fn populate_short_flag(&mut self, name: &char, triggers: &'a Triggers) {
        self.pecking_push(triggers.short_flags.get(name));
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
            Arg::Pos { value: _ } => {
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
pub(crate) enum Name<'a> {
    Short(char),
    Long(Cow<'a, str>),
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

pub fn run<T: 'static>(parser: impl Parser<T> + 'static, args: &[&str]) -> Result<T, Error> {
    let args = Args::from(args).items;
    let ctx = RawCtx::new(args);

    let (handle, act) = ctx.make_raw_task(parser.into_rc());
    let info = ctx.make_child_info(Kind::Prod);
    let task = Task { act, info };
    ctx.add_task(task);
    ctx.execute()?;
    handle.take()
}

#[test]
fn parse_simple_flag_works_0() {
    let a = DummyFlag('a');

    let r = run(a, &["-a"]).unwrap();
    assert!(r);
}

#[test]
fn parse_simple_flag_works_1() {
    let a = short('a').switch();
    let b = short('b').flag(1, 2);
    let c = short('c').req_flag(());
    let parser = construct!(a, b, c);
    let r = run(parser, &["-a", "-b", "-c"]).unwrap();
    assert_eq!(r, (true, 1, ()));
}

#[test]
fn parse_simple_flag_works_2() {
    let a = DummyFlag('a');
    let b = DummyFlag('a');
    let parser = construct!(a, b);
    let r = run(parser, &["-a", "-a"]).unwrap();
    assert_eq!(r, (true, true));
}

#[test]
fn optional_tuple_works() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!(a, b).optional();
    let r = run(parser, &["-a", "-b"]).unwrap();
    assert_eq!(r, Some(('a', 'b')));
}

#[test]
fn parse_simpel_arg_works_1() {
    let a = DummyArg('a');
    let b = DummyArg('b');
    let parser = construct!(a, b);
    let (ra, rb) = run(parser, &["-a=10", "-b", "20"]).unwrap();
    assert_eq!(ra.unwrap(), "10");
    assert_eq!(rb.unwrap(), "20");
}

#[test]
fn simple_parse_any_works() {
    let parser = any(|s| s.parse::<i32>().ok()).to_options();

    let r = parser.run_inner("-42").unwrap();
    assert_eq!(r, -42);
    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn with_flag_parse_any_works() {
    let a = any(|s| s.parse::<i32>().ok());
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-b -10").unwrap();
    assert_eq!(r, (-10, true));
    let r = parser.run_inner("-10 -b").unwrap();
    assert_eq!(r, (-10, true));
}

#[test]
fn parse_optional_temp() {
    let a = short('a').argument::<usize>("ARG").optional();
    let b = short('b').argument::<usize>("ARG2");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-b10 -a 12").unwrap();
    assert_eq!(r, (Some(12), 10));

    let r = parser.run_inner("-b4").unwrap();
    assert_eq!(r, (None, 4));
}

#[test]
fn many_works() {
    let parser = short('a').req_flag(()).many().to_options();
    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[(), (), ()]);
}

#[test]
fn flag_group_works_reqflag() {
    let parser = short('a').req_flag(()).many().to_options();
    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[(), (), ()]);
}

#[test]
fn flag_group_works_switch() {
    let parser = short('a').switch().many().to_options();

    let r = parser.run_inner("-aaa").unwrap();
    assert_eq!(r, &[true, true, true]);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, &[false]);

    let r = parser.run_inner("-a -a -a").unwrap();
    assert_eq!(r, &[true, true, true]);
}

#[test]
fn many_with_optional() {
    let a = short('a').req_flag(()).optional();
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b).many().to_options();

    let r = parser.run_inner("-b30").unwrap();
    assert_eq!(r, &[(None, 30)]);

    let r = parser.run_inner("-a -b=10 -b20 -a -b 30").unwrap();

    assert_eq!(r, &[(Some(()), 10), (Some(()), 20), (None, 30)]);
}

#[test]
fn simple_alt_with_one_flag() {
    let a = short('a').req_flag('a');
    let a1 = short('A').switch();
    let a = construct!(a, a1);
    let b = short('b').req_flag('b');
    let b1 = short('B').switch();
    let b = construct!(b, b1);
    let parser = construct!([a, b]).to_options();
    let r = parser.run_inner("-a -A").unwrap();
    assert_eq!(r, ('a', true));
}

#[test]
fn simple_alt_with_flags() {
    let a = short('a').req_flag('a');
    let a1 = short('A').switch();
    let a2 = short('A').switch();
    let a3 = short('A').switch();
    let a4 = short('A').switch();
    let a = construct!(a, a1, a2, a3, a4);
    let b = short('b').req_flag('b');
    let b1 = short('B').switch();
    let b2 = short('B').switch();
    let b3 = short('B').switch();
    let b4 = short('B').switch();
    let b = construct!(b, b1, b2, b3, b4);
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-a -AAAA").unwrap();
    assert_eq!(r, ('a', true, true, true, true));
}

#[test]
fn nested_alt_works() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let d = short('d').req_flag('d');
    let ab = construct!([a, b]);
    let cd = construct!([c, d]);
    let parser = construct!([ab, cd]).to_options();

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn bare_parser() {
    let parser = short('b').req_flag('b').to_options();
    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn very_very_simple_alt() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).to_options();
    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');
}

#[test]
fn simple_alt_with_option() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).optional().to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, None);

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, Some('b'));
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Some('a'));
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
        ps: [Bp<RcParser<T>>; 4],
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
