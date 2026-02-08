use std::{marker::PhantomData, str::FromStr};

use crate::{
    adapters::PureWith,
    complete::{CompleteReply, StringCompleter, complete_command, complete_value},
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
}

/// Match a named item with a short name: `-v` or `-b name`
pub fn short(name: char) -> Named {
    Named {
        names: vec![name.into()],
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
        self.names.push(name.into());
        self
    }

    pub fn long(mut self, name: &'static str) -> Self {
        self.names.push(name.into());
        self
    }

    pub fn long_string(mut self, name: String) -> Self {
        self.names.push(name.into());
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

    pub fn argument<T>(self, metavar: &'static str) -> Argument<T> {
        Argument {
            named: self,
            metavar: Metavar(metavar),
            ctx: PhantomData,
            adjacent: false,
        }
    }

    pub fn nest<T: 'static, P: Parser<Output = T> + 'static>(self, inner: P) -> Nested<T> {
        Nested {
            outer: Nest::Named(self.req_flag(())),
            inner: inner.into_rc(),
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
    inner: RcParser<T>,
}

impl<T: 'static> Parser for Nested<T> {
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        match &self.outer {
            Nest::Named(named) => named.eval(ctx.clone()).await?,
            Nest::Keyword(kw) => kw.eval(ctx.clone()).await?,
        };
        let inner = ctx.fork(None);
        inner.cursor.update(|c| c + 1);

        let (out, handle) = make_chan();
        let act = inner.make_act(out, &self.inner);
        let info = inner.make_child_info(Kind::Prod);
        inner.add_task(Task { act, info });
        let executor_res = inner.execute(true, &self.inner, None);
        let res = handle.take();
        ctx.consume(inner.cursor.get() - 1 - ctx.cursor.get());

        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1 + e2),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.item(Item::Nested {
            outer: &self.outer,
            inner: &self.inner,
        });
    }
}

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
    pub(crate) info: Info,
    pub(crate) names: Vec<Lit<'static>>,
}

pub fn literal<N: Into<Cow<'static, str>>>(name: N) -> Literal {
    Literal {
        names: vec![Lit(Name::Long(name.into()))],
        info: Info::default(),
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
        self.info.descr = Some(help);
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
            inner: inner.into_rc(),
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
            info: &self.named.info,
            inner: &(),
        };
        visitor.item(item);
    }
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_literal(&self.named.names).await;
        let res = res.map_err(|e| complete_command(&self.named.names, e));
        let value = res?
            .map(|_| {
                ctx.consume(1);
                self.present.clone()
            })
            .or_else(|| self.absent.clone());
        value.ok_or_else(|| {
            Error::missing(MissingItem::Lit {
                value: self.named.names[0].clone(),
            })
        })
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
        let res = ctx.parse_flag(&self.named.names).await;
        let res = res.map_err(|err| self.named.complete_name(err, None));
        if res? {
            Ok(self.present.clone())
        } else if let Some(absent) = &self.absent {
            Ok(absent.clone())
        } else if self.named.get_env().is_some() {
            Ok(self.present.clone())
        } else {
            let item = MissingItem::Named {
                name: self.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: None,
            };
            Err(Error::missing(item))
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

/// Named argument. Parse `VALUE` in `--name VALUE` using [`FromStr`]
///
/// Create it with [`Named::argument`]
pub struct Argument<T> {
    named: Named,
    metavar: Metavar,
    ctx: PhantomData<T>,
    adjacent: bool,
}

impl<T> Argument<T> {
    pub fn adjacent(mut self) -> Self {
        self.adjacent = true;
        self
    }
}

impl<T> Parser for Argument<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_arg(&self.named.names).await;
        let res = res.map_err(|err| self.named.complete_name(err, Some(self.metavar)));

        if let Some(os) = res? {
            if self.adjacent && ctx.current_task.borrow().consumed == 2 {
                let cursor = ctx.cursor.get();
                let name = ctx.args[cursor].clone();
                let value = ctx.args[cursor + 1].clone();
                let problem = Problem::NotAdjacent { name, value };
                Err(Error::Problem(cursor, problem))
            } else {
                parse_os_str(os).map_err(|e| problem_at_pos(&ctx, e))
            }
        } else if let Some(os) = self.named.get_env() {
            parse_os_str(&os).map_err(|p| Error::Problem(u32::MAX, p))
        } else {
            let item = MissingItem::Named {
                name: self.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: Some(self.metavar),
            };
            Err(Error::missing(item))
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
impl<T> Leaf for Argument<T> {}

impl<T> Argument<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.named.help = Some(help);
        self
    }
}

/// # complete for argument
impl<T: 'static> Argument<T> {
    pub fn complete<F>(self, completer: F) -> WithComplete<Argument<T>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
        }
    }
}
/// # complete for positional
impl<T: 'static> Positional<T> {
    pub fn complete<F>(self, completer: F) -> WithComplete<Positional<T>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
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

pub struct WithComplete<P> {
    inner: P,
    group: Option<String>,
    completer: StringCompleter,
}

impl<I> WithComplete<I> {
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

impl<P> Parser for WithComplete<P>
where
    P: Parser,
{
    type Output = P::Output;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        self.inner
            .eval(ctx)
            .await
            .map_err(|err| complete_value(err, self.group.as_deref(), &self.completer))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

impl<P: Leaf> Leaf for WithComplete<P> {}

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

fn complete_pos(err: Error, needs_strict: bool, meta: Metavar) -> Error {
    let Error::CompReq(ref comp) = err else {
        return err;
    };

    match comp {
        CompleteReq::Anything if needs_strict => todo!(),
        CompleteReq::Anything => Error::CompReply(Vec1::new(CompleteReply::Pos { meta })),
        CompleteReq::Name { .. } | CompleteReq::Literal { .. } | CompleteReq::Value(..) => err,
    }
}

fn problem_at_pos(ctx: &Ctx, p: Problem) -> Error {
    Error::Problem(ctx.cursor.get(), p)
}

impl<T> Parser for Positional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Output = T;
    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let res = ctx.parse_pos().await;
        let res = res
            .map_err(|err| complete_pos(err, self.strict && !ctx.strict_pos.get(), self.metavar));

        let Some(os) = res? else {
            let item = MissingItem::Pos { meta: self.metavar };
            return Err(Error::missing(item));
        };
        if self.strict && !ctx.strict_pos.get() {
            let cursor = ctx.cursor.get();
            let problem = Problem::NotStrict {
                metavar: self.metavar,
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
    async fn eval<'p>(&'p self, _ctx: Ctx<'p>) -> Result<T, Error> {
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
