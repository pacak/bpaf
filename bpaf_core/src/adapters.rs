//! Adapters that implement functionality used by the [`Parser`] trait
use crate::{
    Error, Exit, Item, Kind, Lit, Name, ParseFailure, Parser, Problem, RawCtx, RcParser, Reason,
    Task, VKind, Visited,
    args::Args,
    complete::{complete_command, handle_subparser_complete},
    construct,
    error::MissingItem,
    traits::{Leaf, VisitGroup},
    utils::Vec1,
    r#yield,
};
use std::{borrow::Cow, marker::PhantomData};

pub struct Map<P, F, T, R> {
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

impl<P: Leaf, F, T, R> Leaf for Map<P, F, T, R> {}

impl<T: 'static, P: Parser<T>, F, R> Visited for Map<P, F, T, R> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct Optional<T> {
    pub(crate) inner: RcParser<T>,
}
impl<T: 'static> Parser<Option<T>> for Optional<T> {
    async fn run(&self, ctx: crate::Ctx) -> Result<Option<T>, Error> {
        // TODO - use scoped spawn, `Scope` and get rid of spawn_with_early_exit
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
    pub(crate) fn help_parser(&self) -> RcParser<crate::Extra> {
        use crate::{Extra, short};
        let help = short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(Extra::Help)
            .count()
            .parse(|c| match c {
                1 => Ok(crate::Extra::Help),
                2 => Ok(crate::Extra::LongHelp),
                _ => Err("not help"),
            });

        let mut alt = construct!([help]);
        if let Some(v) = self.version {
            let version = short('V')
                .long("version")
                .help("Prints version information")
                .req_flag(Extra::Version(v));

            alt.items.push(version.into_rc());
        }
        alt.hide_usage().into_rc()
    }
}

impl<T: 'static> OptionParser<T> {
    pub fn run_inner(&self, args: impl Into<Args>) -> Result<T, ParseFailure> {
        let ctx = RawCtx::new(args.into());
        Ok(self.run_in_ctx(false, ctx)?)
    }

    fn run_in_ctx(&self, lazy: bool, ctx: crate::Ctx) -> Result<T, Error> {
        let (handle, act) = ctx.make_raw_task(self.inner.clone());

        let no_input = ctx.args.len() == ctx.cursor.get();
        let info = ctx.make_child_info(Kind::Prod);
        let task = Task { act, info };
        ctx.add_task(task);
        let executor_res = ctx.execute(lazy, self, Some(&self.info));

        let res = handle.take();
        if self.info.fallback_to_usage && no_input && matches!(&res, Err(Error::Missing(_))) {
            let help = self.info.help_parser();
            return Err(Error::Final(ctx.render_help_for(self, &help, false)));
        }
        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1 + e2),
        }
    }

    pub fn command(self, name: impl Into<Cow<'static, str>>) -> Command<T> {
        let name = Lit(Name::Long(name.into()));
        Command {
            names: vec![name],
            inner: self,
            lazy: false,
        }
    }

    pub fn header(mut self, text: &'static str) -> Self {
        self.info.header = Some(text);
        self
    }
    pub fn descr(mut self, text: &'static str) -> Self {
        self.info.descr = Some(text);
        self
    }

    pub fn footer(mut self, text: &'static str) -> Self {
        self.info.footer = Some(text);
        self
    }

    pub fn version(mut self, text: &'static str) -> Self {
        self.info.version = Some(text);
        self
    }

    pub fn usage(mut self, text: &'static str) -> Self {
        self.info.usage = Some(text);
        self
    }

    pub fn fallback_to_usage(mut self) -> Self {
        self.info.fallback_to_usage = true;
        self
    }
}

impl<T: 'static> Visited for OptionParser<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::OptionParser {
            info: &self.info,
            inner: &self.inner,
        });

        self.inner.visit(visitor)
    }
}

impl<T: 'static> Command<T> {
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    pub fn long(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        let lit = Lit(Name::Long(name.into()));
        self.names.push(lit);
        self
    }

    pub fn short(mut self, name: char) -> Self {
        let lit = Lit(Name::Short(name));
        self.names.push(lit);
        self
    }
}

