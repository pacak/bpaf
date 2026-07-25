use std::{marker::PhantomData, str::FromStr};

use crate::{
    adapters::PureWith,
    complete::{Completer, complete_value},
    error::MissingItem,
    os_str::parse_os_str,
};

use super::*;

/// Precursor for named parsers - flags, switches, etc
///
/// Create with [`short`], [`long`] or [`env()`]
#[derive(Debug, Clone)]
pub struct Named {
    pub(crate) names: Vec<Name<'static>>,
    pub(crate) env: Vec<&'static str>,
    pub(crate) help: Option<&'static str>,
}

impl Named {
    pub(crate) fn get_short_and_long(&self) -> (Option<char>, Option<&Cow<'static, str>>) {
        let mut short = None;
        let mut long = None;
        for name in &self.names {
            match name {
                Name::Short(s) => short = short.or(Some(*s)),
                Name::Long(l) => long = long.or(Some(l)),
            }
        }
        (short, long)
    }

    /// Iterate names with long names first
    pub(crate) fn all_names_long_first(&self) -> impl Iterator<Item = &Name<'static>> {
        self.names
            .iter()
            .filter(|n| matches!(n, Name::Long(_)))
            .chain(self.names.iter().filter(|n| matches!(n, Name::Short(_))))
    }

    /// Get [`Name`] with a preference to long
    pub(crate) fn name_long_or_short(&self) -> Option<Name<'static>> {
        match self.get_short_and_long() {
            (None, None) => None,
            (_, Some(l)) => Some(Name::Long(l.clone())),
            (Some(s), None) => Some(Name::Short(s)),
        }
    }

    fn get_env(&self) -> Option<OsString> {
        self.env.iter().find_map(std::env::var_os)
    }

    pub(crate) fn missing_item(&self, meta: Option<Metavar>) -> MissingItem {
        if let Some(name) = self.name_long_or_short() {
            MissingItem::Named { name, meta }
        } else if let Some(var_name) = self.env.first() {
            MissingItem::EnvVar { var_name }
        } else {
            unreachable!("Named starts either with name or an env var")
        }
    }
}

/// Match a named item with a short name: `-v` or `-b name`
pub fn short(name: char) -> Named {
    Named {
        names: vec![Name::Short(name)],
        env: Vec::new(),
        help: None,
    }
}

/// Match a named item with a long name: `--verbose` or `--bin name`
pub fn long<N>(name: N) -> Named
where
    N: Into<Cow<'static, str>>,
{
    Named {
        names: vec![Name::Long(name.into())],
        env: Vec::new(),
        help: None,
    }
}

pub fn env(name: &'static str) -> Named {
    Named {
        names: Vec::new(),
        env: vec![name],
        help: None,
    }
}

impl Named {
    pub fn short(mut self, name: char) -> Self {
        self.names.push(Name::Short(name));
        self
    }

    pub fn long(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.names.push(Name::Long(name.into()));
        self
    }

    pub fn env(mut self, name: &'static str) -> Self {
        self.env.push(name);
        self
    }

    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn switch(self) -> Flag<bool> {
        Flag {
            present: true,
            absent: Some(false),
            named: self,
        }
    }
    pub fn flag<T>(self, present: T, absent: T) -> Flag<T> {
        Flag {
            present,
            absent: Some(absent),
            named: self,
        }
    }

    pub fn req_flag<T>(self, present: T) -> Flag<T> {
        Flag {
            present,
            absent: None,
            named: self,
        }
    }

    pub fn argument<T>(self, metavar: &'static str) -> Argument<BasicArgument<T>> {
        Argument(BasicArgument {
            named: self,
            metavar: Metavar(metavar),
            ctx: PhantomData,
            adjacent: false,
        })
    }

    pub fn nest<T: 'static, P: Parser<Output = T> + 'static>(self, inner: P) -> Nested<T> {
        Nested {
            outer: Nest::Named(self.req_flag(())),
            inner: inner.into_box(),
        }
    }
}

