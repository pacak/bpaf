//! [`Parser`] trait and related private helper traits

use crate::{Ctx, Error, Exit, Lit, Metavar, Named};
use std::{marker::PhantomData, pin::Pin, rc::Rc};

use crate::adapters::*;
pub trait Parser<T: 'static>: Visited {
    fn run(&self, ctx: Ctx) -> impl Future<Output = Result<T, Error>>;

    /// Convert the parser into a boxed, reference counted version
    fn into_rc(self) -> RcParser<T>
    where
        Self: Sized + Parser<T> + 'static,
    {
        RcParser(Rc::new(self))
    }

    fn map<F, R>(self, map: F) -> Map<Self, F, T, R>
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
    fn parse<F, R, E>(self, f: F) -> Parse<T, Self, F, E, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> Result<R, E>,
        E: ToString,
    {
        Parse {
            inner: self,
            ctx: PhantomData,
            f,
        }
    }

    fn optional(self) -> Optional<T>
    where
        Self: Sized + 'static,
    {
        Optional {
            inner: self.into_rc(),
        }
    }

    fn many(self) -> Many<T>
    where
        Self: Sized + 'static,
    {
        Many {
            inner: self.into_rc(),
        }
    }

    fn some(self, message: &'static str) -> Many1<T>
    where
        Self: Sized + 'static,
    {
        Many1 {
            inner: self.into_rc(),
            message,
        }
    }

    fn count(self) -> Count<T>
    where
        Self: Sized + 'static,
    {
        Count {
            inner: self.into_rc(),
        }
    }

    fn to_options(self) -> OptionParser<T>
    where
        Self: Sized + 'static,
    {
        OptionParser {
            inner: self.into_rc(),
            info: Default::default(),
        }
    }

    fn guard<F>(self, check: F, message: &'static str) -> Guard<T, Self, F>
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        Guard {
            inner: self,
            check,
            message,
            ctx: PhantomData,
        }
    }

    fn hide(self) -> Hide<T, Self>
    where
        Self: Sized,
    {
        Hide {
            inner: self,
            ctx: PhantomData,
            only_usage: false,
        }
    }

    fn hide_usage(self) -> Hide<T, Self>
    where
        Self: Sized,
    {
        Hide {
            inner: self,
            ctx: PhantomData,
            only_usage: true,
        }
    }

    fn fallback(self, value: T) -> Fallback<T, Self>
    where
        Self: Sized,
    {
        Fallback {
            inner: self,
            value,
            value_str: None,
        }
    }

    fn group_help(self, help: &'static str) -> Group<T, Self>
    where
        Self: Sized,
    {
        Group {
            inner: self,
            title: help,
            ctx: PhantomData,
            descr: None,
        }
    }

    /// Return an index of an item in the argument list OR a position where parser gave up
    /// locating...
    fn offset(self) -> WithOffset<T, Self>
    where
        Self: Sized + Leaf,
    {
        WithOffset {
            inner: self,
            ctx: PhantomData,
        }
    }

    fn then_exit(self, exit: Exit<T>) -> ThenExit<T, Self>
    where
        Self: Sized,
    {
        ThenExit { inner: self, exit }
    }

    fn or_exit(self, exit: Exit<T>) -> OrExit<T, Self>
    where
        Self: Sized,
    {
        OrExit { inner: self, exit }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VKind {
    Error,
    Usage,
    Help,
    Custom,
}
pub trait Visitor<'a> {
    fn item(&mut self, item: Item<'a>);
    fn identify(&self) -> VKind;
    fn push_group(&mut self, group: VisitGroup);
    fn pop_group(&mut self);
}

#[derive(Clone, Copy)]
pub enum Item<'a> {
    /// Top level of the parser, should be visited just once, at the beginning of the evaluation
    OptionParser {
        /// Parser information - header, description, footer
        info: &'a Info,
        /// A way to visit the inner parser - [`Help`] visitor uses it to extract usage information
        inner: &'a dyn Visited,
    },
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
        names: &'a [Lit<'static>],
        info: &'a Info,
        inner: &'a dyn Visited,
    },

    /// Items in this group belong to a single adjacent group and must succeed all
    /// at once or fail. For `--help` purposes this usually means that this is a parser
    /// that takes one named item followed by multiple positional items and should
    /// be rendered as a block
    ///
    /// ```text
    ///     --name <A> <B> <C> ... help for name part
    ///       <A> ... help for A
    ///       <B> ... help for B
    ///       <C> ... help for C
    ///     <- blank line
    /// ```
    ///
    /// All the inner parsers go under "named" parsers unless [`Item::Section`] moves it elsewhere
    Nested {
        named: &'a Named,
        inner: &'a dyn Visited,
    },
    Section {
        title: &'a str,
        descr: Option<&'a str>,
        inner: &'a dyn Visited,
    },
    /// Already rendered fragment to be used by Usage and Help visitors and ignored by all others
    Rendered {
        gr: Option<Gr>,
        text: &'a str,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Gr {
    Named,
    Pos,
    Cmd,
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

#[diagnostic::on_unimplemented(
    message = "Leaf is a marker trait that is only implemented for primitive parsers, not for anything composite."
)]
/// The idea is to have a marker trait for parsers that can either fail or succeed, but not in
/// between. For example, `construct!(a, b)` will remain in partially succeeded state once either
/// matches and will remain in this state until b either succeeds or fails.
///
/// Main problem with partially succeeding parsers is that catching errors from them can lead to
/// data loss, but also the concept of "this parser started here" becomes blurry.
pub trait Leaf {}
