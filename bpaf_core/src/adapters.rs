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

pub struct Map<P, F, R> {
    pub(crate) inner: P,
    pub(crate) map: F,
    pub(crate) ctx: PhantomData<R>,
}

impl<T: 'static, P: Parser<Output = T>, R: 'static, F: Fn(T) -> R> Parser for Map<P, F, R> {
    type Output = R;
    async fn run(&self, ctx: crate::Ctx) -> Result<R, crate::Error> {
        let t = self.inner.run(ctx).await?;
        Ok((self.map)(t))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
impl<P: Leaf, F, R> Leaf for Map<P, F, R> {}

pub struct Optional<T> {
    pub(crate) inner: RcParser<T>,
}
impl<T: 'static> Parser for Optional<T> {
    type Output = Option<T>;
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

/// A top level parser with associated description
///
/// Created with [`Parser::to_options`]
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
    fn vi<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
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

impl<T: 'static> Parser for Command<T> {
    type Output = T;
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let item = Item::Command {
            names: &self.names,
            info: &self.inner.info,
            inner: &self.inner,
        };
        visitor.item(item);
    }
}

pub struct Count<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Count<T> {
    type Output = usize;
    async fn run(&self, ctx: crate::Ctx) -> Result<usize, Error> {
        let many = Many {
            inner: self.inner.clone(),
        };
        Ok(many.run(ctx).await?.len())
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

pub struct Many<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Many<T> {
    type Output = Vec<T>;

    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<Vec<T>, Error>> {
        parse_many(self.inner.clone(), ctx, usize::MAX)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
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

pub struct Many1<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) message: &'static str,
}
impl<T: 'static> Parser for Many1<T> {
    type Output = Vec<T>;
    async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
        let res = parse_many(self.inner.clone(), ctx, usize::MAX).await?;
        if res.is_empty() {
            Err(Error::Problem(u32::MAX, Problem::Static(self.message)))
        } else {
            Ok(res)
        }
    }

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

pub struct Parse<P, F, E, R> {
    pub(crate) ctx: PhantomData<(F, E, R)>,
    pub(crate) inner: P,
    pub(crate) f: F,
}

impl<P, F, E, R> Parser for Parse<P, F, E, R>
where
    R: 'static,
    P: Parser,
    F: Fn(P::Output) -> Result<R, E>,
    E: ToString,
{
    type Output = R;
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

pub struct Guard<P, F> {
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) message: &'static str,
}

impl<P: Parser, F: Fn(&P::Output) -> bool> Parser for Guard<P, F> {
    type Output = P::Output;
    async fn run(&self, ctx: crate::Ctx) -> Result<Self::Output, Error> {
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

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct Hide<P> {
    pub(crate) inner: P,
    pub(crate) only_usage: bool,
}

impl<P: Parser> Parser for Hide<P> {
    type Output = P::Output;
    async fn run(&self, ctx: crate::Ctx) -> Result<P::Output, Error> {
        self.inner.run(ctx).await
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        if self.only_usage && visitor.identify() != VKind::Usage {
            self.inner.visit(visitor);
        }
    }
}

impl<P: Leaf> Leaf for Hide<P> {}
pub struct Fallback<T, P> {
    pub(crate) inner: P,
    pub(crate) value: T,
    pub(crate) value_str: Option<String>,
}

impl<T: 'static + Clone, P: Parser<Output = T>> Parser for Fallback<T, P> {
    type Output = T;
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.inner.run(ctx).await {
            Err(Error::Missing(_)) => Ok(self.value.clone()),
            otherwise => otherwise,
        }
    }

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
    /// Show a fallback value in a help using custom call to [`write!`]
    /// `.format_fallback(|v, f| write!(f, "{v}"))`
    pub fn format_fallback(
        mut self,
        format: impl Fn(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Self {
        self.value_str = Some(format!("\t[default: {}]", DisplayWith(&self.value, format)));
        self
    }
}

/// Helper for [`Fallback`] that allows using a custom formatter
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

/// A parser that produces a value by calling a closure
pub struct PureWith<F> {
    pub(crate) act: F,
}

impl<T: 'static, E: ToString, F: Fn() -> Result<T, E>> Parser for PureWith<F> {
    type Output = T;
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

    fn visit<'a>(&'a self, _visitor: &mut dyn crate::Visitor<'a>) {}
}

pub struct Group<P> {
    pub(crate) inner: P,
    pub(crate) title: &'static str,
    pub(crate) descr: Option<&'static str>,
}

impl<P: Parser> Parser for Group<P> {
    type Output = P::Output;
    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<P::Output, Error>> {
        self.inner.run(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::Section {
            title: self.title,
            descr: self.descr,
            inner: &self.inner,
        });
    }
}

pub struct WithOffset<P> {
    pub(crate) inner: P,
}

impl<P> Parser for WithOffset<P>
where
    P: Parser + Leaf,
{
    type Output = (Option<usize>, P::Output);
    async fn run(&self, ctx: crate::Ctx) -> Result<(Option<usize>, P::Output), Error> {
        let t = self.inner.run(ctx.clone()).await?;
        let consumed = ctx.current_task.borrow().consumed > 0;
        Ok((consumed.then_some(ctx.cursor.get()), t))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

pub struct ThenExit<T, P: Parser> {
    pub(crate) inner: P,
    pub(crate) exit: Box<dyn Fn(P::Output) -> Exit<T>>,
}

impl<T: 'static, P: Parser> Parser for ThenExit<T, P> {
    type Output = T;
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.inner.run(ctx.clone()).await {
            Ok(o) => Err((self.exit)(o).to_error()),
            Err(e) => Err(e),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

pub struct OrExit<T, P> {
    pub(crate) inner: P,
    pub(crate) exit: Exit<T>,
}

impl<T: 'static, P: Parser<Output = T>> Parser for OrExit<T, P> {
    type Output = T;
    async fn run(&self, ctx: crate::Ctx) -> Result<T, Error> {
        match self.inner.run(ctx.clone()).await {
            Err(_) => self.exit.run(ctx).await,
            ok => ok,
        }
    }
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