pub enum Nest {
    Named(Flag<()>),
    Keyword(Keyword<()>),
}

/// A combination of two parsers - right after seeing `A` parse `B`
///
/// Deals with things like multi argument options or inline commands.
///
/// Start by creating the inner parser and a [`Named`] or [`Literal`] parser
/// for the trigger then call [`Named::nest`] or [`Literal::nest`]
pub struct Nested<T> {
    outer: Nest,
    inner: BoxParser<T>,
}

impl<T: 'static> Parser for Nested<T> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        match &self.outer {
            Nest::Named(named) => named.eval(ctx.clone()).await?,
            Nest::Keyword(kw) => kw.eval(ctx.clone()).await?,
        };
        let inner = ctx.fork(None, &self.inner);
        let scope_start = inner.shared.next_free.get();
        // cursor is now shared; save, advance past trigger, run inner, then restore
        let saved = ctx.cursor().get();
        ctx.cursor().set(saved + 1);

        let r = inner.run_inner_executor(true, &self.inner, None, scope_start);

        let end = ctx.cursor().get();
        let consumed = end - saved - 1;

        if let Err(e) = &r {
            let msg = match e {
                crate::Error::Problem(_, problem) => Some(problem.to_string()),
                crate::Error::Missing(m) => Some(m.to_string()),
                _ => None,
            };
            if let Some(msg) = msg {
                let cur = ctx.shared.current_task.borrow();
                ctx.shared
                    .conflicts
                    .borrow_mut()
                    .push(crate::Conflict::Caught {
                        pos: saved,
                        msg,
                        id: cur.id,
                        global: cur.global,
                    });
            }
        }

        ctx.consume(consumed);
        ctx.cursor().set(saved);

        match r {
            // the trigger value was there so we can't simply handle
            // missing value with `.fallback()` or similar
            Err(crate::Error::Missing(m)) => Err(crate::Error::Problem(
                saved,
                Problem::Dynamic { err: m.to_string() },
            )),
            other => other,
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.item(Item::Nested {
            outer: &self.outer,
            inner: &self.inner,
        });
    }
}

impl<T> Leaf for Nested<T> {}

/// Parser for a literal value such as `build`
///
/// Create from [`Literal::flag`], [`Literal::req_flag`], [`Literal::switch`]. Similar
/// to command parser but doesn't create a sub-parser with separate help
pub struct Keyword<T> {
    pub(crate) present: T,
    pub(crate) absent: Option<T>,
    pub(crate) named: Literal,
}

/// A precursor of the [`Keyword`] parser
pub struct Literal {
    pub(crate) help: Option<&'static str>,
    pub(crate) names: Vec<Lit<'static>>,
}

pub fn literal<N: Into<Cow<'static, str>>>(name: N) -> Literal {
    Literal {
        names: vec![Lit(Name::Long(name.into()))],
        help: None,
    }
}

impl Literal {
    pub fn short(mut self, name: char) -> Self {
        self.names.push(Lit(Name::Short(name)));
        self
    }
    pub fn long<N: Into<Cow<'static, str>>>(mut self, name: N) -> Self {
        self.names.push(Lit(Name::Long(name.into())));
        self
    }
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}

impl Literal {
    pub fn switch(self) -> Keyword<bool> {
        Keyword {
            named: self,
            present: true,
            absent: Some(false),
        }
    }
    pub fn req_flag<T: 'static>(self, value: T) -> Keyword<T> {
        Keyword {
            named: self,
            present: value,
            absent: None,
        }
    }
    pub fn flag<T: 'static>(self, present: T, absent: T) -> Keyword<T> {
        Keyword {
            named: self,
            present,
            absent: Some(absent),
        }
    }

    pub fn nest<T: 'static, P: Parser<Output = T> + 'static>(self, inner: P) -> Nested<T> {
        Nested {
            outer: Nest::Keyword(self.req_flag(())),
            inner: inner.into_box(),
        }
    }
}

