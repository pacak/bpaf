//! Adapters that implement functionality used by the [`Parser`] trait
use crate::{
    Ctx, Error, Exit, Id, Item, Kind, Lit, Literal, Name, ParseFailure, Parser, Problem, RawCtx,
    RcParser, Scope, VKind, Visited,
    args::Args,
    complete::handle_subparser_complete,
    error::MissingItem,
    info::*,
    traits::{Leaf, VisitGroup},
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
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<R, crate::Error> {
        let t = self.inner.eval(ctx).await?;
        Ok((self.map)(t))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
impl<P: Leaf, F, R> Leaf for Map<P, F, R> {}

pub(crate) enum Optionality<T> {
    /// Value was produced by parsing something
    Parsed(T),
    /// Value was produced with no changes to the arguments
    Summoned(T),
    /// Error indicates that an item is missing and it CAN be caught
    Missing(Error),
    /// Some other error - it can't be caught
    Failed(Error),
}

#[inline(always)]
pub(crate) async fn optional<'p, T: 'static>(
    ctx: crate::Ctx<'p>,
    parser: &'p impl Parser<Output = T>,
    restore_to: &mut Option<u32>,
) -> Optionality<T> {
    let before = ctx.shared.current_task.borrow().consumed;
    let (handle, scope) = ctx.scoped_spawn(parser, Kind::Sum);
    if let Some(restore) = restore_to {
        let after = ctx.shared.next_free.get();
        assert!(
            after <= *restore,
            "scoped_spawn allocated IDs {after} past restore target {}",
            *restore
        );
        ctx.shared.next_free.set(*restore);
    }
    ctx.early_exit.borrow_mut().insert(scope);
    ctx.wait_for_children().await;
    ctx.early_exit.borrow_mut().remove(&scope);
    let stalled = ctx.shared.current_task.borrow().consumed == before;
    match handle.take() {
        Ok(v) if stalled => Optionality::Summoned(v),
        Ok(v) => Optionality::Parsed(v),
        Err(Error::Missing(i)) => {
            if stalled {
                Optionality::Missing(Error::Missing(i))
            } else {
                Optionality::Failed(Error::Problem(
                    ctx.cursor().get(),
                    Problem::Dynamic { err: i.to_string() },
                ))
            }
        }
        Err(e) => Optionality::Failed(e),
    }
}

pub struct Optional<P> {
    pub(crate) inner: P,
    pub(crate) catch: bool,
}
impl<P: Parser> Parser for Optional<P> {
    type Output = Option<P::Output>;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Option<P::Output>, Error> {
        let mut none = None;
        match optional(ctx.clone(), &self.inner, &mut none).await {
            Optionality::Parsed(v) | Optionality::Summoned(v) => Ok(Some(v)),
            Optionality::Missing(_) => Ok(None),
            Optionality::Failed(e) if self.catch => {
                ctx.shared.current_task.borrow_mut().consumed = 0;
                if let crate::Error::Problem(_, ref problem) = e {
                    let pos = ctx.cursor().get();
                    let msg = problem.to_string();
                    let cur = ctx.shared.current_task.borrow();
                    ctx.shared
                        .conflicts
                        .borrow_mut()
                        .push(crate::Conflict::Caught {
                            pos,
                            msg,
                            id: cur.id,
                            global: cur.global,
                        });
                }
                Ok(None)
            }
            Optionality::Failed(e) => Err(e),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}
impl<P: Leaf> Leaf for Optional<P> {}

impl<P: Leaf> Optional<P> {
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

/// A top level parser with associated description
///
/// Created with [`Parser::to_options`]
pub struct OptionParser<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) info: Info,
}

impl<T: 'static> OptionParser<T> {
    pub fn run_inner(&self, args: impl Into<Args>) -> Result<T, ParseFailure> {
        let custom = match self.info.custom.as_deref() {
            Some(custom) => custom,
            None => &Custom::default(),
        };
        let help_and_version = custom.create(self.info.version).into_box();

        let mut args = args.into();
        args.check_complete()?;

        let ctx = RawCtx::new(&args, &help_and_version, self);
        Ok(self.run_in_ctx(false, ctx)?)
    }

    pub fn run(&self) -> T {
        match self.run_inner(std::env::args_os()) {
            Ok(r) => r,
            Err(e) => {
                let mut cs = self.info.get_colorscheme();
                if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                    cs = None;
                }
                let (code, text) = match e {
                    ParseFailure::Stdout(raw) => (0, raw.with_cs(cs)),
                    ParseFailure::Stderr(raw) => (1, raw.with_cs(cs)),
                    ParseFailure::Console(o) => (0, o),
                };
                print!("{text}");
                std::process::exit(code)
            }
        }
    }

    fn run_in_ctx<'p>(&'p self, lazy: bool, ctx: crate::Ctx<'p>) -> Result<T, Error> {
        let scope_start = ctx.shared.next_free.get();
        let no_input = ctx.shared.args.len() == ctx.cursor().get();
        let result = ctx.run_inner_executor(lazy, &self.inner, Some(&self.info), scope_start);
        if self.info.fallback_to_usage && no_input && matches!(&result, Err(Error::Missing(_))) {
            Err(Error::Final(ParseFailure::Stdout(
                crate::visitors::help::render_help(
                    self,
                    Some(ctx.shared.help_and_version),
                    &ctx.path,
                    false,
                ),
            )))
        } else {
            result
        }
    }

    pub fn command(self, name: impl Into<Cow<'static, str>>) -> Command<T> {
        let name = Lit(Name::Long(name.into()));
        Command {
            names: Literal {
                names: vec![name],
                help: self.info.descr,
            },
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
    }
}

impl<T: 'static> Command<T> {
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    pub fn long(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        let lit = Lit(Name::Long(name.into()));
        self.names.names.push(lit);
        self
    }

    pub fn short(mut self, name: char) -> Self {
        let lit = Lit(Name::Short(name));
        self.names.names.push(lit);
        self
    }
}

