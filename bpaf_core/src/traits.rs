//! [`Parser`] trait and related private helper traits

use crate::{Ctx, Error, Exit, Lit, Metavar, Named};
use std::{marker::PhantomData, pin::Pin, rc::Rc};

/// This mostly exists to allow Executor and various helpers to depend on visits
/// without having to expose them to all the parser details
pub trait Visited {
    fn vi<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
    fn into_box(self) -> Box<dyn Visited>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<P: Parser> Visited for P {
    fn vi<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.visit(visitor)
    }
}

use crate::adapters::*;
pub trait Parser {
    type Output: 'static;
    fn run(&self, ctx: Ctx) -> impl Future<Output = Result<Self::Output, Error>>;

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);

    /// Convert the parser into a boxed, reference counted version
    fn into_rc(self) -> RcParser<Self::Output>
    where
        Self: Sized + 'static,
    {
        RcParser(Rc::new(self))
    }

    fn map<F, R>(self, map: F) -> Map<Self, F, R>
    where
        Self: Sized,
        F: Fn(Self::Output) -> R + 'static,
        R: 'static,
    {
        Map {
            inner: self,
            ctx: PhantomData,
            map,
        }
    }
    fn parse<F, R, E>(self, f: F) -> Parse<Self, F, E, R>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<R, E>,
        E: ToString,
    {
        Parse {
            inner: self,
            ctx: PhantomData,
            f,
        }
    }

    fn optional(self) -> Optional<Self::Output>
    where
        Self: Sized + 'static,
    {
        Optional {
            inner: self.into_rc(),
        }
    }

    fn many(self) -> Many<Self::Output>
    where
        Self: Sized + 'static,
    {
        Many {
            inner: self.into_rc(),
        }
    }

    fn some(self, message: &'static str) -> Many1<Self::Output>
    where
        Self: Sized + 'static,
    {
        Many1 {
            inner: self.into_rc(),
            message,
        }
    }

    fn count(self) -> Count<Self::Output>
    where
        Self: Sized + 'static,
    {
        Count {
            inner: self.into_rc(),
        }
    }

    fn to_options(self) -> OptionParser<Self::Output>
    where
        Self: Sized + 'static,
    {
        OptionParser {
            inner: self.into_rc(),
            info: Default::default(),
        }
    }

    fn guard<F>(self, check: F, message: &'static str) -> Guard<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> bool,
    {
        Guard {
            inner: self,
            check,
            message,
        }
    }

    fn hide(self) -> Hide<Self>
    where
        Self: Sized,
    {
        Hide {
            inner: self,
            only_usage: false,
        }
    }

    fn hide_usage(self) -> Hide<Self>
    where
        Self: Sized,
    {
        Hide {
            inner: self,
            only_usage: true,
        }
    }

    fn fallback(self, value: Self::Output) -> Fallback<Self::Output, Self>
    where
        Self: Sized,
    {
        Fallback {
            inner: self,
            value,
            value_str: None,
        }
    }

    fn group_help(self, help: &'static str) -> Group<Self>
    where
        Self: Sized,
    {
        Group {
            inner: self,
            title: help,
            descr: None,
        }
    }

    /// Return an index of an item in the argument list OR a position where parser gave up
    /// locating...
    fn offset(self) -> WithOffset<Self>
    where
        Self: Sized + Leaf,
    {
        WithOffset { inner: self }
    }

    fn then_exit(self, exit: Exit<Self::Output>) -> ThenExit<Self::Output, Self>
    where
        Self: Sized,
    {
        ThenExit { inner: self, exit }
    }

    fn or_exit(self, exit: Exit<Self::Output>) -> OrExit<Self::Output, Self>
    where
        Self: Sized,
    {
        OrExit { inner: self, exit }
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

impl<T: 'static, P: Parser<Output = T>> DynParser<T> for P {
    fn dyn_run(&self, ctx: Ctx) -> Pin<Box<dyn Future<Output = Result<T, Error>> + '_>> {
        Box::pin(<Self as Parser>::run(self, ctx))
    }
    fn dyn_visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.visit(visitor);
    }
}

impl<T: 'static> Parser for RcParser<T> {
    type Output = T;

    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        self.0.as_ref().dyn_run(ctx).await
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.as_ref().dyn_visit(visitor)
    }

    fn into_rc(self) -> Self {
        self
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
