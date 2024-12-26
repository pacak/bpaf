use std::{borrow::Borrow, marker::PhantomData};

use crate::{
    ctx::Ctx,
    error::Error,
    executor::{futures::LiteralFut, BranchId, Fragment, Id, NodeKind, Parent},
    named::Name,
    visitor::Group,
    OptionParser, Parser,
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
            let (_branch, id) = ctx.current_id();
            let parent = Parent {
                id,
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Guard<P, F> {
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) message: &'static str,
}

impl<P, F, T> Parser<T> for Guard<P, F>
where
    P: Parser<T>,
    T: 'static,
    F: Fn(&T) -> bool,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let t = self.inner.run(ctx).await?;
            if (self.check)(&t) {
                Ok(t)
            } else {
                Err(Error::fail(self.message))
            }
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct Optional<P> {
    pub(crate) inner: P,
}

impl<P, T> Parser<Option<T>> for Optional<P>
where
    P: Parser<T>,
    T: std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, Option<T>> {
        Box::pin(async move {
            let _guard = FallbackGuard::new(ctx.clone());
            match self.inner.run(ctx.clone()).await {
                Ok(ok) => Ok(Some(ok)),
                Err(e) if e.handle_with_fallback() && ctx.items_consumed.get() == 0 => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

/// Fallback registration
///
/// Mostly there so we can remove the interest in fallback once Optional parser finishes or gets
/// dropped
struct FallbackGuard<'ctx> {
    branch: BranchId,
    id: Id,
    ctx: Ctx<'ctx>,
}

impl<'ctx> FallbackGuard<'ctx> {
    fn new(ctx: Ctx<'ctx>) -> Self {
        let (branch, id) = ctx.current_id();
        ctx.add_fallback(branch, id);
        Self { branch, id, ctx }
    }
}

impl Drop for FallbackGuard<'_> {
    fn drop(&mut self) {
        self.ctx.remove_fallback(self.branch, self.id);
    }
}

pub struct Command<T> {
    pub(crate) names: Vec<Name<'static>>,
    pub(crate) parser: OptionParser<T>,
    pub(crate) adjacent: bool,
}

impl<T: 'static> Parser<T> for Command<T> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            LiteralFut {
                values: &self.names,
                ctx: ctx.clone(),
                task_id: None,
            }
            .await?;
            println!("========== Running inner parser");
            ctx.advance(1);
            println!("{:?}", &ctx.args[ctx.cur()..]);
            let runner = crate::executor::Runner::new(ctx);

            let r = runner.run_parser(&self.parser.0.parser);
            println!("=========== Inner done with {:?}", r.is_ok());
            r
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let recursive = visitor.command(&self.names);
        if recursive {
            self.parser.0.parser.visit(visitor);
        }
    }
}