impl<T: 'static> Parser<T> for Command<T> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let res = ctx.parse_literal(&self.names).await;
        let res = res.map_err(|err| complete_command(&self.names, err));
        let Some(name) = res? else {
            let missing = MissingItem::Lit {
                value: self.names[0].clone(),
            };
            return Err(Error::Missing(Vec1::new(missing)));
        };

        let inner = ctx.fork(Some(name.to_string()));
        inner.cursor.update(|c| c + 1);
        let res = self.inner.run_in_ctx(self.lazy, inner.clone());
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

impl<T: 'static> Visited for Command<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let item = Item::Command {
            names: &self.names,
            info: &self.inner.info,
            inner: &self.inner.inner,
        };
        visitor.item(item);
    }
}

pub struct Count<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Visited for Count<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

impl<T: 'static> Parser<usize> for Count<T> {
    async fn run(&self, ctx: crate::Ctx) -> Result<usize, Error> {
        let many = Many {
            inner: self.inner.clone(),
        };
        Ok(many.run(ctx).await?.len())
    }
}

pub struct Many<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser<Vec<T>> for Many<T> {
    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<Vec<T>, Error>> {
        parse_many(self.inner.clone(), ctx, usize::MAX)
    }
}

async fn parse_many<T: 'static>(
    parser: RcParser<T>,
    ctx: crate::Ctx,
    max: usize,
) -> Result<Vec<T>, Error> {
    let mut res = Vec::new();
    let start = ctx.next_free.get();
    let mut consumed_before = 0;
    while matches!(&*ctx.wakeup_reason.borrow(), Reason::Pass | Reason::Push) {
        ctx.next_free.set(start);
        let (h, pair) = ctx.spawn_with_early_exit(parser.clone());

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
        if res.len() >= max {
            break;
        }
    }
    Ok(res)
}

impl<T: 'static> Visited for Many<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

pub struct Many1<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) message: &'static str,
}
impl<T: 'static> Parser<Vec<T>> for Many1<T> {
    async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
        let res = parse_many(self.inner.clone(), ctx, usize::MAX).await?;
        if res.is_empty() {
            Err(Error::Problem(u32::MAX, Problem::Static(self.message)))
        } else {
            Ok(res)
        }
    }
}
impl<T: 'static> Visited for Many1<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Command<T> {
    names: Vec<Lit<'static>>,
    inner: OptionParser<T>,
    lazy: bool,
}

pub struct Parse<T, P, F, E, R> {
    pub(crate) ctx: PhantomData<(T, F, E, R)>,
    pub(crate) inner: P,
    pub(crate) f: F,
}

impl<T: 'static, P, F, E, R> Visited for Parse<T, P, F, E, R>
where
    P: Parser<T>,
{
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

impl<T, P, F, E, R> Parser<R> for Parse<T, P, F, E, R>
where
    T: 'static,
    R: 'static,
    P: Parser<T>,
    F: Fn(T) -> Result<R, E>,
    E: ToString,
{
    async fn run(&self, ctx: crate::Ctx) -> Result<R, Error> {
        let t = self.inner.run(ctx.clone()).await?;
        match (self.f)(t) {
            Ok(r) => Ok(r),
            Err(error) => Err(Error::Problem(
                ctx.leaf_cursor(),
                Problem::Parse {
                    value: None,
                    error: error.to_string(),
                },
            )),
        }
    }
}

pub struct Guard<T, P, F> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) message: &'static str,
}

impl<T: 'static, F: Fn(&T) -> bool, P: Parser<T>> Parser<T> for Guard<T, P, F> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let r = self.inner.run(ctx.clone()).await?;

        if (self.check)(&r) {
            Ok(r)
        } else {
            Err(Error::Problem(
                ctx.leaf_cursor(),
                Problem::GuardFailed {
                    message: self.message,
                    range: ctx.leaf_consumed(),
                },
            ))
        }
    }
}

impl<F, T: 'static, P: Parser<T>> Visited for Guard<T, P, F> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct Hide<T, P> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
    pub(crate) only_usage: bool,
}

impl<T: 'static, P: Parser<T>> Parser<T> for Hide<T, P> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        self.inner.run(ctx).await
    }
}

