//! Structures that implement different methods on [`Parser`] trait
use crate::{info::Error, Args, Meta, Parser};
use std::marker::PhantomData;

#[cfg(feature = "autocomplete")]
use crate::CompleteDecor;

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
        Meta::Optional(Box::new(self.inner.meta()))
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

/// Apply inner parser several times and collect results into `Vec`, created with
/// [`some`](Parser::some), requires for at least one item to be available to succeed.
/// Implements [`catch`](ParseMany::catch)
pub struct ParseSome<P> {
    pub(crate) inner: P,
    pub(crate) message: &'static str,
    pub(crate) catch: bool,
}

impl<P> ParseSome<P> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several structures that implement this attribute: [`ParseOptional`], [`ParseMany`]
    /// and [`ParseSome`], behavior should be identical for all of them.
    #[doc = include_str!("docs/catch.md")]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

impl<T, P> Parser<Vec<T>> for ParseSome<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Vec<T>, Error> {
        let mut res = Vec::new();
        let mut len = args.len();

        while let Some(val) = parse_option(&self.inner, args, self.catch)? {
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
        Meta::Many(Box::new(self.inner.meta()))
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
        args.swap_comps(&mut comps);

        #[allow(clippy::let_and_return)]
        let res = self.inner.eval(args);

        #[cfg(feature = "autocomplete")]
        args.swap_comps(&mut comps);
        res
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that hides inner parser from usage line
///
/// No other changes to the inner parser
pub struct ParseHideUsage<P> {
    pub(crate) inner: P,
}
impl<T, P> Parser<T> for ParseHideUsage<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        Meta::HideUsage(Box::new(self.inner.meta()))
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
        args.swap_comps(&mut comp_items);

        // create forks for both branches, go with a successful one.
        // if they both fail - fallback to the original arguments
        let mut args_a = args.clone();
        args_a.head = usize::MAX;
        let (res_a, err_a) = match self.this.eval(&mut args_a) {
            Ok(ok) => (Some(ok), None),
            Err(err) => (None, Some(err)),
        };

        let mut args_b = args.clone();
        args_b.head = usize::MAX;
        let (res_b, err_b) = match self.that.eval(&mut args_b) {
            Ok(ok) => (Some(ok), None),
            Err(err) => (None, Some(err)),
        };

        if this_or_that_picks_first(
            err_a,
            err_b,
            args,
            &mut args_a,
            &mut args_b,
            #[cfg(feature = "autocomplete")]
            comp_items,
        )? {
            if res_b.is_some() {
                args.conflicts
                    .insert(args_b.head, (self.this.meta(), self.that.meta()));
            }
            Ok(res_a.unwrap())
        } else {
            if res_a.is_some() {
                args.conflicts
                    .insert(args_a.head, (self.that.meta(), self.this.meta()));
            }
            Ok(res_b.unwrap())
        }
    }

    fn meta(&self) -> Meta {
        self.this.meta().or(self.that.meta())
    }
}

