//! FIXME
use std::{marker::PhantomData, str::FromStr};

use crate::{args::Word, info::Error, Args, Meta, Parser};

/// Parser that substitutes missing value with a function results but not parser
/// failure, created with [`Parser::fallback_with`].
pub struct ParseFallbackWith<T, P, F, E> {
    pub(crate) inner: P,
    pub(crate) inner_res: PhantomData<T>,
    pub(crate) fallback: F,
    pub(crate) err: PhantomData<E>,
}

impl<T, P, F, E> Parser<T> for ParseFallbackWith<T, P, F, E>
where
    P: Parser<T>,
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        match self.inner.eval(args) {
            Ok(ok) => Ok(ok),
            e @ Err(Error::Stderr(_) | Error::Stdout(_)) => e,
            Err(Error::Missing(_)) => match (self.fallback)() {
                Ok(ok) => Ok(ok),
                Err(e) => Err(Error::Stderr(e.to_string())),
            },
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Fail with a fixed error message
/// ```rust
/// # use bpaf::*;
/// let a = short('a').switch();
/// let no_a = fail("Custom error message for missing -a");
///
/// // Parser produces a custom error message if `-a` isn't specified
/// let a_ = construct!([a, no_a]); // impl Parser<bool>
/// # drop(a_);
/// ```
#[must_use]
pub fn fail<T>(msg: &'static str) -> ParseFail<T> {
    ParseFail {
        field1: msg,
        field2: PhantomData,
    }
}

/// Parser with attached message to several fields, created with [`Parser::group_help`].
pub struct ParseGroupHelp<P> {
    pub(crate) inner: P,
    pub(crate) message: &'static str,
}

impl<T, P> Parser<T> for ParseGroupHelp<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        Meta::decorate(self.inner.meta(), self.message)
    }
}

/// Parser that applies inner parser multiple times and collects results into Vec, inner parser must
/// succeed at least once, created with [`Parser::some`].
pub struct ParseSome<P> {
    pub(crate) inner: P,
    pub(crate) message: &'static str,
}

