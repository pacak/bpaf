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
