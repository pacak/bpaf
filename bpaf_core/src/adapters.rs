//! Adapters that implement functionality used by the [`Parser`] trait

use crate::{
    Bp, Error, Item, Kind, ParseFailure, Parser, Problem, RawCtx, RcParser, Reason, Task, Visited,
    args::Args,
    complete::{complete_command, handle_subparser_complete},
    make_handle,
    traits::Group,
    r#yield,
};
use std::{borrow::Cow, marker::PhantomData};

pub(crate) struct Map<P, F, T, R> {
    pub(crate) inner: P,
    pub(crate) map: F,
    pub(crate) ctx: PhantomData<(T, R)>,
}

impl<T: 'static, P: Parser<T>, R: 'static, F: Fn(T) -> R> Parser<R> for Map<P, F, T, R> {
    async fn run(&self, ctx: crate::Ctx) -> Result<R, crate::Error> {
        let t = self.inner.run(ctx).await?;
        Ok((self.map)(t))
    }
}

impl<T: 'static, P: Parser<T>, F, R> Visited for Map<P, F, T, R> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub(crate) struct Optional<T> {
    pub(crate) inner: RcParser<T>,
}
impl<T: 'static> Parser<Option<T>> for Optional<T> {
    async fn run(&self, ctx: crate::Ctx) -> Result<Option<T>, Error> {
        // TODO - use scoped spawn, Scope and get rid of spawn_with_early_exit
        let (h, pair) = ctx.spawn_with_early_exit(self.inner.clone());
        r#yield().await;
        ctx.remove_early_exit(pair);
        match h.take() {
            Ok(v) => Ok(Some(v)),
            Err(err) => {
                if err.can_catch() && ctx.current_task.borrow().consumed == 0 {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl<T: 'static> Visited for Optional<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct OptionParser<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) help: Option<String>,
}

impl<T: 'static> Bp<OptionParser<T>> {
    pub fn run_inner(&self, args: impl Into<Args>) -> Result<T, ParseFailure> {
        let ctx = RawCtx::new(args.into());

        let (handle, act) = ctx.make_raw_task(Bp(self.0.inner.clone()));
        let info = ctx.make_child_info(Kind::Prod);
        let task = Task { act, info };
        ctx.add_task(task);
        let executor_res = ctx.execute(&self.0.inner);
        let res = handle.take();
        if res.is_ok() {
            executor_res?;
        }
        Ok(res?)
    }

    pub fn command(self, name: impl Into<Cow<'static, str>>) -> Bp<Command<T>> {
        Bp(Command {
            names: vec![name.into()],
            inner: self.0,
        })
    }
}

impl<T: 'static> Parser<T> for Bp<Command<T>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let (out, handle) = make_handle();
        let inner = &self.0.inner.inner;
        let populate = |ctx: crate::Ctx| {
            // out.clone() is slightly cursed. `parse_literal_and` takes a reference to a closure
            // to avoid instantiating multiple copies of boring code so this closure must be `Fn`
            // (and not `FnOnce`), meaning extra clone for out even though the closure will
            // be executed exactly once
            let act = ctx.make_act(out.clone(), inner.clone());
            let info = ctx.make_child_info(Kind::Prod);
            ctx.add_task(Task { act, info });
        };
        let res = ctx.parse_literal_and(&self.0.names, &populate, inner).await;
        let res = res.map_err(|err| complete_command(&self.0.names, err));
        res?;
        handle.take().map_err(handle_subparser_complete)
    }
}

impl<T: 'static> Visited for Bp<Command<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let item = Item::Command {
            names: &self.0.names,
            help: self.0.inner.help.as_deref(),
            inner: &self.0.inner.inner,
        };
        visitor.item(item);
    }
}

pub struct Many<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser<Vec<T>> for Bp<Many<T>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
        let mut res = Vec::new();
        let start = ctx.next_free.get();
        let mut consumed_before = 0;
        while matches!(&*ctx.wakeup_reason.borrow(), Reason::Pass | Reason::Push) {
            ctx.next_free.set(start);
            let (h, pair) = ctx.spawn_with_early_exit(self.0.inner.clone());

            r#yield().await;
            ctx.remove_early_exit(pair);

            let consumed_after = ctx.current_task.borrow().consumed;
            let advanced = consumed_after > consumed_before;
            consumed_before = consumed_after;

            let val = h.take();

            match (advanced, val) {
                (true, Ok(v)) => res.push(v),
                (true, Err(e)) => return Err(e),
                (false, Ok(v)) => {
                    if res.is_empty() {
                        res.push(v);
                    }
                    break;
                }
                (false, Err(e)) if e.can_catch() => break,
                (false, Err(e)) => return Err(e),
            }
        }
        Ok(res)
    }
}
impl<T: 'static> Visited for Bp<Many<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Many);
        visitor.push_group(Group::Optional);
        self.0.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

pub struct Many1<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) message: &'static str,
}
impl<T: 'static> Parser<Vec<T>> for Bp<Many1<T>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
        let res = Bp(Many {
            inner: self.0.inner.clone(),
        })
        .run(ctx)
        .await?;
        if res.is_empty() {
            Err(Error::Problem(Problem::Static(self.0.message)))
        } else {
            Ok(res)
        }
    }
}
impl<T: 'static> Visited for Bp<Many1<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Many);
        self.0.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Command<T> {
    names: Vec<Cow<'static, str>>,
    inner: OptionParser<T>,
}

pub struct Guard<T, P, F> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) message: &'static str,
}

impl<T: 'static, F: Fn(&T) -> bool, P: Parser<T>> Parser<T> for Bp<Guard<T, P, F>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let r = self.0.inner.run(ctx.clone()).await?;

        if (self.0.check)(&r) {
            Ok(r)
        } else {
            Err(Error::Problem(Problem::GuardFailed {
                message: self.0.message,
                range: ctx.leaf_consumed(),
            }))
        }
    }
}

impl<F, T: 'static, P: Parser<T>> Visited for Bp<Guard<T, P, F>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.0.inner.visit(visitor);
    }
}
