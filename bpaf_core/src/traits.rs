//! [`Parser`] trait and related private helper traits

#![cfg_attr(doc, warn(unused_imports))]

use crate::{
    Ctx, Error, Exit, HelpCallback, HelpItem, HelpLiteral, Lit, Metavar, Named, Nest,
    adapters::{
        Fallback, FallbackStr, FallbackWith, Global, Group, Guard, Hide, Map, OptionParser,
        Optional, OrExit, Parse, ThenExit, WithOffset,
    },
    completions::CompHelp,
    error::ParseFailure,
    info::Info,
    repeat::{Collect, Count, Last, Many, Many1},
    vault::Vault,
};
use std::{boxed::Box, marker::PhantomData, pin::Pin, rc::Rc};

/// This mostly exists to allow Executor and various helpers to depend on visits
/// without having to expose them to all the parser details
pub trait Visited {
    fn vi<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
}

impl<P: Parser> Visited for P {
    fn vi<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.visit(visitor)
    }
}

pub trait Parser {
    type Output: 'static;
    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<Self::Output, Error>> + 'p;

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);

    fn help_literal(self, text: &'static str) -> HelpLiteral<Self>
    where
        Self: Sized,
    {
        HelpLiteral { inner: self, text }
    }

    fn help_callback<F>(self, map: F) -> HelpCallback<Self, F>
    where
        Self: Sized,
        F: Fn(Vec<HelpItem>) -> String,
    {
        HelpCallback {
            inner: self,
            cb: map,
        }
    }

    /// Convert the parser into a boxed, reference counted version
    fn into_rc(self) -> RcParser<Self::Output>
    where
        Self: Sized + 'static,
    {
        RcParser(Rc::new(self))
    }

    /// Convert the parser into a boxed version
    fn into_box(self) -> BoxParser<Self::Output>
    where
        Self: Sized + 'static,
    {
        BoxParser(Box::new(self))
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

    fn vault<F, E, R>(self, f: F) -> Vault<Self, F>
    where
        Self: Sized,
        F: for<'v> Fn(&'v mut crate::vault::Storage, Self::Output) -> Result<R, E>,
        E: ToString,
        R: 'static,
    {
        Vault { inner: self, op: f }
    }

    fn optional(self) -> Optional<Self>
    where
        Self: Sized + 'static,
    {
        Optional {
            inner: self,
            catch: false,
        }
    }

    /// Mark this parser as an anchor start parser.
    fn anchor_start(self) -> crate::adapters::AnchorStart<Self>
    where
        Self: Sized,
    {
        crate::adapters::AnchorStart { inner: self }
    }

    fn many(self) -> Many<Self::Output>
    where
        Self: Sized + 'static,
    {
        Many {
            inner: self.into_rc(),
        }
    }

    fn collect<C>(self) -> Collect<C, Self::Output>
    where
        Self: Sized + 'static,
    {
        Collect {
            inner: self.into_rc(),
            ctx: PhantomData,
            min: 0,
            max: u32::MAX,
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

    fn last(self) -> Last<Self::Output>
    where
        Self: Sized + 'static,
    {
        Last {
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
            pprint: None,
        }
    }

    fn fallback_str(self, value: &'static str) -> FallbackStr<Self>
    where
        Self: Sized,
    {
        FallbackStr {
            inner: self,
            fallback: value,
            pprint: None,
        }
    }

    fn fallback_with<F, E>(self, fallback: F) -> FallbackWith<Self, F, E>
    where
        Self: Sized,
        F: Fn() -> Result<Self::Output, E>,
        E: ToString,
    {
        FallbackWith {
            inner: self,
            fallback,
            ctx: PhantomData,
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

    fn then_exit<T>(self, exit: impl Fn(Self::Output) -> Exit<T> + 'static) -> ThenExit<T, Self>
    where
        Self: Sized,
    {
        ThenExit {
            inner: self,
            exit: Box::new(exit),
        }
    }

    fn or_exit(self, exit: impl Fn(ParseFailure) -> Exit<Self::Output> + 'static) -> OrExit<Self>
    where
        Self: Sized,
    {
        OrExit {
            inner: self,
            exit: Box::new(exit),
        }
    }

    fn or_else(
        self,
        other: impl Parser<Output = Self::Output> + 'static,
    ) -> crate::api::composite::Sum<Self::Output>
    where
        Self: Sized + 'static,
    {
        crate::api::composite::Sum {
            items: vec![self.into_rc(), other.into_rc()],
        }
    }

    /// Override help description when used shell completion
    ///
    /// Full help message might be too verbose for use in shell completion,
    /// with this method you can override it for completion purposes only
    fn comp_help(self, help: &'static str) -> CompHelp<Self>
    where
        Self: Sized,
    {
        CompHelp { inner: self, help }
    }

    /// Mark this parser as global - its triggers are shared across all executor scopes.
    ///
    /// Global tasks' triggers are visible to every subcommand and nested executor,
    /// and when a global task advances it notifies all sums, not just those within
    /// the current executor scope.
    fn global(self) -> Global<Self>
    where
        Self: Sized,
    {
        Global { inner: self }
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
    fn item<'t>(&mut self, item: Item<'a, 't>);
    fn identify(&self) -> VKind;
    fn push_group(&mut self, group: VisitGroup);
    fn pop_group(&mut self);
}

#[derive(Clone, Copy)]
pub enum Item<'a, 't> {
    /// Top level of the parser, should be visited just once, at the beginning of the evaluation
    OptionParser {
        /// Parser information - header, description, footer
        info: &'a Info,
        /// A way to visit the inner parser as a whole, possibly with a different visitor. Visitor that
        /// generates Help uses it to extract usage information.
        ///
        /// After receiving this event visitor will receive events related to this parser
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
        /// Is the item "strict positional" that needs to be to the right of `--`?
        strict: bool,
    },
    Command {
        names: &'a [Lit<'static>],
        help: Option<&'a str>,
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
        outer: &'a Nest,
        inner: &'a dyn Visited,
    },
    Section {
        title: &'a str,
        descr: Option<&'a str>,
        inner: &'a dyn Visited,
    },
    /// Already rendered fragment to be used by Usage and Help visitors and ignored by all others
    /// It can contain help group definitions, similar to .group_help("xxx") method
    Rendered {
        text: &'t str,
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
    /// items visible across all subcommands
    Global,
}

/// Helper trait that allows shoving non-dyn compatible trait [`Parser`] into an [`Rc`]
trait DynParser<T: 'static> {
    fn dyn_eval<'p>(&'p self, ctx: Ctx<'p>)
    -> Pin<Box<dyn Future<Output = Result<T, Error>> + 'p>>;
    fn dyn_visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>);
}

impl<T: 'static, P: Parser<Output = T>> DynParser<T> for P {
    fn dyn_eval<'p>(
        &'p self,
        ctx: Ctx<'p>,
    ) -> Pin<Box<dyn Future<Output = Result<T, Error>> + 'p>> {
        Box::pin(<Self as Parser>::eval(self, ctx))
    }
    fn dyn_visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.visit(visitor);
    }
}

impl<T: 'static> Parser for RcParser<T> {
    type Output = T;

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<T, Error>> {
        self.0.as_ref().dyn_eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.as_ref().dyn_visit(visitor)
    }

    fn into_rc(self) -> Self {
        self
    }
}

/// Reference counted boxed [`Parser<Output = T>`](Parser) - it is cheap to clone
#[repr(transparent)]
pub struct RcParser<T>(Rc<dyn DynParser<T>>);

impl<T> Clone for RcParser<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Heap allocated boxed [`Parser<Output = T>`](Parser)
#[repr(transparent)]
pub struct BoxParser<T>(Box<dyn DynParser<T>>);

impl<T: 'static> Parser for BoxParser<T> {
    type Output = T;

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<T, Error>> {
        self.0.as_ref().dyn_eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.as_ref().dyn_visit(visitor)
    }

    fn into_box(self) -> Self {
        self
    }
}

#[diagnostic::on_unimplemented(
    message = "Leaf is a marker trait that is only implemented for atomic parsers, not constructed ones"
)]
/// The idea is to have a marker trait for parsers that can either fail or succeed, but not in
/// between. For example, `construct!(a, b)` will remain in partially succeeded state once either
/// matches and will remain in this state until b either succeeds or fails.
///
/// Main problem with partially succeeding parsers is that catching errors from them can lead to
/// data loss, but also the concept of "this parser started here" becomes blurry.
pub trait Leaf {}

