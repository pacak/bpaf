use std::{borrow::Borrow, marker::PhantomData};

use crate::{
    ctx::Ctx,
    error::Error,
    executor::{
        family::{NodeKind, Parent},
        Fragment,
    },
    Parser,
};

#[derive(Clone)]
pub struct Many<P, C, T> {
    pub(crate) inner: P,
    pub(crate) error: &'static str,
    pub(crate) at_least: u32,
    pub(crate) at_most: u32,
    pub(crate) ty: PhantomData<(T, C)>,
}

impl<T, P, C> Parser<C> for Many<P, C, T>
where
    P: Parser<T>,
    T: std::fmt::Debug + 'static,
    C: FromIterator<T> + std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, C> {
        let mut res = Vec::new();
        Box::pin(async move {
            let parent = Parent {
                id: ctx.current_id(),
                field: 0,
                kind: NodeKind::Prod,
            };

            let err = loop {
                if res.len() as u32 >= self.at_most {
                    break Ok(());
                };
                match ctx.spawn(parent, &self.inner, true).await {
                    Ok(t) => res.push(t),
                    Err(e) if e.handle_with_fallback() => break Err(e),
                    Err(e) => return Err(e),
                }
            };
            // TODO - handle error-missing + at least
            if res.len() as u32 >= self.at_least {
                Ok(res.into_iter().collect())
            } else if let Err(err) = err {
                Err(err)
            } else {
                panic!("at least/at most disagreement");
            }
        })
    }
}

pub struct Guard<P, F, Q> {
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) ty: PhantomData<Q>,
    pub(crate) message: &'static str,
}

impl<P, F, T, Q> Parser<T> for Guard<P, F, Q>
where
    P: Parser<T>,
    T: Borrow<Q> + std::fmt::Debug + 'static,
    F: Fn(&Q) -> bool,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let t = self.inner.run(ctx).await?;
            if (self.check)(t.borrow()) {
                Ok(t)
            } else {
                Err(Error::fail(self.message))
            }
        })
    }
}

pub struct Count<P, T> {
    pub(crate) inner: P,
    pub(crate) ctx: PhantomData<T>,
}

impl<P, T> Parser<usize> for Count<P, T>
where
    P: Parser<T>,
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, usize> {
        Box::pin(async move {
            let mut count = 0;
            while (self.inner.run(ctx.clone()).await).is_ok() {
                count += 1;
            }
            Ok(count)
        })
    }
}

pub struct Last<P, T> {
    pub(crate) inner: P,
    pub(crate) ctx: PhantomData<T>,
}

impl<P, T> Parser<T> for Last<P, T>
where
    T: 'static,
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let mut last = self.inner.run(ctx.clone()).await?;
            while let Ok(t) = self.inner.run(ctx.clone()).await {
                last = t;
            }
            Ok(last)
        })
    }
}

pub struct Map<P, F, T, R> {
    pub(crate) inner: P,
    pub(crate) map: F,
    pub(crate) ctx: PhantomData<(T, R)>,
}

impl<P, F, T, R> Parser<R> for Map<P, F, T, R>
where
    P: Parser<T>,
    F: Fn(T) -> R,

    R: 'static,
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, R> {
        let inner = self.inner.run(ctx);
        Box::pin(async move { Ok((self.map)(inner.await?)) })
    }
}

pub struct Parse<P, F, T, R> {
    pub(crate) inner: P,
    pub(crate) ctx: PhantomData<(T, R)>,
    pub(crate) f: F,
}

impl<P, F, T, R, E> Parser<R> for Parse<P, F, T, R>
where
    P: Parser<T>,
    F: Fn(T) -> Result<R, E>,
    E: ToString,
    R: 'static,
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, R> {
        let inner = self.inner.run(ctx);
        Box::pin(async move {
            let t = inner.await?;
            (self.f)(t).map_err(|e| Error::parse_fail(e.to_string()))
        })
    }
}
