//! [`Parser`] trait and related private helper traits

use crate::{Bp, Ctx, Error, Metavar, Named};
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

    fn some(self, message: &'static str) -> Bp<Many1<T>>
    where
        Self: Sized + 'static,
    {
        Bp(Many1 {
            inner: self.into_rc().0,
            message,
        })
    }

    fn to_options(self) -> Bp<OptionParser<T>>
    where
        Self: Sized + 'static,
    {
        Bp(OptionParser {
            inner: self.into_rc().0,
            info: Default::default(),
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

    fn hide(self) -> Bp<Hide<T, Self>>
    where
        Self: Sized,
    {
        Bp(Hide {
            inner: self,
            ctx: PhantomData,
        })
    }

    fn fallback(self, value: T) -> Bp<Fallback<T, Self>>
    where
        Self: Sized,
    {
        Bp(Fallback { inner: self, value })
    }

    fn group_help(self, help: &'static str) -> Bp<Group<T, Self>>
    where
        Self: Sized,
    {
        Bp(Group {
            inner: self,
            title: help,
            ctx: PhantomData,
            descr: None,
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
    fn push_group(&mut self, group: VisitGroup);
    fn pop_group(&mut self);
}

#[derive(Copy, Clone)]
pub enum Item<'a> {
    Flag {
        named: &'a Named,
    },
    Arg {
        named: &'a Named,
        meta: Metavar,
    },
    Positional {
        meta: Metavar,
        help: Option<&'a str>,
    },
    Command {
        names: &'a [Cow<'static, str>],
        info: &'a Info,
        inner: &'a dyn Visited,
    },

    /// Items in this group belong to a single adjacent group and must succeed all
    /// at once or fail. For `--help` purposes this usually means that this is a parser
    /// that takes one named item followed by multiple positional items and should
    /// be rendered as a block
    ///
    /// --name <A> <B> <C> ... help for name part
    ///   <A> ... help for A
    ///   <B> ... help for B
    ///   <C> ... help for C
    /// <- blank line
    ///
    /// All the inner parsers go under "named" parsers unless [`Item::Section`] moves it elsewhere
    Nested {
        named: &'a Named,
        inner: &'a dyn Visited,
    },
    OptionParser {
        info: &'a Info,
        inner: &'a dyn Visited,
    },
    Section {
        title: &'a str,
        descr: Option<&'a str>,
        inner: &'a dyn Visited,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VisitGroup {
    /// inner parser can succeed multiple times, required unless made optional
    Many,
    /// inner parser can succeed with no input
    Optional,
    /// product group, all members must succeed
    Prod,
    /// sum group, exactly one member must succeed
    Sum,
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
