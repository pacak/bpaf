mod ctx;
mod error;
mod executor;
mod mini_ansi;
pub mod parsers;
mod pecking;
mod split;
mod visitor;
/// Reexports used by the [`construct!`] macro
#[doc(hidden)]
pub mod __private {
    pub use crate::{
        ctx::Ctx,
        error::Error,
        executor::{downcast, hint, Alt, Con, Fragment},
        Metavisit, Parser,
    };
}

use error::ParseFailure;

use crate::{
    ctx::Ctx,
    error::Metavar,
    executor::Fragment,
    named::{Argument, Flag, Name, Named},
    parsers::{
        Anything, Command, Count, Fallback, FallbackWith, GroupHelp, Guard, HideUsage, Last, Many,
        Map, Optional, Parse, Pure,
    },
    positional::Positional,
    split::{Args, OsOrStr},
    visitor::Visitor,
};
use std::{borrow::Cow, marker::PhantomData, ops::RangeBounds, rc::Rc};

#[macro_export]
macro_rules! construct {
    // === capture initial shape of the query

    // construct!(Enum::Cons { a, b, c })
    ($(::)? $ns:ident $(:: $con:ident)* { $($tokens:tt)* }) =>
        {{ $crate::construct!(@prepare [named [$ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!(Enum::Cons ( a, b, c ))
    ($(::)? $ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) =>
        {{ $crate::construct!(@prepare [pos [$ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!( a, b, c )
    ($first:ident $($tokens:tt)*) => // first to make sure we have at least one item
        {{ $crate::construct!(@prepare [pos] [] $first $($tokens)*) }};

    // construct!([a, b, c])
    ([$first:ident $($tokens:tt)*]) => // first - to make sure we have at lest one item
        {{ $crate::construct!(@prepare [alt] [] $first $($tokens)*) }};

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
        $crate::__private::Alt { items: ::std::vec![ $($field.into_box()),*] }
    };

    // For product type the logic is a bit more complicated - do one more step
    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::construct!(@fin $ty [ $($fields)* ])
    };

    // === Making a body for the product parser

    // Two special cases where we construct something with no fields, use `Parser::pure` for that
    (@fin [named [$($con:tt)+]] []) => { $crate::pure($($con)+ { })};
    (@fin [pos   [$($con:tt)+]] []) => { $crate::pure($($con)+ ( ))};

    (@fin $ty:tt [$($fields:ident)*]) => {
                #[allow(unused_assignments)]
        {
        let mut visitors = ::std::vec::Vec::<::std::boxed::Box<dyn $crate::__private::Metavisit>>::new();
        let mut parsers = ::std::vec::Vec::<::std::boxed::Box<dyn ::std::any::Any>>::new();
        $(
            let $fields: ::std::rc::Rc<dyn $crate::__private::Parser<_>> = $fields.into_rc();
            visitors.push(::std::boxed::Box::new($fields.clone()));
            parsers.push(::std::boxed::Box::new($fields.clone()));
            let $fields = $crate::__private::hint($fields);
        )*

        // making a future assumes parser is borrowed with the same lifetime as the
        // context. This helps to avoid a whole lot of boxing.
        // Problem is that here parsers are owned, so we must store them inside Con.
        //
        // There's several parsers and type aligned sequences are not a thing so
        // each parser is casted first into a parser trait object then into Any trait object
        // and passed along with the context.
        //
        // Later Any::downcast_ref helps to recover parser trait objects to run and create the
        // future
        //
        // Next problem is that downcast_ref needs to know the type to recover, we do this
        // by getting a type hint PhantomData<T> from Rc<dyn Parser<T>>, passing the hint
        // (PhantomData is Copy!) into the `run` and use it to downcast to the precise type
        //
        // If only `call` on `Fn` had a reference on `&self` lifetime in the output...
        // `fn call(&self, args: Args) -> Self::Output`
        let run: ::std::boxed::Box<dyn for<'a>
            ::std::ops::Fn(&'a [::std::boxed::Box<dyn ::std::any::Any>], $crate::__private::Ctx<'a>) ->
                $crate::__private::Fragment<'a, _>> =
            ::std::boxed::Box::new(move |parsers, ctx| {
            let mut n = 0;

            $(
                let $fields = $crate::__private::downcast($fields, &parsers[n]);
                n += 1;
            )*
            ::std::boxed::Box::pin(async move {
                let (_branch, id) = ctx.current_id();
                let mut n = 0;
                $(
                    let $fields = ctx.spawn(id.prod(n), $fields, false);
                    n += 1;
                )*
                ctx.early_exit(n).await?;
                ::std::result::Result::Ok::<_, $crate::__private::Error>
                    ($crate::construct!(@make $ty [$($fields)*]))
            })
        });
        $crate::__private::Con { visitors, parsers, run }

    }};

    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields.await?),* } };
    // for positional - ()
    (@make [pos   [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ ( $($fields.await?),* ) };
    // And this handles the case where there's no constructor and we are makig a tuple
    (@make [pos                ] [$($fields:ident)*]) => { ( $($fields.await?),* ) };
}

/// Parser with extra typestate information
///
/// **bpaf** exposes all of it's functionality using
/// [Fluent interface](https://en.wikipedia.org/wiki/Fluent_interface?useskin=vector), this
/// datatype collects all possible intermediate representations along with all the
/// methods they support in the same place.
///
/// In additon to inherent methods, most of the `Cx` states implement [`Parser`] trait with more
/// operations.
///
/// While this documentation explains *fluent interface* specifically, most of it applies
// TODO - add link
/// to the derive API as well, the main difference is that instead of methods being chained
/// on the parser - you list them inside derive annotations. In both of those examples `many` refers to
/// [`Parser::many`].
///
/// Fluent API:
/// ```ignore
/// ...
/// let field = some_parser().many();
/// ...
/// ```
///
/// Derive API:
/// ```ignore
/// ...
/// #[bpaf(external(some_parser), many)]
/// field: Vec<String>
/// ...
/// ```
///
///
/// Some of the notable states
///
/// - [Named items](#named-items)
#[must_use = "Bpaf parsers do nothing unless executed"]
pub struct Cx<I>(I);

/// Make named item with short name
pub fn short(name: char) -> Cx<Named> {
    Cx(named::short(name))
}

pub fn long(name: &'static str) -> Cx<Named> {
    Cx(named::long(name))
}

pub fn positional<T>(meta: &'static str) -> Cx<Positional<T>> {
    Cx(positional::positional(Metavar(meta)))
}

/// # Named items
///
/// Parses something with name attached
impl Cx<Named> {
    pub fn short(mut self, name: char) -> Self {
        self.0.short(name);
        self
    }
    pub fn long(mut self, name: &'static str) -> Self {
        self.0.long(name);
        self
    }
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help(help);
        self
    }
    pub fn switch(self) -> Cx<Flag<bool>> {
        Cx(self.0.flag(true, Some(false)))
    }
    pub fn flag<T>(self, present: T, absent: T) -> Cx<Flag<T>> {
        Cx(self.0.flag(present, Some(absent)))
    }
    pub fn req_flag<T>(self, present: T) -> Cx<Flag<T>> {
        Cx(self.0.flag(present, None))
    }
    pub fn argument<T>(self, meta: &'static str) -> Cx<Argument<T>> {
        Cx(self.0.argument(meta))
    }
}

/// # Parser for a named item
impl<T> Cx<Flag<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help(help);
        self
    }
    // adjacent_to
}

impl<T> Cx<Argument<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help(help);
        self
    }
}

/// # Positional items
///
/// Parses a positional item, usually with something attached
impl<T> Cx<Positional<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help(help);
        self
    }
}