macro_rules! tuple_impl {
    ($( $name:ident)+) => {
        impl<$( $name: Parser + 'static),+> Parser for ($($name),+) {
            type Output = ($( $name::Output ),+);

            async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
                #![allow(non_snake_case)]
                let ($( $name ),+) = &self;
                $( let $name = ctx.spawn(crate::Kind::Prod, $name);)+
                ctx.wait_for_children().await;
                let mut err = None;
                $( let $name = $name.take().map_err(|e| e.append_to(&mut err));)+

                if let Some(err) = err {
                    Err(err)
                } else {
                    Ok(($($name?),+))
                }
            }

            fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
                #![allow(non_snake_case)]
                visitor.push_group(VisitGroup::Prod);
                let ($( $name ),+) = &self;
                $($name.visit(visitor); )+
                visitor.pop_group();
            }
        }
    };
}

tuple_impl! { A B }
tuple_impl! { A B C }
tuple_impl! { A B C D }
tuple_impl! { A B C D E }
tuple_impl! { A B C D E F }
tuple_impl! { A B C D E F G }
tuple_impl! { A B C D E F G H }
tuple_impl! { A B C D E F G H I }
tuple_impl! { A B C D E F G H I J }
tuple_impl! { A B C D E F G H I J K }
tuple_impl! { A B C D E F G H I J K L }