impl<T: 'static> Parser for Command<T> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_literal(&self.names).await?;
        if res.is_none() {
            let missing = MissingItem::Cmd {
                _value: self.names.names[0].clone(),
            };
            return Err(Error::missing(missing));
        };
        // TODO - can use value returned in `res` here
        let Some(Lit(Name::Long(n))) = self.names.names.first().as_ref() else {
            unreachable!("For commands first name should always be a long one, by construction");
        };

        let inner = ctx.fork(Some(n.as_ref()), &self.inner);
        // cursor is now shared between ctx and inner;
        // save position, advance past trigger, run inner, then restore
        let saved = ctx.cursor().get();
        ctx.cursor().set(saved + 1);
        let res = self.inner.run_in_ctx(self.lazy, inner.clone());
        ctx.consume(ctx.cursor().get() - saved - 1);
        ctx.cursor().set(saved);
        let res = res.map_err(handle_subparser_complete);
        res.map_err(Error::finalize_problems)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let item = Item::Command {
            names: &self.names.names,
            help: self.names.help,
            inner: &self.inner,
        };
        visitor.item(item);
    }
}

impl<T> Leaf for Command<T> {}

pub struct Command<T> {
    names: Literal,
    inner: OptionParser<T>,
    lazy: bool,
}

impl<T> Command<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.names.help = Some(help);
        self
    }
}

pub struct Parse<P, F, E, R> {
    pub(crate) ctx: PhantomData<(F, E, R)>,
    pub(crate) inner: P,
    pub(crate) f: F,
}

impl<P: Leaf, F, E, R> Leaf for Parse<P, F, E, R> {}