mod named {
    use std::{borrow::Cow, marker::PhantomData, str::FromStr};

    use crate::{
        ctx::Ctx,
        error::{Error, Metavar},
        executor::{
            futures::{ArgFut, FlagFut},
            Fragment,
        },
        visitor::Item,
        Parser,
    };

    pub struct Argument<T> {
        named: Named,
        ty: PhantomData<T>,
        meta: Metavar,
    }

    pub struct Flag<T> {
        named: Named,
        present: T,
        absent: Option<T>,
    }

    impl<T> Parser<T> for Argument<T>
    where
        T: 'static + FromStr,
        <T as FromStr>::Err: std::error::Error,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
            Box::pin(async move {
                ArgFut {
                    name: &self.named.names,
                    meta: self.meta,
                    ctx,
                    task_id: None,
                }
                .await?
                .parse::<T>()
            })
        }

        fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
            visitor.item(Item::Arg {
                names: &self.named.names,
                meta: self.meta,
                help: self.named.help,
            })
        }
    }

    impl<T> Parser<T> for Flag<T>
    where
        T: Clone + 'static,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
            Box::pin(async move {
                let fut = FlagFut {
                    name: &self.named.names,
                    ctx,
                    task_id: None,
                };
                match fut.await {
                    Ok(_) => Ok(self.present.clone()),
                    Err(e) if e.handle_with_fallback() => match self.absent.as_ref().cloned() {
                        Some(v) => Ok(v),
                        None => Err(e),
                    },
                    Err(e) => Err(e),
                }
            })
        }

        fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
            visitor.item(Item::Flag {
                names: &self.named.names,
                help: self.named.help,
            })
        }
    }

    impl Named {
        pub(crate) fn flag<T>(self, present: T, absent: Option<T>) -> Flag<T> {
            Flag {
                named: self,
                present,
                absent,
            }
        }
    }

    impl std::fmt::Display for Name<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if f.alternate() {
                match self {
                    Name::Short(s) => write!(f, "{s}"),
                    Name::Long(l) => write!(f, "{l}"),
                }
            } else {
                match self {
                    Name::Short(s) => write!(f, "-{s}"),
                    Name::Long(l) => write!(f, "--{l}"),
                }
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub enum Name<'a> {
        Short(char),
        Long(Cow<'a, str>),
    }

    impl Name<'_> {
        /// Convert possibly borrowed name into an owned one
        pub(crate) fn to_owned(&self) -> Name<'static> {
            match self {
                Self::Short(s) => Name::Short(*s),
                Self::Long(cow) => Name::Long(Cow::Owned(cow.to_string())),
            }
        }
        pub(crate) fn as_ref(&self) -> Name {
            match self {
                Name::Short(s) => Name::Short(*s),
                Name::Long(cow) => Name::Long(Cow::Borrowed(cow.as_ref())),
            }
        }
        pub(crate) fn long<'a>(name: &'a str) -> Name<'a> {
            Name::Long(Cow::Borrowed(name))
        }
    }

    pub struct Named {
        names: Vec<Name<'static>>,
        help: Option<&'static str>,
    }

    pub(crate) fn short(name: char) -> Named {
        Named {
            names: vec![Name::Short(name)],
            help: None,
        }
    }
    pub(crate) fn long(name: &'static str) -> Named {
        Named {
            names: vec![Name::Long(Cow::Borrowed(name))],
            help: None,
        }
    }
    impl Named {
        pub(crate) fn short(&mut self, name: char) {
            self.names.push(Name::Short(name));
        }
        pub(crate) fn long(&mut self, name: &'static str) {
            self.names.push(Name::Long(Cow::Borrowed(name)));
        }
        pub(crate) fn help(&mut self, help: &'static str) {
            self.help = Some(help);
        }
        pub(crate) fn argument<T>(self, meta: &'static str) -> Argument<T> {
            Argument {
                named: self,
                ty: PhantomData,
                meta: Metavar(meta),
            }
        }
    }
    impl<T> Flag<T> {
        pub(crate) fn help(&mut self, help: &'static str) {
            self.named.help = Some(help);
        }
    }

    impl<T> Argument<T> {
        pub(crate) fn help(&mut self, help: &'static str) {
            self.named.help = Some(help);
        }
    }
}

