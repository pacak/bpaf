//!
use std::{marker::PhantomData, str::FromStr};

use crate::{args::Word, info::Error, Args, Meta, Parser};

#[cfg(feature = "autocomplete")]
use crate::complete_gen::Comp;

/// Parser that substitutes missing value with a function results but not parser
/// failure, created with [`fallback_with`](Parser::fallback_with).
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

/// Parser with attached message to several fields, created with [`group_help`](Parser::group_help).
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
        Meta::Decorated(Box::new(self.inner.meta()), self.message)
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
        let mut res = Vec::new();
        let mut len = args.len();

        while let Some(val) = parse_option(&self.inner, args)? {
            if args.len() < len {
                len = args.len();
                res.push(val);
            } else {
                break;
            }
        }

        if res.is_empty() {
            Err(Error::Stderr(self.message.to_string()))
        } else {
            Ok(res)
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
        #[cfg(feature = "autocomplete")]
        let mut comps = Vec::new();

        #[cfg(feature = "autocomplete")]
        if let Some(comp) = &mut args.comp {
            std::mem::swap(&mut comps, &mut comp.comps);
        }

        #[allow(clippy::let_and_return)]
        let res = self.inner.eval(args);

        #[cfg(feature = "autocomplete")]
        if let Some(comp) = &mut args.comp {
            std::mem::swap(&mut comps, &mut comp.comps);
        }
        res
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
        #[cfg(feature = "autocomplete")]
        let mut comp_items = Vec::new();
        #[cfg(feature = "autocomplete")]
        if let Some(comp) = &mut args.comp {
            std::mem::swap(&mut comp_items, &mut comp.comps);
        }

        // create forks for both branches, go with a successful one.
        // if they both fail - fallback to the original arguments
        let mut args_a = args.clone();
        let mut args_b = args.clone();
        args_a.head = usize::MAX;
        args_b.head = usize::MAX;
        let res_a = self.this.eval(&mut args_a);
        let res_b = self.that.eval(&mut args_b);

        // if higher depth parser succeeds - it takes a priority
        // completion from different depths should never mix either
        match Ord::cmp(&args_a.depth, &args_b.depth) {
            std::cmp::Ordering::Less => {
                std::mem::swap(args, &mut args_b);
                #[cfg(feature = "autocomplete")]
                if let Some(comp) = &mut args.comp {
                    comp.comps.extend(comp_items);
                }
                return res_b;
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                std::mem::swap(args, &mut args_a);
                #[cfg(feature = "autocomplete")]
                if let Some(comp) = &mut args.comp {
                    comp.comps.extend(comp_items);
                }
                return res_a;
            }
        }

        #[cfg(feature = "autocomplete")]
        if let (Some(a), Some(b)) = (&mut args_a.comp, &mut args_b.comp) {
            comp_items.extend(a.comps.drain(0..));
            comp_items.extend(b.comps.drain(0..));
        }

        // otherwise pick based on the left most or successful one
        #[allow(clippy::let_and_return)]
        let res = match (res_a, res_b) {
            (Ok(a), Ok(b)) => {
                if args_a.head <= args_b.head {
                    std::mem::swap(args, &mut args_a);
                    Ok(a)
                } else {
                    std::mem::swap(args, &mut args_b);
                    Ok(b)
                }
            }
            (Ok(a), Err(_)) => {
                std::mem::swap(args, &mut args_a);
                Ok(a)
            }
            (Err(_), Ok(b)) => {
                std::mem::swap(args, &mut args_b);
                Ok(b)
            }
            (Err(e1), Err(e2)) => {
                args.tainted |= args_a.tainted | args_b.tainted;
                Err(e1.combine_with(e2))
            }
        };

        #[cfg(feature = "autocomplete")]
        if let Some(comp) = &mut args.comp {
            comp.comps.extend(comp_items);
        }

        res
    }

    fn meta(&self) -> Meta {
        self.this.meta().or(self.that.meta())
    }
}

/// Parser that transforms parsed value with a failing function, created with
/// [`parse`](Parser::parse)
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
            Err(e) => {
                args.tainted = true;
                Err(Error::Stderr(
                    if let Some(Word { utf8: Some(w), .. }) = args.current_word() {
                        format!("Couldn't parse {:?}: {}", w, e.to_string())
                    } else {
                        format!("Couldn't parse: {}", e.to_string())
                    },
                ))
            }
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that substitutes missing value but not parse failure, created with
/// [`fallback`](Parser::fallback).
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
        let mut clone = args.clone();
        match self.inner.eval(&mut clone) {
            Ok(ok) => {
                std::mem::swap(args, &mut clone);
                Ok(ok)
            }
            e @ Err(Error::Stderr(_) | Error::Stdout(_)) => e,
            Err(Error::Missing(_)) => Ok(self.value.clone()),
        }
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