impl Visited for () {
    fn vi<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}
impl<T: Clone + 'static> Parser for Keyword<T> {
    type Output = T;
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Command {
            names: &self.named.names,
            help: self.named.help,
            inner: &(),
        };
        visitor.item(item);
    }
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_literal(&self.named).await?;
        let value = if res.is_some() {
            &self.present
        } else if let Some(absent) = &self.absent {
            absent
        } else {
            return Err(Error::missing(MissingItem::Lit {
                value: self.named.names[0].clone(),
            }));
        };
        Ok(value.clone())
    }
}

impl<T> Leaf for Keyword<T> {}

/// Named Flag - detects presence or absence of `--flag`
///
/// Create it with [`Named::flag`], [`Named::req_flag`] or [`Named::switch`]
pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    pub(crate) named: Named,
}

impl<T: Clone + 'static> Parser for Flag<T> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_flag(&self.named).await?;
        if res || self.named.get_env().is_some() {
            Ok(self.present.clone())
        } else if let Some(absent) = &self.absent {
            Ok(absent.clone())
        } else {
            Err(Error::missing(self.named.missing_item(None)))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Flag { named: &self.named };
        if self.absent.is_some() {
            visitor.push_group(VisitGroup::Optional);
            visitor.item(item);
            visitor.pop_group();
        } else {
            visitor.item(item);
        }
    }
}
impl<T> Leaf for Flag<T> {}

impl<T> Flag<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.named.help = Some(help);
        self
    }
}

impl<T: Clone> Flag<T> {
    /// Change this flag to be the default one when used to select between a group of flags
    pub fn default(mut self) -> Self {
        self.absent = Some(self.present.clone());
        self
    }
}

/// Named argument. Parse `VALUE` in `--name VALUE` using [`FromStr`]
///
/// Create it with [`Named::argument`]
pub struct Argument<I>(I);

impl<I: ArgumentLike> Argument<I> {
    pub fn adjacent(self) -> Self {
        Argument(self.0.adjacent())
    }

    pub fn negative_lit(self) -> Argument<NegArgument<I>>
    where
        I::Output: FromStr + 'static,
        <I::Output as std::str::FromStr>::Err: std::fmt::Display,
    {
        Argument(self.0.negative_lit())
    }

    pub fn on_missing_value<F: Fn() -> Result<I::Output, String>>(
        self,
        handler: F,
    ) -> Argument<OnMissingValue<I, F>> {
        Argument(self.0.on_missing_value(handler))
    }
}

impl<P: Parser> Argument<P> {
    pub fn complete<C, F: Completer<C>>(self, c: F) -> WithComplete<Argument<P>, F, C> {
        WithComplete {
            inner: self,
            ctr: c,
            ctx: PhantomData,
        }
    }
}

impl<P: Parser> Parser for Argument<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        self.0.eval(ctx).await
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.visit(visitor)
    }
}

impl<P> Leaf for Argument<P> {}

pub struct BasicArgument<T> {
    named: Named,
    metavar: Metavar,
    ctx: PhantomData<T>,
    adjacent: bool,
}

pub struct OnMissingValue<P, F> {
    arg: P,
    handler: F,
}

impl<P, F> Parser for OnMissingValue<P, F>
where
    P: Parser,
    F: Fn() -> Result<P::Output, String>,
{
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let r = self.arg.eval(ctx.clone()).await;
        let Err(Error::Problem(
            i,
            Problem::WrongArgument {
                meta: _,
                name: _,
                value: None,
            },
        )) = r
        else {
            return r;
        };
        match (self.handler)() {
            Ok(v) => {
                ctx.consume(1);
                Ok(v)
            }
            Err(err) => Err(Error::Problem(i, Problem::Dynamic { err })),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.arg.visit(visitor)
    }
}

