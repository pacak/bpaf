use std::{marker::PhantomData, str::FromStr};

use crate::os_str::parse_os_str;

use super::*;

#[derive(Debug, Clone)]
pub struct Named {
    pub(crate) names: Vec<Name<'static>>,
    pub(crate) env: Vec<String>,
    pub(crate) help: Option<String>,
}

impl Named {
    fn get_short_and_long(&self) -> (Option<char>, Option<Cow<'static, str>>) {
        let mut short = None;
        let mut long = None;

        for n in &self.names {
            match n {
                Name::Short(s) => {
                    if short.is_none() {
                        short = Some(*s)
                    }
                }
                Name::Long(cow) => {
                    if long.is_none() {
                        long = Some(cow.clone())
                    }
                }
            }
        }

        (short, long)
    }

    /// Get [`Name`] with a preference to short
    fn name_short_or_long(&self) -> Option<Name<'static>> {
        match self.get_short_and_long() {
            (None, None) => None,
            (None, Some(l)) => Some(Name::Long(l.clone())),
            (Some(s), _) => Some(Name::Short(s)),
        }
    }

    /// Get [`Name`] with a preference to long
    fn name_long_or_short(&self) -> Option<Name<'static>> {
        match self.get_short_and_long() {
            (None, None) => None,
            (_, Some(l)) => Some(Name::Long(l.clone())),
            (Some(s), None) => Some(Name::Short(s)),
        }
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

pub struct Nested<T> {
    names: Named,
    inner: RcParser<T>,
}

impl<T: 'static> Parser<T> for Bp<Nested<T>> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let (out, handle) = make_handle();
        let inner = &self.0.inner;
        let populate = |ctx: crate::Ctx| {
            // out.clone() is slightly cursed. `parse_literal_and` takes a reference to a closure
            // to avoid instantiating multiple copies of boring code so this closure must be Fn
            // (and not FnOnce), meaning extra clone for out even though the closure will
            // be executed exactly once
            let act = ctx.make_act(out.clone(), inner.clone());
            let info = ctx.make_child_info(Kind::Prod);
            ctx.add_task(Task { act, info });
        };
        ctx.parse_flag_and(&self.0.names.names, &populate).await?;
        handle.take()
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
        } else {
            let item = MissingItem::Named {
                name: self.0.named.name_long_or_short().unwrap(), // TODO - handle env
                meta: None,
            };
            Err(Error::missing(item))
        }
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

        match res?.map(parse_os_str) {
            Some(Ok(t)) => Ok(t),
            Some(Err(err)) => todo!("{err:?}"),
            None => {
                let item = MissingItem::Named {
                    name: self.0.named.name_long_or_short().unwrap(), // TODO - handle env
                    meta: Some(self.0.metavar),
                };
                Err(Error::missing(item))
            }
        }
    }
}

pub struct Positional<T> {
    metavar: Metavar,
    ctx: PhantomData<T>,
}

pub fn positional<T: 'static>(metavar: &'static str) -> Bp<Positional<T>> {
    Bp(Positional {
        metavar: Metavar(metavar),
        ctx: PhantomData,
    })
}

impl<T> Parser<T> for Bp<Positional<T>>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        match ctx.parse_pos().await?.map(parse_os_str) {
            Some(Ok(t)) => Ok(t),
            Some(Err(err)) => todo!("{err:?}"),
            None => {
                let item = MissingItem::Pos {
                    meta: self.0.metavar,
                };
                Err(Error::missing(item))
            }
        }
    }
}

struct DummyAnyOs<T>(Rc<dyn Fn(&OsStr) -> Option<T>>);
struct DummyAny<T>(Rc<dyn Fn(&str) -> Option<T>>);

pub fn any<T: 'static>(check: impl Fn(&str) -> Option<T> + 'static) -> impl Parser<T> {
    DummyAny(Rc::new(check))
}

impl<T: 'static> Parser<T> for DummyAny<T> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        let parser = self.0.clone();
        let check = Box::new(move |os: &OsStr| -> Option<Box<dyn std::any::Any>> {
            Some(Box::new(parser(os.to_str()?)?))
        });
        Ok(*ctx
            .parse_any(check)
            .await?
            .unwrap()
            .downcast()
            .expect("It should match"))
    }
}