impl<T, P: Leaf> Leaf for Hide<T, P> {}

impl<T: 'static, P: Parser<T>> Visited for Hide<T, P> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        if self.only_usage && visitor.identify() != VKind::Usage {
            self.inner.visit(visitor);
        }
    }
}

pub struct Fallback<T, P> {
    pub(crate) inner: P,
    pub(crate) value: T,
    pub(crate) value_str: Option<String>,
}

impl<T: 'static + Clone, P: Parser<T>> Parser<T> for Fallback<T, P> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.inner.run(ctx).await {
            Err(Error::Missing(_)) => Ok(self.value.clone()),
            otherwise => otherwise,
        }
    }
}

impl<T: 'static, P: Parser<T>> Visited for Fallback<T, P> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        if let Some(text) = self.value_str.as_deref()
            && matches!(visitor.identify(), crate::VKind::Help)
        {
            visitor.item(Item::Rendered { text, gr: None });
        }
        visitor.pop_group();
    }
}

impl<T: 'static + std::fmt::Debug, P> Fallback<T, P> {
    pub fn debug_fallback(mut self) -> Self {
        self.value_str = Some(format!("\t[default: {:?}]", self.value));
        self
    }
}

impl<T: 'static + std::fmt::Display, P> Fallback<T, P> {
    pub fn display_fallback(mut self) -> Self {
        self.value_str = Some(format!("\t[default: {}]", self.value));
        self
    }
}

impl<T, P> Fallback<T, P> {
    pub fn format_fallback(
        mut self,
        format: impl Fn(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Self {
        self.value_str = Some(format!("\t[default: {}]", DisplayWith(&self.value, format)));
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

impl<T: 'static, E: ToString, F: Fn() -> Result<T, E>> Parser<T> for PureWith<F> {
    async fn run(&self, _ctx: crate::Ctx) -> Result<T, Error> {
        (self.act)().map_err(|err| {
            Error::Problem(
                u32::MAX,
                Problem::Dynamic {
                    err: err.to_string(),
                },
            )
        })
    }
}

impl<F> Visited for PureWith<F> {
    fn visit<'a>(&'a self, _visitor: &mut dyn crate::Visitor<'a>) {}
}

pub struct Group<T, P> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
    pub(crate) title: &'static str,
    pub(crate) descr: Option<&'static str>,
}

impl<T: 'static, P: Parser<T>> Parser<T> for Group<T, P> {
    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<T, Error>> {
        self.inner.run(ctx)
    }
}

impl<T, P> Visited for Group<T, P>
where
    P: Visited,
{
    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::Section {
            title: self.title,
            descr: self.descr,
            inner: &self.inner,
        });
    }
}

pub struct WithOffset<T, P> {
    pub(crate) ctx: PhantomData<T>,
    pub(crate) inner: P,
}

impl<T, P> Parser<(Option<usize>, T)> for WithOffset<T, P>
where
    T: 'static,
    P: Parser<T> + Leaf,
{
    async fn run(&self, ctx: crate::Ctx) -> Result<(Option<usize>, T), Error> {
        let t = self.inner.run(ctx.clone()).await?;
        let consumed = ctx.current_task.borrow().consumed > 0;
        Ok((consumed.then_some(ctx.cursor.get()), t))
    }
}

impl<T: 'static, P: Visited> Visited for WithOffset<T, P> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

pub struct ThenExit<T, P> {
    pub(crate) inner: P,
    pub(crate) exit: Exit<T>,
}

impl<T: 'static, P: Parser<T>> Parser<T> for ThenExit<T, P> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        let a = self.inner.run(ctx.clone()).await;

        if a.is_ok() {
            self.exit.run(ctx).await
        } else {
            a
        }
    }
}

impl<T, P: Visited> Visited for ThenExit<T, P> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct OrExit<T, P> {
    pub(crate) inner: P,
    pub(crate) exit: Exit<T>,
}

impl<T: 'static, P: Parser<T>> Parser<T> for OrExit<T, P> {
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.inner.run(ctx.clone()).await {
            Err(_) => self.exit.run(ctx).await,
            ok => ok,
        }
    }
}

impl<T, P: Visited> Visited for OrExit<T, P> {
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