impl<T> Parser for BasicArgument<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_arg(&self.named, self.metavar).await?;
        if let Some(os) = res {
            if self.adjacent && ctx.shared.current_task.borrow().consumed == 2 {
                let cursor = ctx.cursor().get();
                let name = ctx.shared.args[cursor].to_string_lossy().into_owned();
                let value = ctx.shared.args[cursor + 1].to_string_lossy().into_owned();
                let problem = Problem::NotAdjacent { name, value };
                Err(Error::Problem(cursor, problem))
            } else {
                parse_os_str(os).map_err(|e| problem_at_pos(&ctx, e))
            }
        } else if let Some(os) = self.named.get_env() {
            parse_os_str(&os).map_err(|p| Error::Problem(u32::MAX, p))
        } else {
            Err(Error::missing(self.named.missing_item(Some(self.metavar))))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Arg {
            named: &self.named,
            meta: self.metavar,
        };
        visitor.item(item);
    }
}

pub struct NegArgument<P> {
    inner: P,
}

impl<P> Parser for NegArgument<P>
where
    P: Parser,
    P::Output: FromStr + 'static,
    <P::Output as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = P::Output;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        let r = self.inner.eval(ctx.clone()).await;
        let Err(Error::Problem(
            _,
            Problem::WrongArgument {
                meta: _,
                name: _,
                value: Some(value),
            },
        )) = &r
        else {
            return r;
        };

        match value.parse::<P::Output>() {
            Ok(v) => {
                ctx.consume(2);
                Ok(v)
            }
            Err(_) => r,
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

impl<T> Argument<BasicArgument<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.named.help = Some(help);
        self
    }
}

/// # complete for positional
impl<T: 'static> Positional<T> {
    pub fn complete<C, F: Completer<C>>(self, completer: F) -> WithComplete<Positional<T>, F, C>
    where
        Self: Sized,
    {
        WithComplete {
            inner: self,
            ctr: completer,
            ctx: PhantomData,
        }
    }
}
impl<T> Leaf for Positional<T> {}

impl<T> Positional<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}

pub struct WithComplete<P, F, C> {
    pub(crate) inner: P,
    pub(crate) ctr: F,
    pub(crate) ctx: PhantomData<C>,
}

impl<P, F, C> Parser for WithComplete<P, F, C>
where
    P: Parser,
    F: Completer<C>,
{
    type Output = P::Output;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        self.inner
            .eval(ctx.clone())
            .await
            .map_err(|err| complete_value(err, &self.ctr, &ctx))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

impl<P: Leaf, F, C> Leaf for WithComplete<P, F, C> {}

/// Implementation details for the `Argument` parser
///
/// To make it optimal in the most common case scenarios we need a trait
/// to deal with less common ones.
///
/// _Pay no attention to man behind the curtain._
pub trait ArgumentLike: Parser + Sized {
    fn adjacent(self) -> Self;

    fn negative_lit(self) -> NegArgument<Self> {
        NegArgument { inner: self }
    }

    fn on_missing_value<F: Fn() -> Result<Self::Output, String>>(
        self,
        handler: F,
    ) -> OnMissingValue<Self, F> {
        OnMissingValue { arg: self, handler }
    }
}

impl<T> ArgumentLike for BasicArgument<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn adjacent(mut self) -> Self {
        self.adjacent = true;
        self
    }
}

impl<P> ArgumentLike for NegArgument<P>
where
    P: ArgumentLike,
    P::Output: FromStr + 'static,
    <P::Output as std::str::FromStr>::Err: std::fmt::Display,
{
    fn adjacent(self) -> Self {
        NegArgument {
            inner: self.inner.adjacent(),
        }
    }
}

impl<P, F> ArgumentLike for OnMissingValue<P, F>
where
    P: ArgumentLike,
    F: Fn() -> Result<P::Output, String>,
{
    fn adjacent(self) -> Self {
        OnMissingValue {
            arg: self.arg.adjacent(),
            handler: self.handler,
        }
    }
}