mod positional {
    use std::{marker::PhantomData, str::FromStr};

    use crate::{
        ctx::Ctx,
        error::{Error, Message, Metavar},
        executor::futures::PositionalFut,
        visitor::Item,
        Parser,
    };
    pub struct Positional<T> {
        meta: Metavar,
        help: Option<&'static str>,
        ty: PhantomData<T>,
    }
    pub(crate) fn positional<T>(meta: Metavar) -> Positional<T> {
        Positional {
            meta,
            help: None,
            ty: PhantomData,
        }
    }
    impl<T> Positional<T> {
        pub(crate) fn help(&mut self, help: &'static str) {
            self.help = Some(help);
        }
    }

    impl<T> Parser<T> for Positional<T>
    where
        T: 'static + FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> crate::executor::Fragment<'a, T> {
            Box::pin(async {
                PositionalFut {
                    ctx,
                    task_id: None,
                    meta: self.meta,
                }
                .await?
                .parse::<T>()
            })
        }

        fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
            visitor.item(Item::Positional {
                meta: self.meta,
                help: self.help,
            })
        }
    }
}

pub trait Parser<T: 'static> {
    #[doc(hidden)]
    /// Create a fragment that executor can work with
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T>;

    #[doc(hidden)]
    /// Gather information
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);

    /// Convert parser into Boxed trait object
    fn into_box(self) -> Box<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    /// Convert parser into Rc trait object
    fn into_rc(self) -> Rc<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }

    /// many/some/collect/take/at_least/in_range
    fn many<C>(self) -> crate::Cx<Many<Self, C, T>>
    where
        Self: Sized,
    {
        crate::Cx(Many {
            inner: self,
            error: "",
            at_least: 0,
            at_most: u32::MAX,
            ty: PhantomData,
        })
    }

    fn some<C>(self, error: &'static str) -> crate::Cx<Many<Self, C, T>>
    where
        Self: Sized,
    {
        crate::Cx(Many {
            inner: self,
            error,
            at_least: 1,
            at_most: u32::MAX,
            ty: PhantomData,
        })
    }

    fn take<C>(self, at_most: u32) -> crate::Cx<Many<Self, C, T>>
    where
        Self: Sized,
    {
        crate::Cx(Many {
            inner: self,
            error: "",
            at_least: 0,
            at_most,
            ty: PhantomData,
        })
    }

    fn at_least<C>(self, at_least: u32, error: &'static str) -> Cx<Many<Self, C, T>>
    where
        Self: Sized,
    {
        Cx(Many {
            inner: self,
            error,
            at_least,
            at_most: u32::MAX,
            ty: PhantomData,
        })
    }

    fn in_range<C>(self, range: impl RangeBounds<u32>, error: &'static str) -> Cx<Many<Self, C, T>>
    where
        Self: Sized,
    {
        Cx(Many {
            inner: self,
            error,
            at_least: match range.start_bound() {
                std::ops::Bound::Included(x) => *x,
                std::ops::Bound::Excluded(x) => *x + 1,
                std::ops::Bound::Unbounded => 0,
            },
            at_most: match range.end_bound() {
                std::ops::Bound::Included(m) => m.saturating_add(1),
                std::ops::Bound::Excluded(m) => *m,
                std::ops::Bound::Unbounded => u32::MAX,
            },
            ty: PhantomData,
        })
    }

    fn count(self) -> Count<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        Count {
            inner: self,
            ctx: PhantomData,
        }
    }

    fn last(self) -> Last<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        Last {
            inner: self,
            ctx: PhantomData,
        }
    }

    fn map<F, R>(self, map: F) -> Map<Self, F, T, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> R + 'static,
    {
        Map {
            inner: self,
            ctx: PhantomData,
            map,
        }
    }

    /// Apply a failing transformation to a contained value
    fn parse<F, R, E>(self, f: F) -> Parse<Self, F, T, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> Result<R, E>,
        E: ToString,
    {
        Parse {
            inner: self,
            f,
            ctx: PhantomData,
        }
    }

    fn guard<F>(self, check: F, message: &'static str) -> Guard<Self, F>
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        Guard {
            inner: self,
            check,
            message,
        }
    }

    fn optional(self) -> Cx<Optional<Self>>
    where
        Self: Sized,
    {
        Cx(Optional { inner: self })
    }

    fn fallback(self, value: T) -> Cx<Fallback<Self, T>>
    where
        Self: Sized,
    {
        Cx(Fallback { inner: self, value })
    }

    fn fallback_with<F, E>(self, f: F) -> Cx<FallbackWith<Self, T, F, E>>
    where
        Self: Sized,
        F: Fn() -> Result<T, E>,
        E: ToString,
    {
        Cx(FallbackWith {
            inner: self,
            f,
            ctx: PhantomData,
        })
    }

    fn to_options(self) -> Cx<Options<T>>
    where
        Self: Sized + 'static,
    {
        Cx(Options {
            parser: self.into_box(),
            fallback_to_usage: false,
        })
    }

    fn group_help(self, title: &'static str) -> Cx<GroupHelp<Self>>
    where
        Self: Sized,
    {
        Cx(GroupHelp { inner: self, title })
    }

    fn hide_usage(self) -> Cx<HideUsage<Self>>
    where
        Self: Sized,
    {
        Cx(HideUsage { inner: self })
    }

    // hide
    // hide_usage
    // with_usage
    // group_help
    // with_group_help
    // complete
    // complete_shell
    //
    // "render items inside as a custom help section"
}

pub fn pure<T>(value: T) -> Cx<Pure<T>> {
    Cx(Pure { value })
}

pub struct Options<T> {
    parser: Box<dyn Parser<T>>,
    fallback_to_usage: bool,
}

pub type OptionParser<T> = Cx<Options<T>>;

impl<T: 'static> Cx<Options<T>> {
    pub fn run(&self) -> T {
        self.try_run().unwrap() // TODO
    }

    pub fn try_run(&self) -> Result<T, ParseFailure> {
        self.run_inner(std::env::args_os())
    }

    pub fn run_inner<'a>(&'a self, args: impl Into<Args<'a>>) -> Result<T, ParseFailure> {
        let args = args.into();

        let help = crate::parsers::help_and_version();
        let ctx = Ctx::new(args.as_ref(), args.ctx_start, &help);
        self.0.execute(ctx)
    }

    pub fn command(self, name: &'static str) -> Cx<Command<T>> {
        Cx(Command {
            names: vec![Name::Long(Cow::Borrowed(name))],
            parser: self,
            adjacent: false,
        })
    }
    pub fn fallback_to_usage(mut self) -> Self {
        self.0.fallback_to_usage = true;
        self
    }
}

