//! All the individual parser components
//!
//! This module exposes parsers that accept further configuration with builder pattern
//!
//! **In most cases you won’t be using those names directly, they’re only listed here to provide access to documentation**

//!
use crate::{
    ctx::Ctx,
    error::{Error, Metavar},
    executor::{
        futures::{AltFuture, AnyFut, LiteralFut},
        BranchId, Fragment, Id, NodeKind, Parent,
    },
    named::Name,
    split::OsOrStr,
    visitor::{Group, Mode, Visitor},
    Metavisit, OptionParser, Parser,
};
use std::{any::Any, marker::PhantomData, rc::Rc};

pub struct ManyV<P> {
    pub(crate) inner: P,
}

impl<P, T> Parser<Vec<T>> for ManyV<P>
where
    T: 'static,
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, Vec<T>> {
        Box::pin(async move {
            let mut res = Vec::new();
            let (_branch, id) = ctx.current_id();
            let parent = Parent {
                id,
                field: 0,
                kind: NodeKind::Prod,
            };
            let _guard = FallbackGuard::new(ctx.clone());
            let mut prev_consumed = 0;
            while !ctx.is_term() {
                let r = ctx.spawn(parent, &self.inner, true).await;

                let consumed = ctx.items_consumed.get() - prev_consumed;
                prev_consumed = ctx.items_consumed.get();
                match r {
                    Ok(val) => res.push(val),
                    Err(err) => {
                        if err.handle_with_fallback() && consumed == 0 {
                            break;
                        } else {
                            return Err(err);
                        }
                    }
                }
            }

            Ok(res)
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(Group::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

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
    T: 'static,
    C: FromIterator<T> + 'static,
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

            let mut prev_consumed = 0;
            let err = loop {
                if res.len() as u32 >= self.at_most {
                    break Ok(());
                };
                if ctx.is_term() {
                    // Termination at this point means that the inner parser
                    // won't be able to produce any more result.
                    // Not even fallback since those will be terminated sooner
                    // albe to produce any more
                    break Ok(());
                }
                let _guard = FallbackGuard::new(ctx.clone());
                let r = ctx.spawn(parent, &self.inner, true).await;

                // If parser produces a result without consuming anything
                // from the command line - this means it will continue to do
                // so indefinitely, so the idea is to stop consuming once we
                // reach such item (and throw it away - to deal with
                // switch().many() producing several true followed by false.
                //
                // At the same time it is still a good idea to return the
                // first such value... At least that's how v0.9 behaves

                let consumed = ctx.items_consumed.get() - prev_consumed;
                prev_consumed = ctx.items_consumed.get();
                match r {
                    Ok(t) => {
                        if consumed > 0 {
                            res.push(t);
                        } else {
                            // TODO - add test for that
                            if res.is_empty() {
                                res.push(t);
                            }
                            break Ok(());
                        }
                    }
                    Err(e) => {
                        if e.handle_with_fallback() && consumed == 0 {
                            break Ok(());
                        } else {
                            return Err(e);
                        }
                    }
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
                // problem: Unlike the pure applicative versions
                // we don't know what exactly was parsed in order to produce T even if it
                // came from a primitive parser directly. For now just display the guard message
                // and hope it was descriptive enough.
                // TODO - document this
                //
                // In the future we can explore posibilities of either provenance of T
                // or adding a guard a method on Positional/Named/Any that creates a custom
                Err(Error::new(crate::error::Message::GuardFailed {
                    message: self.message,
                }))
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
    T: 'static,
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
/// Registers interest in "no parse" scenario. With that in place if executor doesn't know
/// what parsers to run it would start terminating children of whaterver task that instantiates
/// this guard - this means the parser will receive an error.
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
            let prev_ctx = ctx.ctx_start.get();
            ctx.ctx_start.set(ctx.cur() as u32);
            let runner = crate::executor::Runner::new(ctx.clone());

            let r = runner.run_parser(&self.parser.0.parser, true);
            ctx.ctx_start.set(prev_ctx);
            println!("=========== Inner done with {:?}", r.is_ok());
            r
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let recursive = visitor.command(&self.names);
        if recursive {
            visitor.push_group(Group::Subparser);
            self.parser.0.parser.visit(visitor);
            visitor.pop_group();
        }
    }
}

impl<T, P> Parser<T> for Fallback<P, T>
where
    T: 'static + Clone,
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let _guard = FallbackGuard::new(ctx.clone());
            match self.inner.run(ctx.clone()).await {
                Ok(ok) => Ok(ok),
                Err(e) if e.handle_with_fallback() && ctx.items_consumed.get() == 0 => {
                    Ok(self.value.clone())
                }
                Err(e) => Err(e),
            }
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::visitor::Visitor<'a>) {
        visitor.push_group(Group::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

impl<T: 'static, P, F, E> Parser<T> for FallbackWith<P, T, F, E>
where
    P: Parser<T>,
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let _guard = FallbackGuard::new(ctx.clone());
            match self.inner.run(ctx.clone()).await {
                Ok(ok) => Ok(ok),
                Err(e) if e.handle_with_fallback() && ctx.items_consumed.get() == 0 => {
                    match (self.f)() {
                        Ok(ok) => Ok(ok),
                        Err(e) => Err(Error::parse_fail(e.to_string())),
                    }
                }
                Err(e) => Err(e),
            }
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::visitor::Visitor<'a>) {
        todo!()
    }
}

impl<T: 'static + Clone> Parser<T> for Pure<T> {
    fn run<'a>(&'a self, _ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move { Ok(self.value.clone()) })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::visitor::Visitor<'a>) {
        todo!();
    }
}

