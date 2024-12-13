use std::{borrow::Borrow, marker::PhantomData};

use crate::{
    error::Error,
    executor::{
        family::{NodeKind, Parent},
        Ctx, Fragment,
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
