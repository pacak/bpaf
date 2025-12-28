use std::{marker::PhantomData, str::FromStr};

use crate::{
    adapters::PureWith,
    complete::{CompleteReply, complete_command, complete_value},
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

    /// Get [`Name`] with a preference to short
    pub(crate) fn name_short_or_long(&self) -> Option<Name<'static>> {
        match self.get_short_and_long() {
            (None, None) => None,
            (None, Some(l)) => Some(Name::Long(l.clone())),
            (Some(s), _) => Some(Name::Short(s)),
        }
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

pub fn short(name: char) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}

pub fn long(name: &'static str) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}

pub fn long_string(name: String) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}
pub fn env(name: &'static str) -> Bp<Named> {
    Bp(Named {
        names: Vec::new(),
        env: vec![name],
        help: None,
    })
}

impl Bp<Named> {
    pub fn short(mut self, name: char) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn long(mut self, name: &'static str) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn long_string(mut self, name: String) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn env(mut self, name: &'static str) -> Self {
        self.0.env.push(name);
        self
    }

    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help = Some(help);
        self
    }

    pub fn switch(self) -> Bp<Flag<bool>> {
        Bp(Flag {
            present: true,
            absent: Some(false),
            named: self.0,
        })
    }
    pub fn flag<T>(self, present: T, absent: T) -> Bp<Flag<T>> {
        Bp(Flag {
            present,
            absent: Some(absent),
            named: self.0,
        })
    }

    pub fn req_flag<T>(self, present: T) -> Bp<Flag<T>> {
        Bp(Flag {
            present,
            absent: None,
            named: self.0,
        })
    }

    pub fn argument<T>(self, metavar: &'static str) -> Bp<Argument<T>> {
        Bp(Argument {
            named: self.0,
            metavar: Metavar(metavar),
            ctx: PhantomData,
        })
    }

    pub fn nest<T: 'static, P: Parser<T> + 'static>(self, inner: P) -> Bp<Nested<T>> {
        Bp(Nested {
            names: self.0,
            inner: inner.into_rc().0,
        })
    }
}

impl<T: 'static> Bp<Nested<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.names.help = Some(help);
        self
    }
}

pub struct Nested<T> {
    names: Named,
    inner: RcParser<T>,
}

impl<T: 'static> Parser<T> for Bp<Nested<T>> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_flag(&self.0.names.names).await;
        let res = res.map_err(|err| self.0.names.complete_name(err, None)); // TODO - can we do better?

        if !res? {
            let item = MissingItem::Named {
                name: self.0.names.names[0].clone().into_owned(),
                meta: None,
            };
            return Err(Error::missing(item));
        }
        let inner = ctx.fork(None);
        inner.cursor.update(|c| c + 1);

        let (out, handle) = make_handle();
        let act = inner.make_act(out, self.0.inner.clone());
        let info = inner.make_child_info(Kind::Prod);
        inner.add_task(Task { act, info });
        let executor_res = inner.execute(true, &self.0.inner, None);
        let res = handle.take();
        ctx.consume((inner.cursor.get() - ctx.cursor.get()) as u32);

        match (res, executor_res) {
            (res @ Ok(_), Ok(_)) => Ok(res?),
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e),
            (Err(e1), Err(e2)) => Err(e1 + e2),
        }
    }
}

impl<T: 'static> Visited for Bp<Nested<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Nested {
            named: &self.0.names,
            inner: &self.0.inner,
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

pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    named: Named,
}

impl<T: Clone + 'static> Parser<T> for Bp<Flag<T>> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_flag(&self.0.named.names).await;
        let res = res.map_err(|err| self.0.named.complete_name(err, None));
        if res? {
            Ok(self.0.present.clone())
        } else if let Some(absent) = &self.0.absent {
            Ok(absent.clone())
        } else if self.0.named.get_env().is_some() {
            Ok(self.0.present.clone())
        } else {
            let item = MissingItem::Named {
                name: self.0.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: None,
            };
            Err(Error::missing(item))
        }
    }
}
impl<T> Visited for Bp<Flag<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Flag {
            named: &self.0.named,
        };
        if self.0.absent.is_some() {
            visitor.push_group(VisitGroup::Optional);
            visitor.item(item);
            visitor.pop_group();
        } else {
            visitor.item(item);
        }
    }
}

