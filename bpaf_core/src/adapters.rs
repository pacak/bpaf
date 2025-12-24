//! Adapters that implement functionality used by the [`Parser`] trait

use crate::{
    Bp, Error, Item, Kind, MissingItem, ParseFailure, Parser, Problem, RawCtx, RcParser, Reason,
    Task, Visited,
    args::Args,
    complete::{complete_command, handle_subparser_complete},
    traits::VisitGroup,
    utils::Vec1,
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
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct OptionParser<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) info: Info,
}

#[derive(Debug, Default)]
pub struct Info {
    pub header: Option<&'static str>,
    pub descr: Option<&'static str>,
    pub footer: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub version: Option<&'static str>,
    pub fallback_to_usage: bool,
}

impl Info {
    pub(crate) fn help_parser(&self) -> Bp<RcParser<crate::Extra>> {
        use crate::{Alt, Extra, short};
        let help = short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(Extra::Help)
            .into_rc();
        let mut alt = Alt { items: vec![help] };
        if let Some(v) = self.version {
            let version = short('V')
                .long("version")
                .help("Prints version information")
                .req_flag(Extra::Version(v))
                .into_rc();
            alt.items.push(version);
        }
        alt.into_rc()
    }
}

impl<T: 'static> Bp<OptionParser<T>> {
    pub fn run_inner(&self, args: impl Into<Args>) -> Result<T, ParseFailure> {
        let ctx = RawCtx::new(args.into());
        Ok(self.run_in_ctx(None, ctx)?)
    }

    fn run_in_ctx(&self, cmd: Option<&str>, ctx: crate::Ctx) -> Result<T, Error> {
        let (handle, act) = ctx.make_raw_task(Bp(self.0.inner.clone()));

        let no_input = ctx.args.len() == ctx.cursor.get();
        let info = ctx.make_child_info(Kind::Prod);
        let task = Task { act, info };
        ctx.add_task(task);
        let executor_res = ctx.execute(self, Some(&self.0.info));

        let res = handle.take();
        if self.0.info.fallback_to_usage && no_input && matches!(&res, Err(Error::Missing(_))) {
            let help = self.0.info.help_parser();
            return Err(Error::Final(ctx.render_help_for(self, &help)));
        }
        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1 + e2),
        }
    }

    pub fn command(self, name: impl Into<Cow<'static, str>>) -> Bp<Command<T>> {
        Bp(Command {
            names: vec![name.into()],
            inner: self,
        })
    }

    pub fn header(mut self, text: &'static str) -> Self {
        self.0.info.header = Some(text);
        self
    }
    pub fn descr(mut self, text: &'static str) -> Self {
        self.0.info.descr = Some(text);
        self
    }

    pub fn footer(mut self, text: &'static str) -> Self {
        self.0.info.footer = Some(text);
        self
    }

    pub fn version(mut self, text: &'static str) -> Self {
        self.0.info.version = Some(text);
        self
    }

    pub fn usage(mut self, text: &'static str) -> Self {
        self.0.info.usage = Some(text);
        self
    }

    pub fn fallback_to_usage(mut self) -> Self {
        self.0.info.fallback_to_usage = true;
        self
    }
}

impl<T: 'static> Visited for Bp<OptionParser<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::OptionParser {
            info: &self.0.info,
            inner: &self.0.inner,
        });

        self.0.inner.visit(visitor)
    }
}

impl<T: 'static> Parser<T> for Bp<Command<T>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let res = ctx.parse_literal(&self.0.names).await;
        let res = res.map_err(|err| complete_command(&self.0.names, err));
        let Some(name) = res? else {
            let missing = MissingItem::Lit {
                value: self.0.names[0].clone(),
            };
            return Err(Error::Missing(Vec1::new(missing)));
        };

        let inner = ctx.fork(Some(name.clone()));
        inner.cursor.update(|c| c + 1);
        let res = self.0.inner.run_in_ctx(Some(&name), inner.clone());
        ctx.consume((inner.cursor.get() - ctx.cursor.get()) as u32);
        let res = res.map_err(handle_subparser_complete);
        res.map_err(Error::finalize_problems)
    }
}

impl Error {
    fn finalize_problems(self) -> Error {
        match &self {
            Error::Missing(..) | Error::CompReply(..) | Error::CompReq(..) | Error::Problem(..) => {
                ParseFailure::from(self).into()
            }
            Error::Final(..) | Error::Silent(_) => self,
        }
    }
}

