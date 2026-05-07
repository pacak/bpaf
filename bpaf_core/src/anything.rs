use std::{ffi::OsStr, rc::Rc, str::FromStr};

use crate::{
    Ctx, ExitHandle, JoinHandle, Metavar, Parser,
    error::Error,
    make_chan,
    os_str::parse_os_str,
    traits::{Item, Visitor},
};

pub struct Anything<T> {
    meta: Metavar,
    help: Option<&'static str>,
    join: JoinHandle<T>,
    check: Rc<dyn Fn(&OsStr) -> bool>,
}

impl<T> Anything<T> {
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}

pub trait AnyCheck<C, T> {
    fn into_boxed(self, h: ExitHandle<T>) -> Rc<dyn Fn(&OsStr) -> bool>;
}

impl<T, F> AnyCheck<&OsStr, T> for F
where
    T: 'static,
    F: Fn(&OsStr) -> Option<T> + 'static,
{
    fn into_boxed(self, h: ExitHandle<T>) -> Rc<dyn Fn(&OsStr) -> bool> {
        Rc::new(move |os: &OsStr| match self(os) {
            Some(v) => {
                h.exit(Ok(v));
                true
            }
            None => false,
        })
    }
}
impl<T, F> AnyCheck<&str, T> for F
where
    T: 'static,
    F: Fn(&str) -> Option<T> + 'static,
{
    fn into_boxed(self, h: ExitHandle<T>) -> Rc<dyn Fn(&OsStr) -> bool> {
        Rc::new(move |s: &OsStr| match s.to_str().and_then(|v| self(v)) {
            Some(v) => {
                h.exit(Ok(v));
                true
            }
            None => false,
        })
    }
}

pub fn any<K, T: 'static>(meta: &'static str, check: impl AnyCheck<K, T>) -> Anything<T> {
    let (exit, join) = make_chan();
    Anything {
        meta: Metavar(meta),
        help: None,
        join,
        check: check.into_boxed(exit),
    }
}

impl<T: 'static> Parser for Anything<T> {
    type Output = T;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let r = ctx
            .await_passing_check(self.meta, self.check.clone())
            .await?;

        if r {
            self.join.take()
        } else {
            let item = crate::error::MissingItem::Pos { meta: self.meta };
            Err(Error::missing(item))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.item(Item::Positional {
            meta: self.meta,
            help: self.help,
        })
    }
}

pub fn any_from_str<T: FromStr + 'static>(meta: &'static str) -> Anything<T>
where
    <T as FromStr>::Err: std::error::Error,
{
    any(meta, |os: &OsStr| parse_os_str(os).ok())
}