pub struct GroupHelp<P> {
    pub(crate) inner: P,
    pub(crate) title: &'static str,
}

impl<T: 'static, P: Parser<T>> Parser<T> for GroupHelp<P> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.inner.run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(Group::HelpGroup(self.title));
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Fallback<P, T> {
    pub(crate) inner: P,
    pub(crate) value: T,
}

impl<P, T> crate::Cx<Fallback<P, T>> {
    pub fn display_fallback(self) -> Self {
        self
    }
}

pub struct FallbackWith<P, T, F, E> {
    pub(crate) inner: P,
    pub(crate) f: F,
    pub(crate) ctx: PhantomData<(T, E)>,
}

pub struct Pure<T> {
    pub(crate) value: T,
}

pub struct Anything<T> {
    pub(crate) metavar: Metavar,
    pub(crate) check: Box<dyn Fn(OsOrStr) -> Option<T>>,
}

impl<T: 'static> Parser<T> for Anything<T> {
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(AnyFut {
            check: &self.check,
            metavar: self.metavar,
            ctx,
            task_id: None,
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        todo!()
    }
}

pub struct HideUsage<P> {
    pub(crate) inner: P,
}

impl<T: 'static, P> Parser<T> for HideUsage<P>
where
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: crate::Ctx<'a>) -> crate::Fragment<'a, T> {
        self.inner.run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        if visitor.mode() != Mode::Usage {
            self.inner.visit(visitor);
        }
    }
}

pub struct CustomHelp<P> {
    pub(crate) inner: P,
    pub(crate) custom: fn(&P, &mut dyn Visitor),
}

impl<T: 'static, P: Parser<T>> Parser<T> for CustomHelp<P> {
    fn run<'a>(&'a self, ctx: crate::Ctx<'a>) -> crate::Fragment<'a, T> {
        self.inner.run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        (self.custom)(&self.inner, visitor);
    }
}

// pub struct IsEmpty<T>(T);
//
// impl<T: 'static> Parser<T> for IsEmpty<T> {
//     fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, ()> {
//         todo!()
//     }
//
//     fn visit<'a>(&'a self, _visitor: &mut dyn Visitor<'a>) {}
// }

#[derive(Debug, Copy, Clone)]
pub(crate) enum HelpWrap {
    Version,
    Help,
    DetailedHelp,
}

pub(crate) fn help_and_version() -> impl Parser<HelpWrap> {
    use crate::*;
    let help = short('h')
        .long("help")
        .help("Prints help information")
        .req_flag(HelpWrap::Help)
        .hide_usage();
    let version = short('V')
        .long("version")
        .help("Prints version information")
        .req_flag(HelpWrap::Version)
        .hide_usage();
    construct!([help, version])
}

impl<T> Parser<T> for Box<dyn Parser<T>>
where
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.as_ref().run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.as_ref().visit(visitor)
    }
}

impl<T> Parser<T> for Rc<dyn Parser<T>>
where
    T: 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        self.as_ref().run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.as_ref().visit(visitor)
    }
}

pub struct Alt<T: 'static> {
    pub items: Vec<Box<dyn Parser<T>>>,
}

impl<T> Parser<T> for Alt<T>
where
    T: Clone + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        Box::pin(async move {
            let (_branch, id) = ctx.current_id();

            // Spawn a task for all the branches
            let handles = self
                .items
                .iter()
                .enumerate()
                .map(|(ix, p)| ctx.spawn(id.sum(ix as u32), p, false))
                .collect::<Vec<_>>();

            let mut fut = AltFuture { handles };
            // TODO: this should be some low priority error
            let mut res = Error::empty();

            // must collect results as they come. If

            // return first succesful result or the best error
            while !fut.handles.is_empty() {
                let hh = (&mut fut).await;
                res = match (res, hh) {
                    (ok @ Ok(_), _) | (Err(_), ok @ Ok(_)) => return ok,
                    (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
                }
            }
            res
        })
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Sum);
        for i in &self.items {
            i.visit(visitor);
        }
        visitor.pop_group();
    }
}

// [`downcast`] + [`hint`] are used to smuggle type of the field inside the [`Con`]
pub fn downcast<T: 'static>(_: PhantomData<T>, parser: &Box<dyn Any>) -> &Rc<dyn Parser<T>> {
    parser.downcast_ref().expect("Can't downcast")
}
pub fn hint<T: 'static>(_: impl Parser<T>) -> PhantomData<T> {
    PhantomData
}

pub struct Con<T> {
    pub visitors: Vec<Box<dyn Metavisit>>,
    pub parsers: Vec<Box<dyn Any>>,

    #[allow(clippy::type_complexity)] // And who's fault is that?
    pub run: Box<dyn for<'a> Fn(&'a [Box<dyn Any>], Ctx<'a>) -> Fragment<'a, T>>,
}

impl<T> Parser<T> for Con<T>
where
    T: std::fmt::Debug + 'static,
{
    fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, T> {
        (self.run)(&self.parsers, ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(Group::Prod);
        for v in &self.visitors {
            v.visit(visitor);
        }
        visitor.pop_group();
    }
}