/// Parser fails with a message if check returns false, created with [`guard`](Parser::guard).
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
            args.tainted = true;
            Err(Error::Stderr(
                if let Some(Word { utf8: Some(w), .. }) = args.current_word() {
                    format!("Failed to verify {:?}: {}", w, self.message)
                } else {
                    self.message.to_string()
                },
            ))
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that returns results of inner parser wrapped into [`Option`], created with
/// [`optional`](Parser::optional).
pub struct ParseOptional<P> {
    pub(crate) inner: P,
}

impl<T, P> Parser<Option<T>> for ParseOptional<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Option<T>, Error> {
        parse_option(&self.inner, args)
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

/// Parser that uses [`FromStr`] instance of a type, created with [`from_str`](Parser::from_str).
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
            Err(e) => {
                args.tainted = true;
                Err(Error::Stderr(
                    if let Some(Word { utf8: Some(w), .. }) = args.current_word() {
                        format!("Couldn't parse {:?}: {}", w, e.to_string())
                    } else {
                        format!("Couldn't parse: {}", e.to_string())
                    },
                ))
            }
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parser that applies inner parser multiple times and collects results into [`Vec`], created with
/// [`many`](Parser::many).
pub struct ParseMany<P> {
    pub(crate) inner: P,
}

fn parse_option<P, T>(parser: &P, args: &mut Args) -> Result<Option<T>, Error>
where
    P: Parser<T>,
{
    let mut orig_args = args.clone();
    match parser.eval(args) {
        Ok(val) => Ok(Some(val)),
        Err(err) => {
            std::mem::swap(args, &mut orig_args);
            #[cfg(feature = "autocomplete")]
            if orig_args.comp.is_some() {
                std::mem::swap(&mut args.comp, &mut orig_args.comp);
            }
            match err {
                Error::Stdout(_) | Error::Stderr(_) => Err(err),
                Error::Missing(_) => Ok(None),
            }
        }
    }
}

impl<T, P> Parser<Vec<T>> for ParseMany<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Vec<T>, Error> {
        let mut res = Vec::new();
        let mut len = args.len();
        while let Some(val) = parse_option(&self.inner, args)? {
            if args.len() < len {
                len = args.len();
                res.push(val);
            } else {
                break;
            }
        }
        println!("returning with res {:?}", args);
        Ok(res)
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(self.inner.meta()))
    }
}

/// Parser that returns a given value without consuming anything, created with
/// [`pure`](crate::pure).
pub struct ParsePure<T>(pub(crate) T);
impl<T: Clone + 'static> Parser<T> for ParsePure<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        args.current = None;
        Ok(self.0.clone())
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that fails without consuming any input, created with [`fail`](crate::fail).
pub struct ParseFail<T> {
    pub(crate) field1: &'static str,
    pub(crate) field2: PhantomData<T>,
}
impl<T> Parser<T> for ParseFail<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        args.current = None;
        Err(Error::Stderr(self.field1.to_owned()))
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that transforms parsed value with a function, created with [`map`](Parser::map).
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

/// Create parser from a function, [`construct!`](crate::construct!) uses it internally
pub struct PCon<P> {
    /// inner parser closure
    pub inner: P,
    /// metas for inner parsers
    pub meta: Meta,
}

impl<T, P> Parser<T> for PCon<P>
where
    P: Fn(&mut Args) -> Result<T, Error>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let res = (self.inner)(args);
        args.current = None;
        res
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}

/// Parser that replaces metavar placeholders with actual info in shell completion
#[cfg(feature = "autocomplete")]
pub struct ParseComp<P, F> {
    pub(crate) inner: P,
    pub(crate) op: F,
}

#[cfg(feature = "autocomplete")]
impl<P, T, F, M> Parser<T> for ParseComp<P, F>
where
    P: Parser<T> + Sized,
    M: Into<String>,
    F: Fn(Option<&T>) -> Vec<(M, Option<M>)>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        // stash old
        let mut comp_items = Vec::new();

        if let Some(comp) = &mut args.comp {
            std::mem::swap(&mut comp_items, &mut comp.comps);
        }

        let res = self.inner.eval(args);

        if let Some(comp) = &mut args.comp {
            // restore old, now metavars added by inner parser, if any, are in comp_items
            std::mem::swap(&mut comp_items, &mut comp.comps);
            for ci in comp_items {
                if let Comp::Meta { .. } = ci {
                    for (replacement, description) in (self.op)(res.as_ref().ok()) {
                        comp.push_value(
                            replacement.into(),
                            description.map(|s| s.into()),
                            args.depth,
                        );
                    }
                } else {
                    comp.comps.push(ci);
                }
            }
        }
        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}
