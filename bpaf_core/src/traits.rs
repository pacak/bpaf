//! [`Parser`] trait and related private helper traits

use crate::{Bp, Ctx, Error, Metavar, Name};
use std::{borrow::Cow, marker::PhantomData, pin::Pin, rc::Rc};

use crate::adapters::*;
pub trait Parser<T: 'static>: Visited {
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
            help: None,
        })
    }

    fn guard<F>(self, check: F, message: &'static str) -> Bp<Guard<T, Self, F>>
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        Bp(Guard {
            inner: self,
            check,
            message,
            ctx: PhantomData,
        })
    }
}

impl<A: Visited, B: Visited> Visited for (A, B) {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.visit(visitor);
        self.1.visit(visitor);
    }
}

pub trait Visited {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
    fn into_box(self) -> Box<dyn Visited>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub trait Visitor<'a> {
    fn item(&mut self, item: Item<'a>);
    fn push_group(&mut self, group: Group);
    fn pop_group(&mut self);
}

#[derive(Copy, Clone)]
pub enum Item<'a> {
    Flag {
        names: &'a [Name<'static>],
        help: Option<&'a str>,
    },
    Arg {
        names: &'a [Name<'static>],
        meta: Metavar,
        help: Option<&'a str>,
    },
    Positional {
        meta: Metavar,
        help: Option<&'a str>,
    },
    Command {
        names: &'a [Cow<'static, str>],
        help: Option<&'a str>,
        inner: &'a dyn Visited,
    },
    Nested {
        names: &'a [Name<'static>],
        help: Option<&'a str>,
        inner: &'a dyn Visited,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Group {
    /// inner parser can succeed multiple times, required unless made optional
    Many,
    /// inner parser can succeed with no input
    Optional,
    /// product group, all members must succeed
    Prod,
    /// sum group, exactly one member must succeed
    Sum,
    /// All nested items should go into a custom section
    HelpSection(&'static str),
    /// Items in this group belong to a sub-parser such as subcommand or
    /// a parser adjacent to a name
    Subparser,
}

/// Helper trait that allows shoving non-dyn compatible trait [`Parser`] into an [`Rc`]
trait DynParser<T: 'static> {
    fn dyn_run(&self, ctx: Ctx) -> Pin<Box<dyn Future<Output = Result<T, Error>> + '_>>;
    fn dyn_visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
}

impl<T: 'static, P: Parser<T>> DynParser<T> for P {
    fn dyn_run(&self, ctx: Ctx) -> Pin<Box<dyn Future<Output = Result<T, Error>> + '_>> {
        Box::pin(<Self as Parser<T>>::run(self, ctx))
    }
    fn dyn_visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.visit(visitor);
    }
}

impl<T: 'static> Visited for Bp<RcParser<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.0.as_ref().dyn_visit(visitor)
    }
}

impl<T: 'static> Visited for RcParser<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.as_ref().dyn_visit(visitor)
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