impl<P, F, E, R> Parser for Parse<P, F, E, R>
where
    R: 'static,
    P: Parser,
    F: Fn(P::Output) -> Result<R, E>,
    E: ToString,
{
    type Output = R;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<R, Error> {
        let t = self.inner.eval(ctx.clone()).await?;
        match (self.f)(t) {
            Ok(r) => Ok(r),
            Err(error) => Err(Error::Problem(
                ctx.leaf_cursor(),
                Problem::Parse {
                    value: ctx
                        .current_value
                        .borrow()
                        .as_ref()
                        .map(|v| v.to_string_lossy().into_owned()),
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
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        let r = self.inner.eval(ctx.clone()).await?;

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
        self.inner.visit(visitor)
    }
}

pub struct Hide<P> {
    pub(crate) inner: P,
    pub(crate) only_usage: bool,
}

impl<P: Parser> Parser for Hide<P> {
    type Output = P::Output;
    fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> impl Future<Output = Result<P::Output, Error>> {
        self.inner.eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        if self.only_usage && visitor.identify() != VKind::Usage {
            self.inner.visit(visitor);
        }
    }
}

impl<P: Leaf> Leaf for Hide<P> {}
pub struct Fallback<T: 'static, P> {
    pub(crate) inner: P,
    pub(crate) value: T,
    pub(crate) pprint: Option<fn(&T) -> String>,
}

impl<T, P: Leaf> Leaf for Fallback<T, P> {}
impl<T: 'static + Clone, P: Parser<Output = T>> Parser for Fallback<T, P> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<T, Error> {
        match self.inner.eval(ctx).await {
            Err(Error::Missing(_)) => Ok(self.value.clone()),
            otherwise => otherwise,
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        if let Some(f) = self.pprint
            && matches!(visitor.identify(), crate::VKind::Help)
        {
            let text = f(&self.value);
            visitor.item(Item::Rendered {
                text: &text,
                gr: None,
            });
        }
        visitor.pop_group();
    }
}

impl<T: 'static + std::fmt::Debug, P> Fallback<T, P> {
    pub fn debug_fallback(mut self) -> Self {
        self.pprint = Some(|value| format!("\t[default: {:?}]", value));
        self
    }
}

impl<T: 'static + std::fmt::Display, P> Fallback<T, P> {
    pub fn display_fallback(mut self) -> Self {
        self.pprint = Some(|value| format!("\t[default: {}]", value));
        self
    }
}

impl<T, P> Fallback<T, P> {
    /// Show a fallback value in a help using custom format
    /// `.format_fallback(|v| format!("{v}"))`
    pub fn format_fallback(mut self, format: fn(&T) -> String) -> Self {
        self.pprint = Some(format);
        self
    }
}

pub struct FallbackStr<P> {
    pub(crate) inner: P,
    pub(crate) fallback: &'static str,
    pub(crate) pprint: Option<fn(&str) -> String>,
}
impl<P: Leaf> Leaf for FallbackStr<P> {}

impl<P> Parser for FallbackStr<P>
where
    P: Parser,
    P::Output: std::str::FromStr,
    <P::Output as std::str::FromStr>::Err: ToString,
{
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        match self.inner.eval(ctx.clone()).await {
            Err(Error::Missing(_)) => match self.fallback.parse() {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::Problem(
                    ctx.leaf_cursor(),
                    Problem::Dynamic { err: e.to_string() },
                )),
            },
            otherwise => otherwise,
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        if let Some(f) = self.pprint
            && matches!(visitor.identify(), crate::VKind::Help)
        {
            let text = f(self.fallback);
            visitor.item(Item::Rendered {
                text: &text,
                gr: None,
            });
        }
        visitor.pop_group();
    }
}

impl<P> FallbackStr<P> {
    pub fn display_fallback(mut self) -> Self {
        self.pprint = Some(|value| format!("\t[default: {}]", value));
        self
    }

    pub fn debug_fallback(mut self) -> Self {
        self.pprint = Some(|value| format!("\t[default: {:?}]", value));
        self
    }

    pub fn format_fallback(mut self, format: fn(&str) -> String) -> Self {
        self.pprint = Some(format);
        self
    }
}

pub struct FallbackWith<P, F, E> {
    pub(crate) inner: P,
    pub(crate) fallback: F,
    pub(crate) ctx: PhantomData<E>,
}