fn this_or_that_picks_first(
    err_a: Option<Error>,
    err_b: Option<Error>,
    args: &mut Args,
    args_a: &mut Args,
    args_b: &mut Args,

    #[cfg(feature = "autocomplete")] mut comp_stash: Vec<crate::complete_gen::Comp>,
) -> Result<bool, Error> {
    // if higher depth parser succeeds - it takes a priority
    // completion from different depths should never mix either
    match Ord::cmp(&args_a.depth, &args_b.depth) {
        std::cmp::Ordering::Less => {
            std::mem::swap(args, args_b);
            #[cfg(feature = "autocomplete")]
            if let Some(comp) = &mut args.comp {
                comp.comps.extend(comp_stash);
            }
            return match err_b {
                Some(err) => Err(err),
                None => Ok(false),
            };
        }
        std::cmp::Ordering::Equal => {}
        std::cmp::Ordering::Greater => {
            std::mem::swap(args, args_a);
            #[cfg(feature = "autocomplete")]
            if let Some(comp) = &mut args.comp {
                comp.comps.extend(comp_stash);
            }
            return match err_a {
                Some(err) => Err(err),
                None => Ok(true),
            };
        }
    }

    #[cfg(feature = "autocomplete")]
    if let (Some(a), Some(b)) = (&mut args_a.comp, &mut args_b.comp) {
        comp_stash.extend(a.comps.drain(0..));
        comp_stash.extend(b.comps.drain(0..));
    }

    // otherwise pick based on the left most or successful one
    #[allow(clippy::let_and_return)]
    let res = match (err_a, err_b) {
        (None, None) => {
            if args_a.head <= args_b.head {
                std::mem::swap(args, args_a);
                Ok(true)
            } else {
                std::mem::swap(args, args_b);
                Ok(false)
            }
        }
        (None, Some(_)) => {
            std::mem::swap(args, args_a);
            Ok(true)
        }
        (Some(_), None) => {
            std::mem::swap(args, args_b);
            Ok(false)
        }
        (Some(e1), Some(e2)) => Err(e1.combine_with(e2)),
    };

    #[cfg(feature = "autocomplete")]
    if let Some(comp) = &mut args.comp {
        comp.comps.extend(comp_stash);
    }

    res
}

