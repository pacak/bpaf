use std::{marker::PhantomData, str::FromStr};

use crate::{
    adapters::PureWith,
    complete::{CompleteReply, StringCompleter, complete_command, complete_value},
    error::MissingItem,
    os_str::parse_os_str,
};

use super::*;

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
        }
    }

    pub fn nest<T: 'static, P: Parser<T> + 'static>(self, inner: P) -> Nested<T> {
        Nested {
            names: self,
            inner: inner.into_rc(),
        }
    }
}

impl<T: 'static> Nested<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.names.help = Some(help);
        self
    }
}

pub struct Nested<T> {
    names: Named,
    inner: RcParser<T>,
}

impl<T: 'static> Parser<T> for Nested<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_flag(&self.names.names).await;
        let res = res.map_err(|err| self.names.complete_name(err, None)); // TODO - can we do better?

        if !res? {
            let item = MissingItem::Named {
                name: self.names.names[0].clone().into_owned(),
                meta: None,
            };
            return Err(Error::missing(item));
        }
        let inner = ctx.fork(None);
        inner.cursor.update(|c| c + 1);

        let (out, handle) = make_handle();
        let act = inner.make_act(out, self.inner.clone());
        let info = inner.make_child_info(Kind::Prod);
        inner.add_task(Task { act, info });
        let executor_res = inner.execute(true, &self.inner, None);
        let res = handle.take();
        ctx.consume((inner.cursor.get() - ctx.cursor.get()) as u32);

        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1 + e2),
        }
    }
}

impl<T: 'static> Visited for Nested<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Nested {
            named: &self.names,
            inner: &self.inner,
        };
        visitor.item(item);
    }
}

pub struct Literal<T> {
    pub(crate) present: T,
    pub(crate) absent: Option<T>,
    pub(crate) named: LNamed,
}

pub struct LNamed {
    pub(crate) info: Info,
    pub(crate) names: Vec<Lit<'static>>,
}

pub fn literal<N: Into<Cow<'static, str>>>(name: N) -> LNamed {
    LNamed {
        names: vec![Lit(Name::Long(name.into()))],
        info: Info::default(),
    }
}

impl LNamed {
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

impl LNamed {
    pub fn switch(self) -> Literal<bool> {
        Literal {
            named: self,
            present: true,
            absent: Some(false),
        }
    }
    pub fn req_flag<T: 'static>(self, value: T) -> Literal<T> {
        Literal {
            named: self,
            present: value,
            absent: None,
        }
    }
    pub fn flag<T: 'static>(self, present: T, absent: T) -> Literal<T> {
        Literal {
            named: self,
            present,
            absent: Some(absent),
        }
    }
}

impl Visited for () {
    fn visit<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}

impl<T: 'static> Visited for Literal<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Command {
            names: &self.named.names,
            info: &self.named.info,
            inner: &(),
        };
        visitor.item(item);
    }
}

impl<T: Clone + 'static> Parser<T> for Literal<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
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

impl<T> Leaf for Literal<T> {}

pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    named: Named,
}

impl<T: Clone + 'static> Parser<T> for Flag<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
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
}
impl<T> Visited for Flag<T> {
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

pub struct Argument<T> {
    named: Named,
    metavar: Metavar,
    ctx: PhantomData<T>,
}

impl<T> Parser<T> for Argument<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_arg(&self.named.names).await;
        let res = res.map_err(|err| self.named.complete_name(err, Some(self.metavar)));

        if let Some(os) = res? {
            parse_os_str(os).map_err(|e| problem_at_pos(&ctx, e))
        } else if let Some(os) = self.named.get_env() {
            parse_os_str(os).map_err(|p| Error::Problem(u32::MAX, p))
        } else {
            let item = MissingItem::Named {
                name: self.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: Some(self.metavar),
            };
            Err(Error::missing(item))
        }
    }
}

impl<T> Visited for Argument<T> {
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
    pub fn complete<F>(self, completer: F) -> WithComplete<T, Argument<T>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
            ctx: PhantomData,
        }
    }
}
/// # complete for positional
impl<T: 'static> Positional<T> {
    pub fn complete<F>(self, completer: F) -> WithComplete<T, Positional<T>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
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