impl<T, P> Parser<Vec<T>> for ParseSome<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Vec<T>, Error> {
        let items = std::iter::from_fn(|| self.inner.eval(args).ok()).collect::<Vec<_>>();
        if items.is_empty() {
            Err(Error::Stderr(self.message.to_string()))
        } else {
            Ok(items)
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that returns results as usual but not shown in `--help` output, created with
/// [`Parser::hide`]
pub struct ParseHide<P> {
    pub(crate) inner: P,
}

impl<T, P> Parser<T> for ParseHide<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that tries to either of two parsers and uses one that succeeeds, created with
/// [`Parser::or_else`].
pub struct ParseOrElse<A, B> {
    pub(crate) this: A,
    pub(crate) that: B,
}

impl<A, B, T> Parser<T> for ParseOrElse<A, B>
where
    A: Parser<T>,
    B: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let mut args_a = args.clone();
        let mut args_b = args.clone();
        match (self.this.eval(&mut args_a), self.that.eval(&mut args_b)) {
            // side channel (--help) reporting takes priority
            (e @ Err(Error::Stdout(_)), _) | (_, e @ Err(Error::Stdout(_))) => e,

            (Ok(a), Ok(b)) => {
                if args_a.head <= args_b.head {
                    *args = args_a;
                    Ok(a)
                } else {
                    *args = args_b;
                    Ok(b)
                }
            }
            (Ok(a), Err(_)) => {
                *args = args_a;
                Ok(a)
            }
            (Err(_), Ok(b)) => {
                *args = args_b;
                Ok(b)
            }
            (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
        }
    }

    fn meta(&self) -> Meta {
        self.this.meta().or(self.that.meta())
    }
}

/// Parser that transforms parsed value with a failing function, created with [`Parser::parse`]
pub struct ParseWith<T, P, F, R, E> {
    pub(crate) inner: P,
    pub(crate) inner_res: PhantomData<T>,
    pub(crate) parse_fn: F,
    pub(crate) res: PhantomData<R>,
    pub(crate) err: PhantomData<E>,
}

impl<T, P, F, E, R> Parser<R> for ParseWith<T, P, F, E, R>
where
    P: Parser<T>,
    F: Fn(T) -> Result<R, E>,
    E: ToString,
{
    fn eval(&self, args: &mut Args) -> Result<R, Error> {
        let t = self.inner.eval(args)?;
        match (self.parse_fn)(t) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::Stderr(
                if let Some(Word { utf8: Some(w), .. }) = args.current_word() {
                    format!("Couldn't parse {:?}: {}", w, e.to_string())
                } else {
                    format!("Couldn't parse: {}", e.to_string())
                },
            )),
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that substitutes missing value but not parse failure, created with [`Parser::fallback`].
pub struct ParseFallback<P, T> {
    pub(crate) inner: P,
    pub(crate) value: T,
}

impl<P, T> Parser<T> for ParseFallback<P, T>
where
    P: Parser<T>,
    T: Clone,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        match self.inner.eval(args) {
            Ok(ok) => Ok(ok),
            e @ Err(Error::Stderr(_) | Error::Stdout(_)) => e,
            Err(Error::Missing(_)) => Ok(self.value.clone()),
        }
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

/// Parser fails with a message if check returns false, created with [`Parser::guard`].
pub struct ParseGuard<P, F> {
    pub(crate) inner: P,
    pub(crate) check: F,
    pub(crate) message: &'static str,
}

impl<T, P, F> Parser<T> for ParseGuard<P, F>
where
    P: Parser<T>,
    F: Fn(&T) -> bool,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let t = self.inner.eval(args)?;
        if (self.check)(&t) {
            Ok(t)
        } else {
            Err(Error::Stderr(self.message.to_string()))
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that returns results of inner parser wrapped into [`Option`], created with [`Parser::optional`].
pub struct ParseOptional<P> {
    pub(crate) inner: P,
}

impl<T, P> Parser<Option<T>> for ParseOptional<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Option<T>, Error> {
        let orig_args = args.clone();
        if let Ok(val) = self.inner.eval(args) {
            Ok(Some(val))
        } else {
            *args = orig_args;
            Ok(None)
        }
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

/// Parser that uses [`FromStr`] instance of a type, created with [`Parser::from_str`].
pub struct ParseFromStr<P, R> {
    pub(crate) inner: P,
    pub(crate) ty: PhantomData<R>,
}

impl<E, P, T> Parser<T> for ParseFromStr<P, T>
where
    P: Parser<String>,
    T: FromStr<Err = E>,
    E: ToString,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let s = self.inner.eval(args)?;
        match T::from_str(&s) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Stderr(
                if let Some(Word { utf8: Some(w), .. }) = args.current_word() {
                    format!("Couldn't parse {:?}: {}", w, e.to_string())
                } else {
                    format!("Couldn't parse: {}", e.to_string())
                },
            )),
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that applies inner parser multiple times and collects results into [`Vec`], created with
/// [`Parser::many`].
pub struct ParseMany<P> {
    pub(crate) inner: P,
}

impl<T, P> Parser<Vec<T>> for ParseMany<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Vec<T>, Error> {
        Ok(std::iter::from_fn(|| self.inner.eval(args).ok()).collect())
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(self.inner.meta()))
    }
}

/// Parser that returns a given value without consuming anything, created with
/// [`pure`](crate::pure).
pub struct ParsePure<T>(pub(crate) T);
impl<T: Clone + 'static> Parser<T> for ParsePure<T> {
    fn eval(&self, _args: &mut Args) -> Result<T, Error> {
        Ok(self.0.clone())
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that fails without consuming any input, created with [`fail`].
pub struct ParseFail<T> {
    pub(crate) field1: &'static str,
    pub(crate) field2: PhantomData<T>,
}
impl<T> Parser<T> for ParseFail<T> {
    fn eval(&self, _args: &mut Args) -> Result<T, Error> {
        Err(Error::Stderr(self.field1.to_owned()))
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that transforms parsed value with a function, created with [`Parser::map`].
pub struct ParseMap<T, P, F, R> {
    pub(crate) inner: P,
    pub(crate) inner_res: PhantomData<T>,
    pub(crate) map_fn: F,
    pub(crate) res: PhantomData<R>,
}
impl<P, T, F, R> Parser<R> for ParseMap<T, P, F, R>
where
    F: Fn(T) -> R,
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut Args) -> Result<R, Error> {
        let t = self.inner.eval(args)?;
        Ok((self.map_fn)(t))
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Create parser from a function, [`construct!`] uses it internally
#[derive(Clone)]
pub struct ParseConstruct<P> {
    /// TODO
    pub inner: P,
    /// TODO
    pub meta: Meta,
}

impl<T, P> Parser<T> for ParseConstruct<P>
where
    P: Fn(&mut Args) -> Result<T, Error>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let mut args_copy = args.clone();
        let res = (self.inner)(&mut args_copy);

        match res {
            Ok(val) => {
                std::mem::swap(args, &mut args_copy);
                Ok(val)
            }
            Err(err) => Err(err),
        }
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}