impl<T: 'static> Options<T> {
    fn execute<'a>(&'a self, ctx: Ctx<'a>) -> Result<T, ParseFailure> {
        use crate::parsers::HelpWrap;

        let help_p = ctx.help_and_version;
        match executor::Runner::new(ctx.clone()).run_parser(&self.parser, true) {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let help = || {
                    let mut help = visitor::help::Help::default();
                    self.parser.visit(&mut help);
                    ctx.help_and_version.visit(&mut help);
                    Err(ParseFailure::Stdout(help.render("")))
                };
                if let Ok(ans) = executor::Runner::new(ctx.clone()).run_parser(help_p, false) {
                    match ans {
                        HelpWrap::Version => todo!(),
                        HelpWrap::Help => help(),
                        HelpWrap::DetailedHelp => todo!(),
                    }
                } else if ctx.current_ctx().is_empty() && self.fallback_to_usage {
                    help()
                } else {
                    Err(err.render())
                }
            }
        }
    }
}

impl<T> Cx<Command<T>> {
    pub fn short(mut self, name: char) -> Self {
        // TODO - same approach in all the Cx things
        self.0.names.push(Name::Short(name));
        self
    }

    pub fn long(mut self, name: &'static str) -> Self {
        // TODO - same approach in all the Cx things
        self.0.names.push(Name::Long(Cow::Borrowed(name)));
        self
    }
}

impl<P, T> Parser<T> for Cx<P>
where
    P: Parser<T>,
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.0.run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.visit(visitor)
    }
}

pub trait Metavisit {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
}

impl<T: 'static> Metavisit for Rc<dyn Parser<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        <Self as Parser<T>>::visit(self, visitor)
    }
}

pub fn any<I, T>(metavar: &'static str, check: impl Fn(I) -> Option<T> + 'static) -> Cx<Anything<T>>
where
    I: std::str::FromStr + 'static,
    <I as std::str::FromStr>::Err: std::fmt::Display,
{
    Cx(Anything {
        metavar: Metavar(metavar),
        check: Box::new(move |os: OsOrStr| check(os.parse::<I>().ok()?)),
    })
}