pub struct WithComplete<T, P> {
    ctx: PhantomData<T>,
    inner: P,
    group: Option<String>,
    completer: StringCompleter,
}

impl<T, I> WithComplete<T, I> {
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

impl<P, T> Parser<T> for WithComplete<T, P>
where
    T: 'static,
    P: Parser<T>,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        self.inner
            .run(ctx)
            .await
            .map_err(|err| complete_value(err, self.group.as_deref(), &self.completer))
    }
}

impl<P, T> Visited for WithComplete<T, P>
where
    T: 'static,
    P: Parser<T>,
{
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

impl<T, P: Leaf> Leaf for WithComplete<T, P> {}

pub struct Positional<T> {
    pub(crate) metavar: Metavar,
    pub(crate) help: Option<&'static str>,
    ctx: PhantomData<T>,
}

pub fn positional<T: 'static>(metavar: &'static str) -> Positional<T> {
    Positional {
        metavar: Metavar(metavar),
        ctx: PhantomData,
        help: None,
    }
}

fn complete_pos(err: Error, meta: Metavar) -> Error {
    let Error::CompReq(ref comp) = err else {
        return err;
    };
    match comp {
        CompleteReq::Anything => Error::CompReply(Vec1::new(CompleteReply::Pos { meta })),
        CompleteReq::Name { .. } | CompleteReq::Literal { .. } | CompleteReq::Value(..) => err,
    }
}

fn problem_at_pos(ctx: &Ctx, p: Problem) -> Error {
    Error::Problem(ctx.cursor.get() as u32, p)
}

impl<T> Parser<T> for Positional<T>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_pos().await;
        let res = res.map_err(|err| complete_pos(err, self.metavar));

        if let Some(os) = res? {
            parse_os_str(os).map_err(|p| problem_at_pos(&ctx, p))
        } else {
            let item = MissingItem::Pos { meta: self.metavar };
            Err(Error::missing(item))
        }
    }
}
impl<T> Visited for Positional<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Positional {
            meta: self.metavar,
            help: self.help,
        };
        visitor.item(item);
    }
}

struct DummyAnyOs<T>(Rc<dyn Fn(&OsStr) -> Option<T>>);
struct DummyAny<T> {
    meta: Metavar,
    check: Rc<dyn Fn(&str) -> Option<T>>,
}

pub fn any<T: 'static>(
    meta: &'static str,
    check: impl Fn(&str) -> Option<T> + 'static,
) -> impl Parser<T> {
    DummyAny {
        meta: Metavar(meta),
        check: Rc::new(check),
    }
}

pub fn any_from_str<T: FromStr + 'static>(meta: &'static str) -> impl Parser<T> {
    DummyAny {
        meta: Metavar(meta),
        check: Rc::new(|s: &str| T::from_str(s).ok()),
    }
}

impl<T> Visited for DummyAny<T> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Positional {
            meta: self.meta,
            help: None,
        };
        visitor.item(item)
    }
}

impl<T: 'static> Parser<T> for DummyAny<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let h = Rc::new(Cell::new(None));
        let out = h.clone();
        let c = self.check.clone();
        let check = Rc::new(move |os: &OsStr| -> bool {
            let r = os.to_str().and_then(|v| c(v));
            match r {
                Some(v) => {
                    h.set(Some(v));
                    true
                }
                None => false,
            }
        });

        if ctx.await_passing_check(check).await? {
            Ok(out.take().unwrap())
        } else {
            let item = MissingItem::Pos {
                meta: Metavar("XXX"), // TODO
            };
            Err(Error::missing(item))
        }
    }
}

/// In case of conflicts it is excluded from "earlier running parser" wins
/// and it's position in the selection takes a priority. Including it in conflict resolution will
/// make it so any branch containing `pure` anywhere automatically advances.
pub fn pure<T: Clone + 'static>(value: T) -> Pure<T> {
    Pure { value }
}

pub struct Pure<T> {
    value: T,
}

impl<T: 'static + Clone> Parser<T> for Pure<T> {
    async fn run(&self, _ctx: Ctx) -> Result<T, Error> {
        Ok(self.value.clone())
    }
}

impl<T> Visited for Pure<T> {
    fn visit<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}

pub fn pure_with<T, F, E>(act: F) -> PureWith<F>
where
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    PureWith { act }
}
