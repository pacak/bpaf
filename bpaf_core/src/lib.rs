mod visitor;

pub mod executor;
pub mod parsers;
pub use error::Error;
pub use executor::Con;
use executor::{Ctx, Fragment};
use named::{Argument, Flag, Named};
use parsers::{Guard, Many};
use positional::Positional;
use std::{marker::PhantomData, ops::RangeBounds, rc::Rc};
mod error;

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
    ($first:ident $($tokens:tt)*) =>
        {{ $crate::construct!(@prepare [pos] [] $first $($tokens)*) }};

    // construct!([a, b, c])
    ([$first:ident $($tokens:tt)*]) =>
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
        $crate::Alt { items: vec![ ($field.to_box())*] }
    };

    // For product type the logic is a bit more complicated - do one more step
    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::construct!(@fin $ty [ $($fields)* ])
    };

    // === Making a body for the product parser

    // Two special cases where we construct something with no fields, use `Parser::pure` for that
    (@fin [named [$($con:tt)+]] []) => { $crate::pure($($con)+ { })};
    (@fin [pos   [$($con:tt)+]] []) => { $crate::pure($($con)+ ( ))};


    (@fin $ty:tt [$($fields:ident)*]) => {{
        $(let $fields = $fields.into_rc();)*
        // <- visitor goes here
        let run = move |ctx: $crate::Ctx| {
            let mut n = 0;

            $(let $fields = $fields.clone();)*
            let frag: $crate::executor::Fragment::<_> = Box::pin(async move {
                let id = ctx.current_id();
                $(
                    let $fields = ctx.spawn(id.prod(n), &$fields, false);
                    n += 1;
                )*
                // <- check parent errors here
                ::std::result::Result::Ok::<_, $crate::Error>
                    ($crate::construct!(@make $ty [$($fields)*]))
            });
            frag

        //
        //     args.current = None;
        //     ::std::result::Result::Ok::<_, $crate::Error>
        //         ($crate::construct!(@make $ty [$front $($fields)*]))
        // };
        };
        $crate::Con { run: Box::new(run) }
    }};

    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ { $($fields.await?),* } };
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
pub struct Cx<I>(I);

/// Make named item with short name
pub fn short(name: char) -> Cx<Named> {
    Cx(named::short(name))
}

pub fn long(name: &'static str) -> Cx<Named> {
    Cx(named::long(name))
}

pub fn positional<T>(meta: &'static str) -> Cx<Positional<T>> {
    Cx(positional::positional(meta))
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
    pub fn help(mut self, help: String) -> Self {
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
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
    // adjacent_to
}

impl<T> Cx<Argument<T>> {
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
}

/// # Positional items
///
/// Parses a positional item, usually with something attached
impl<T> Cx<Positional<T>> {
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
}

mod named {
    use std::{marker::PhantomData, str::FromStr};

    use crate::{
        error::Error,
        executor::{ArgFut, Ctx, FlagFut, Fragment},
        Parser,
    };

    pub struct Argument<T> {
        named: Named,
        ty: PhantomData<T>,
        meta: &'static str,
    }

    pub struct Flag<T> {
        named: Named,
        present: T,
        absent: Option<T>,
    }

    impl<T> Parser<T> for Argument<T>
    where
        T: std::fmt::Debug + 'static + FromStr,
        <T as FromStr>::Err: std::error::Error,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
            Box::pin(async move {
                let fut = ArgFut {
                    name: &self.named.names,
                    meta: self.meta,
                    ctx,
                    task_id: None,
                };

                let s = fut.await?;
                match s.parse::<T>() {
                    Ok(t) => Ok(t),
                    Err(e) => Err(Error::parse_fail(format!("Can't parse {s:?} : {e}"))),
                }
            })
        }
    }

    impl<T> Parser<T> for Flag<T>
    where
        T: std::fmt::Debug + Clone + 'static,
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

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Name<'a> {
        Short(char, [u8; 4]),
        Long(&'a str),
    }

    impl Name<'_> {
        pub(crate) fn as_bytes(&self) -> &[u8] {
            match self {
                Name::Short(_, a) => a.as_slice(),
                Name::Long(l) => l.as_bytes(),
            }
        }

        pub(crate) fn short(name: char) -> Name<'static> {
            let mut buf = [0; 4];
            name.encode_utf8(&mut buf);

            Name::Short(name, buf)
        }
    }

    impl std::borrow::Borrow<[u8]> for Name<'_> {
        fn borrow(&self) -> &[u8] {
            self.as_bytes()
        }
    }

    pub struct Named {
        names: Vec<Name<'static>>,
        help: Option<String>,
    }

    pub(crate) fn short(name: char) -> Named {
        let mut buf = [0; 4];
        name.encode_utf8(&mut buf);
        Named {
            names: vec![Name::Short(name, buf)],
            help: None,
        }
    }
    pub(crate) fn long(name: &'static str) -> Named {
        Named {
            names: vec![Name::Long(name)],
            help: None,
        }
    }
    impl Named {
        pub(crate) fn short(&mut self, name: char) {
            let mut buf = [0; 4];
            name.encode_utf8(&mut buf);
            self.names.push(Name::Short(name, buf));
        }
        pub(crate) fn long(&mut self, name: &'static str) {
            self.names.push(Name::Long(name));
        }
        pub(crate) fn help(&mut self, help: String) {
            self.help = Some(help);
        }
        pub(crate) fn argument<T>(self, meta: &'static str) -> Argument<T> {
            Argument {
                named: self,
                ty: PhantomData,
                meta,
            }
        }
    }
    impl<T> Flag<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.named.help = Some(help);
        }
    }

    impl<T> Argument<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.named.help = Some(help);
        }
    }
}

mod positional {
    use std::{marker::PhantomData, str::FromStr};

    use crate::{
        error::{Error, Message},
        executor::{Ctx, PositionalFut},
        Parser,
    };
    pub struct Positional<T> {
        meta: &'static str,
        help: Option<String>,
        ty: PhantomData<T>,
    }
    pub(crate) fn positional<T>(meta: &'static str) -> Positional<T> {
        Positional {
            meta,
            help: None,
            ty: PhantomData,
        }
    }
    impl<T> Positional<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.help = Some(help);
        }
    }

    impl<T> Parser<T> for Positional<T>
    where
        T: std::fmt::Debug + 'static + FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> crate::executor::Fragment<'a, T> {
            Box::pin(async {
                let s = PositionalFut {
                    ctx,
                    task_id: None,
                    meta: self.meta,
                }
                .await;
                T::from_str(s?).map_err(|e| Error {
                    message: Message::ParseFailed(None, format!("{e}")),
                })
            })
        }
    }
}

pub trait Parser<T: 'static + std::fmt::Debug> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T>;
    fn into_box(self) -> Box<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn into_rc(self) -> Rc<dyn Parser<T>>
    where
        Self: Sized + 'static,
    {
        Rc::new(self)
    }

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

    fn guard<F, Q>(self, check: F, message: &'static str) -> Guard<Self, F, Q>
    where
        T: std::borrow::Borrow<Q> + std::fmt::Debug + 'static,
        Self: Sized,
    {
        Guard {
            inner: self,
            check,
            message,
            ty: PhantomData,
        }
    }
}

impl<P, T> Parser<T> for Cx<P>
where
    P: Parser<T>,
    T: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: executor::Ctx<'a>) -> Fragment<'a, T> {
        self.0.run(ctx)
    }
}