impl<T> Bp<Flag<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.named.help = Some(help);
        self
    }
}

pub struct Argument<T> {
    named: Named,
    metavar: Metavar,
    ctx: PhantomData<T>,
}

impl<T> Parser<T> for Bp<Argument<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_arg(&self.0.named.names).await;
        let res = res.map_err(|err| self.0.named.complete_name(err, Some(self.0.metavar)));

        if let Some(os) = res? {
            parse_os_str(os).map_err(|e| problem_at_pos(&ctx, e))
        } else if let Some(os) = self.0.named.get_env() {
            parse_os_str(os).map_err(|p| Error::Problem(u32::MAX, p))
        } else {
            let item = MissingItem::Named {
                name: self.0.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: Some(self.0.metavar),
            };
            Err(Error::missing(item))
        }
    }
}

impl<T> Visited for Bp<Argument<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Arg {
            named: &self.0.named,
            meta: self.0.metavar,
        };
        visitor.item(item);
    }
}

impl<T> Bp<Argument<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.named.help = Some(help);
        self
    }
}

/// # complete for argument
impl<T: 'static> Bp<Argument<T>> {
    pub fn complete<F>(self, completer: F) -> Bp<WithComplete<T, Argument<T>>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        Bp(WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
            ctx: PhantomData,
        })
    }
}
/// # complete for positional
impl<T: 'static> Bp<Positional<T>> {
    pub fn complete<F>(self, completer: F) -> Bp<WithComplete<T, Positional<T>>>
    where
        Self: Sized,
        F: Fn(&str) -> Vec<(String, Option<String>)> + 'static,
    {
        Bp(WithComplete {
            inner: self,
            completer: Box::new(completer),
            group: None,
            ctx: PhantomData,
        })
    }
}

impl<T> Bp<Positional<T>> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.0.help = Some(help);
        self
    }
}

pub struct WithComplete<T, P> {
    ctx: PhantomData<T>,
    inner: Bp<P>,
    group: Option<String>,
    completer: Box<dyn Fn(&str) -> Vec<(String, Option<String>)>>,
}

impl<T, I> Bp<WithComplete<T, I>> {
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.0.group = Some(group.into());
        self
    }
}

impl<P, T> Parser<T> for Bp<WithComplete<T, P>>
where
    T: 'static,
    Bp<P>: Parser<T>,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        self.0
            .inner
            .run(ctx)
            .await
            .map_err(|err| complete_value(err, self.0.group.as_deref(), &self.0.completer))
    }
}

impl<P, T> Visited for Bp<WithComplete<T, P>>
where
    T: 'static,
    Bp<P>: Parser<T>,
{
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.0.inner.visit(visitor)
    }
}

pub struct Positional<T> {
    pub(crate) metavar: Metavar,
    pub(crate) help: Option<&'static str>,
    ctx: PhantomData<T>,
}

pub fn positional<T: 'static>(metavar: &'static str) -> Bp<Positional<T>> {
    Bp(Positional {
        metavar: Metavar(metavar),
        ctx: PhantomData,
        help: None,
    })
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

impl<T> Parser<T> for Bp<Positional<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let res = ctx.parse_pos().await;
        let res = res.map_err(|err| complete_pos(err, self.0.metavar));

        if let Some(os) = res? {
            parse_os_str(os).map_err(|p| problem_at_pos(&ctx, p))
        } else {
            let item = MissingItem::Pos {
                meta: self.0.metavar,
            };
            Err(Error::missing(item))
        }
    }
}
impl<T> Visited for Bp<Positional<T>> {
    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        let item = Item::Positional {
            meta: self.0.metavar,
            help: self.0.help.as_deref(),
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
pub fn pure<T: Clone + 'static>(value: T) -> Bp<Pure<T>> {
    Bp(Pure { value })
}

pub struct Pure<T> {
    value: T,
}

impl<T: 'static + Clone> Parser<T> for Bp<Pure<T>> {
    async fn run(&self, _ctx: Ctx) -> Result<T, Error> {
        Ok(self.0.value.clone())
    }
}

impl<T> Visited for Bp<Pure<T>> {
    fn visit<'a>(&'a self, _: &mut dyn Visitor<'a>) {}
}

pub fn pure_with<T, F, E>(act: F) -> Bp<PureWith<F>>
where
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    Bp(PureWith { act })
}