impl<P: Leaf, F, E> Leaf for FallbackWith<P, F, E> {}
impl<P, F, E> Parser for FallbackWith<P, F, E>
where
    P: Parser,
    F: Fn() -> Result<P::Output, E>,
    E: ToString,
{
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        match self.inner.eval(ctx.clone()).await {
            Err(Error::Missing(_)) => match (self.fallback)() {
                Ok(v) => Ok(v),

                Err(e) => Err(Error::Problem(
                    ctx.leaf_cursor(),
                    Problem::Dynamic { err: e.to_string() },
                )),
            },
            otherwise => otherwise,
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

/// A parser that produces a value by calling a closure
pub struct PureWith<F> {
    pub(crate) act: F,
}

impl<T: 'static, E: ToString + 'static, F: Fn() -> Result<T, E>> Parser for PureWith<F> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<T, Error> {
        let id = ctx.shared.current_task.borrow().id;
        let scope = Scope {
            start: id,
            end: Id(id.0 + 1),
        };
        ctx.early_exit.borrow_mut().insert(scope);
        r#yield().await;
        ctx.early_exit.borrow_mut().remove(&scope);
        (self.act)().map_err(|err| {
            let problem = Problem::Dynamic {
                err: err.to_string(),
            };
            Error::Problem(ctx.cursor().get(), problem)
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
    fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> impl Future<Output = Result<P::Output, Error>> {
        self.inner.eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.item(Item::Section {
            title: self.title,
            descr: self.descr,
            inner: &self.inner,
        });
    }
}

impl<P: Leaf> Leaf for Group<P> {}
pub struct WithOffset<P> {
    pub(crate) inner: P,
}

impl<P> Parser for WithOffset<P>
where
    P: Parser + Leaf,
{
    type Output = (Option<u32>, P::Output);
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<(Option<u32>, P::Output), Error> {
        let t = self.inner.eval(ctx.clone()).await?;
        let consumed = ctx.shared.current_task.borrow().consumed > 0;
        Ok((consumed.then_some(ctx.cursor().get()), t))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}
impl<P: Leaf> Leaf for WithOffset<P> {}

pub struct ThenExit<T, P: Parser> {
    pub(crate) inner: P,
    pub(crate) exit: Box<dyn Fn(P::Output) -> Exit<T>>,
}

impl<T: 'static, P: Parser> Parser for ThenExit<T, P> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<T, Error> {
        match self.inner.eval(ctx.clone()).await {
            Ok(o) => Err((self.exit)(o).to_error()),
            Err(e) => Err(e),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
impl<T, P: Parser + Leaf> Leaf for ThenExit<T, P> {}
pub struct OrExit<P: Parser> {
    pub(crate) inner: P,
    pub(crate) exit: Box<dyn Fn(ParseFailure) -> Exit<P::Output>>,
}

impl<P: Parser> Parser for OrExit<P> {
    type Output = P::Output;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        match self.inner.eval(ctx.clone()).await {
            Err(e) => Err((self.exit)(ParseFailure::from(e)).to_error()),
            ok => ok,
        }
    }
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}
impl<P: Parser + Leaf> Leaf for OrExit<P> {}

/// Run inner parser before and separately of everything else.
///
/// Multiple anchored parsers run sequentially, first encountered - first run
pub struct AnchorStart<P> {
    pub(crate) inner: P,
}

impl<P: Parser> Parser for AnchorStart<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<P::Output, Error> {
        let inner = ctx.fork(None, &self.inner);
        let scope_start = inner.shared.next_free.get();
        let saved = ctx.cursor().get();

        let r = inner.run_inner_executor(true, &self.inner, None, scope_start);
        if r.is_err() {
            ctx.cursor().set(saved);
        }
        r
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        let is_err = match visitor.identify() {
            VKind::Error => false,
            VKind::Usage | VKind::Help | VKind::Custom => true,
        };
        if is_err {
            // not really improving the error message
            self.inner.visit(visitor);
        }
    }
}

/// Mark this parser and all its descendants as using the global trigger set.
///
/// Global triggers persist across all executor scopes (including forked
/// contexts from subcommands) and are shared by all running executors.
pub struct Global<P> {
    pub(crate) inner: P,
}

impl<P: Parser> Parser for Global<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        ctx.shared.current_task.borrow_mut().global = true;
        let f = ctx.spawn(Kind::Prod, &self.inner);
        ctx.wait_for_children().await;
        f.take()
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Global);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}