impl<T: 'static> Visited for Bp<Command<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let item = Item::Command {
            names: &self.0.names,
            info: &self.0.inner.0.info,
            inner: &self.0.inner.0.inner,
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
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
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
            Err(Error::Problem(u32::MAX, Problem::Static(self.0.message)))
        } else {
            Ok(res)
        }
    }
}
impl<T: 'static> Visited for Bp<Many1<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        self.0.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Command<T> {
    names: Vec<Cow<'static, str>>,
    inner: Bp<OptionParser<T>>,
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
            Err(Error::Problem(
                ctx.leaf_cursor(),
                Problem::GuardFailed {
                    message: self.0.message,
                    range: ctx.leaf_consumed(),
                },
            ))
        }
    }
}

impl<F, T: 'static, P: Parser<T>> Visited for Bp<Guard<T, P, F>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.0.inner.visit(visitor);
    }
}

pub struct Hide<T, P> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
}

impl<T: 'static, P: Parser<T>> Parser<T> for Bp<Hide<T, P>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        self.0.inner.run(ctx).await
    }
}

impl<T: 'static, P: Parser<T>> Visited for Bp<Hide<T, P>> {
    fn visit<'a>(&'a self, _visitor: &mut dyn crate::Visitor<'a>) {}
}

pub struct Fallback<T, P> {
    pub(crate) inner: P,
    pub(crate) value: T,
    pub(crate) value_str: Option<String>,
}

impl<T: 'static + Clone, P: Parser<T>> Parser<T> for Bp<Fallback<T, P>> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.0.inner.run(ctx).await {
            Err(Error::Missing(_)) => Ok(self.0.value.clone()),
            otherwise => otherwise,
        }
    }
}

impl<T: 'static, P: Parser<T>> Visited for Bp<Fallback<T, P>> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.0.inner.visit(visitor);
        if let Some(text) = self.0.value_str.as_deref()
            && matches!(visitor.identify(), crate::VKind::Help)
        {
            visitor.item(Item::Rendered { text });
        }
        visitor.pop_group();
    }
}

impl<T: 'static + std::fmt::Debug, P> Bp<Fallback<T, P>> {
    pub fn debug_fallback(mut self) -> Self {
        self.0.value_str = Some(format!("\t[default: {:?}]", self.0.value));
        self
    }
}

impl<T: 'static + std::fmt::Display, P> Bp<Fallback<T, P>> {
    pub fn display_fallback(mut self) -> Self {
        self.0.value_str = Some(format!("\t[default: {}]", self.0.value));
        self
    }
}

impl<T, P> Bp<Fallback<T, P>> {
    pub fn format_fallback(
        mut self,
        format: impl Fn(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Self {
        self.0.value_str = Some(format!(
            "\t[default: {}]",
            DisplayWith(&self.0.value, format)
        ));
        self
    }
}

struct DisplayWith<'a, T, F>(&'a T, F);

impl<'a, T, F: Fn(&'a T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result> std::fmt::Display
    for DisplayWith<'a, T, F>
{
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value, display) = self;
        display(value, f)
    }
}

pub struct PureWith<F> {
    pub(crate) act: F,
}

impl<T: 'static, E: ToString, F: Fn() -> Result<T, E>> Parser<T> for Bp<PureWith<F>> {
    async fn run(&self, _ctx: crate::Ctx) -> Result<T, Error> {
        (self.0.act)().map_err(|err| {
            Error::Problem(
                u32::MAX,
                Problem::Dynamic {
                    err: err.to_string(),
                },
            )
        })
    }
}

impl<F> Visited for Bp<PureWith<F>> {
    fn visit<'a>(&'a self, _visitor: &mut dyn crate::Visitor<'a>) {}
}

pub struct Group<T, P> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
    pub(crate) title: &'static str,
    pub(crate) descr: Option<&'static str>,
}

impl<T: 'static, P: Parser<T>> Parser<T> for Bp<Group<T, P>> {
    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<T, Error>> {
        self.0.inner.run(ctx)
    }
}

impl<T, P> Visited for Bp<Group<T, P>>
where
    P: Visited,
{
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::Section {
            title: self.0.title,
            descr: self.0.descr,
            inner: &self.0.inner,
        });
    }
}