/// A parser for positional items - parses operands using [`FromStr`]
pub struct Positional<T> {
    pub(crate) metavar: Metavar,
    pub(crate) help: Option<&'static str>,
    pub(crate) strict: bool,
    ctx: PhantomData<T>,
}

pub fn positional<T: 'static>(metavar: &'static str) -> Positional<T> {
    Positional {
        metavar: Metavar(metavar),
        ctx: PhantomData,
        help: None,
        strict: false,
    }
}

impl<T: 'static> Positional<T> {
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }
}

fn problem_at_pos(ctx: &Ctx, p: Problem) -> Error {
    Error::Problem(ctx.cursor().get(), p)
}

impl<T> Parser for Positional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_pos(self.help, self.metavar, self.strict).await?;

        let Some(os) = res else {
            let item = MissingItem::Pos { meta: self.metavar };
            return Err(Error::missing(item));
        };
        if self.strict && !ctx.strict_pos.get() {
            let cursor = ctx.cursor().get();
            let problem = Problem::NotStrict {
                metavar: self.metavar,
                string: os.to_string_lossy().into_owned(),
            };
            Err(Error::Problem(cursor, problem))
        } else {
            parse_os_str(os).map_err(|p| problem_at_pos(&ctx, p))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Positional {
            meta: self.metavar,
            help: self.help,
            strict: self.strict,
        };
        visitor.item(item);
    }
}

/// In case of conflicts it is excluded from "earlier running parser" wins
/// and it's position in the selection takes a priority. Including it in conflict resolution will
/// make it so any branch containing `pure` anywhere automatically advances.
pub fn pure<T: Clone + 'static>(value: T) -> Pure<T> {
    Pure { value }
}

/// A parser that produces a value `T` without consuming anything
///
/// Created with [`pure`]
pub struct Pure<T> {
    value: T,
}

impl<T: 'static + Clone> Parser for Pure<T> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let id = ctx.shared.current_task.borrow().id;
        let scope = Scope {
            start: id,
            end: Id(id.0 + 1),
        };
        ctx.early_exit.borrow_mut().insert(scope);
        r#yield().await;
        ctx.early_exit.borrow_mut().remove(&scope);

        Ok(self.value.clone())
    }

    fn visit<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}

pub fn pure_with<T, F, E>(act: F) -> PureWith<F>
where
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    PureWith { act }
}

pub fn leftovers<T>() -> Leftovers<T> {
    Leftovers { ctx: PhantomData }
}

pub struct Leftovers<T> {
    ctx: PhantomData<T>,
}

impl<T> Parser for Leftovers<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = Vec<T>;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let ctx = ctx.clone();

        let id = ctx.shared.current_task.borrow().id;
        let scope = Scope {
            start: id,
            end: Id(id.0 + 1),
        };
        ctx.early_exit.borrow_mut().insert(scope);
        r#yield().await;
        ctx.early_exit.borrow_mut().remove(&scope);
        let start = ctx.cursor().get();
        let leftovers = if matches!(
            &*ctx.shared.wakeup_reason.borrow(),
            Reason::Kill(KillReason::NoMatchingInput)
        ) {
            ctx.cursor().set(ctx.shared.args.len());
            ctx.consume(ctx.shared.args.len() - start);
            &ctx.shared.args.items[start as usize..]
        } else {
            return Ok(Vec::new());
        };

        let mut out = Vec::with_capacity(leftovers.len());

        for (pos, os) in (start..).zip(leftovers) {
            match parse_os_str(os) {
                Ok(v) => out.push(v),
                Err(problem) => return Err(Error::Problem(pos, problem)),
            }
        }

        Ok(out)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        if visitor.identify() == VKind::Usage {
            visitor.item(Item::Rendered { text: "..." });
        }
    }
}