/// Parser that transforms parsed value with a failing function, created with
/// [`parse`](Parser::parse)
pub struct ParseWith<T, P, F, E, R> {
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
            Err(e) => Err(args.word_parse_error(&e.to_string())),
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
            Err(Error::Missing(_)) => {
                #[cfg(feature = "autocomplete")]
                std::mem::swap(&mut args.comp, &mut clone.comp);
                Ok(self.value.clone())
            }
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
            Err(args.word_validate_error(self.message))
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Apply inner parser, return a value in `Some` if items requested by it are all present, restore
/// and return `None` if any are missing. Created with [`optional`](Parser::optional). Implements
/// [`catch`](ParseOptional::catch)
pub struct ParseOptional<P> {
    pub(crate) inner: P,
    pub(crate) catch: bool,
}

impl<T, P> Parser<Option<T>> for ParseOptional<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<Option<T>, Error> {
        parse_option(&self.inner, args, self.catch)
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

impl<P> ParseOptional<P> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several structures that implement this attribute: [`ParseOptional`], [`ParseMany`]
    /// and [`ParseSome`], behavior should be identical for all of them.
    #[doc = include_str!("docs/catch.md")]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

/// Apply inner parser several times and collect results into `Vec`, created with
/// [`many`](Parser::many), implements [`catch`](ParseMany::catch).
pub struct ParseMany<P> {
    pub(crate) inner: P,
    pub(crate) catch: bool,
}

impl<P> ParseMany<P> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several structures that implement this attribute: [`ParseOptional`], [`ParseMany`]
    /// and [`ParseSome`], behavior should be identical for all of them.
    #[doc = include_str!("docs/catch.md")]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

fn parse_option<P, T>(parser: &P, args: &mut Args, catch: bool) -> Result<Option<T>, Error>
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
                Error::Stderr(_) if catch => Ok(None),
                Error::Stderr(_) | Error::Stdout(_) => Err(err),
                Error::Missing(_) => {
                    if args.len() == orig_args.len() {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                }
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
        while let Some(val) = parse_option(&self.inner, args, self.catch)? {
            if args.len() < len {
                len = args.len();
                res.push(val);
            } else {
                break;
            }
        }
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

pub struct ParsePureWith<T, F, E>(pub(crate) F)
where
    F: Fn() -> Result<T, E>,
    E: ToString;
impl<T: Clone + 'static, F: Fn() -> Result<T, E>, E: ToString> Parser<T>
    for ParsePureWith<T, F, E>
{
    fn eval(&self, _args: &mut Args) -> Result<T, Error> {
        match (self.0)() {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Stderr(e.to_string())),
        }
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
pub struct ParseCon<P> {
    /// inner parser closure
    pub inner: P,
    /// metas for inner parsers
    pub meta: Meta,
}

impl<T, P> Parser<T> for ParseCon<P>
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
    F: Fn(&T) -> Vec<(M, Option<M>)>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        // stash old
        let mut comp_items = Vec::new();
        args.swap_comps(&mut comp_items);

        let res = self.inner.eval(args);

        // restore old, now metavars added by inner parser, if any, are in comp_items
        args.swap_comps(&mut comp_items);

        if let Some(comp) = &mut args.comp {
            if res.is_err() {
                comp.comps.extend(comp_items);
                return res;
            }
        }

        let res = res?;

        // completion function generates suggestions based on the parsed inner value, for
        // that `res` must contain a parsed value
        if let Some(comp) = &mut args.comp {
            for ci in comp_items {
                if let Some(is_arg) = ci.meta_type() {
                    for (replacement, description) in (self.op)(&res) {
                        comp.push_value(
                            replacement.into(),
                            description.map(Into::into),
                            args.depth,
                            is_arg,
                        );
                    }
                } else {
                    comp.comps.push(ci);
                }
            }
        }
        Ok(res)
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}
#[cfg(feature = "autocomplete")]
pub struct ParseCompStyle<P> {
    pub(crate) inner: P,
    pub(crate) style: CompleteDecor,
}

#[cfg(feature = "autocomplete")]
impl<P, T> Parser<T> for ParseCompStyle<P>
where
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let mut comp_items = Vec::new();
        args.swap_comps(&mut comp_items);
        let res = self.inner.eval(args);
        args.swap_comps(&mut comp_items);
        extend_with_args_style(args, self.style, &mut comp_items);
        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

#[cfg(feature = "autocomplete")]
fn extend_with_args_style(
    args: &mut Args,
    style: CompleteDecor,
    comps: &mut Vec<crate::complete_gen::Comp>,
) {
    if let Some(comp) = &mut args.comp {
        for mut item in comps.drain(..) {
            item.set_decor(style);
            comp.comps.push(item);
        }
    }
}

pub struct ParseAdjacent<P> {
    pub(crate) inner: P,
}
impl<P, T> Parser<T> for ParseAdjacent<P>
where
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let mut guess = 0..args.items.len();
        let mut scratch = args.clone();
        let mut res = self.inner.eval(&mut scratch);
        let mut refined = true;
        while refined {
            refined = scratch.refine_range(args, &mut guess);
            scratch = args.clone();
            scratch.restrict_to_range(&guess);
            res = self.inner.eval(&mut scratch);
        }

        if guess.start > 0 {
            scratch.copy_usage_from(args, 0..guess.start);
        }
        if guess.end < args.items.len() {
            scratch.copy_usage_from(args, guess.end..args.items.len());
        }
        std::mem::swap(args, &mut scratch);
        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Create boxed parser
///
/// Boxed parser doesn't expose internal representation in it's type and allows to return
/// different parsers in different conditional branches
///
/// You can create it with a single argument `construct` macro:
#[doc = include_str!("docs/boxed.md")]
pub struct ParseBox<T> {
    /// Boxed inner parser
    pub inner: Box<dyn Parser<T>>,
}

impl<T> Parser<T> for ParseBox<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.eval(args)
    }
    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Parse anywhere
///
/// Most generic escape hatch available, in combination with [`any`] allows to parse anything
/// anywhere, works by repeatedly trying to run the inner parser on each subsequent context.
/// Can be expensive performance wise especially if parser contains complex logic.
///
#[doc = include_str!("docs/anywhere.md")]
pub struct ParseAnywhere<P> {
    pub(crate) inner: P,
}

impl<P, T> Parser<T> for ParseAnywhere<P>
where
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comp_items = Vec::new();

        for (start, mut this_arg) in args.ranges() {
            if let Ok(ok) = self.inner.eval(&mut this_arg) {
                if start > 0 {
                    this_arg.copy_usage_from(args, 0..start);
                }
                std::mem::swap(&mut this_arg, args);
                return Ok(ok);
            } else {
                #[cfg(feature = "autocomplete")]
                this_arg.swap_comps(&mut comp_items);
            }
        }

        #[cfg(feature = "autocomplete")]
        args.swap_comps(&mut comp_items);
        Err(Error::Missing(vec![]))
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}
