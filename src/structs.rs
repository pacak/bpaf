//! Structures that implement different methods on [`Parser`] trait
use crate::{
    args::State,
    buffer::MetaInfo,
    error::{Message, MissingItem},
    Doc, Error, Meta, Parser,
};
use std::marker::PhantomData;

/// Parser that substitutes missing value with a function results but not parser
/// failure, created with [`fallback_with`](Parser::fallback_with).
pub struct ParseFallbackWith<T, P, F, E> {
    pub(crate) inner: P,
    pub(crate) inner_res: PhantomData<T>,
    pub(crate) fallback: F,
    pub(crate) value_str: String,
    pub(crate) err: PhantomData<E>,
}

impl<T, P, F, E> Parser<T> for ParseFallbackWith<T, P, F, E>
where
    P: Parser<T>,
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let mut clone = args.clone();
        match self.inner.eval(&mut clone) {
            Ok(ok) => {
                std::mem::swap(args, &mut clone);
                Ok(ok)
            }
            Err(Error(e)) => {
                #[cfg(feature = "autocomplete")]
                args.swap_comps(&mut clone);
                if e.can_catch() {
                    match (self.fallback)() {
                        Ok(ok) => Ok(ok),
                        Err(e) => Err(Error(Message::PureFailed(e.to_string()))),
                    }
                } else {
                    Err(Error(e))
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        let m = Meta::Optional(Box::new(self.inner.meta()));
        if self.value_str.is_empty() {
            m
        } else {
            let buf = Doc::from(self.value_str.as_str());
            Meta::Suffix(Box::new(m), Box::new(buf))
        }
    }
}

/// Parser with attached message to several fields, created with [`group_help`](Parser::group_help).
pub struct ParseGroupHelp<P> {
    pub(crate) inner: P,
    pub(crate) message: Doc,
}

impl<T, P> Parser<T> for ParseGroupHelp<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comp_items = Vec::new();
        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comp_items);

        #[allow(clippy::let_and_return)]
        let res = self.inner.eval(args);

        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comp_items);
        #[cfg(feature = "autocomplete")]
        args.push_with_group(&self.message.to_completion(), &mut comp_items);

        res
    }

    fn meta(&self) -> Meta {
        let meta = Box::new(self.inner.meta());
        Meta::Subsection(meta, Box::new(self.message.clone()))
    }
}

/// Parser with attached message to several fields, created with [`group_help`](Parser::group_help).
pub struct ParseWithGroupHelp<P, F> {
    pub(crate) inner: P,
    pub(crate) f: F,
}

impl<T, P, F> Parser<T> for ParseWithGroupHelp<P, F>
where
    P: Parser<T>,
    F: Fn(MetaInfo) -> Doc,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        let meta = self.inner.meta();
        let buf = (self.f)(MetaInfo(&meta));

        Meta::Subsection(Box::new(meta), Box::new(buf))
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
    #[cfg_attr(not(doctest), doc = include_str!("docs2/some_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

impl<T, P> Parser<Vec<T>> for ParseSome<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<Vec<T>, Error> {
        let mut res = Vec::new();
        let mut len = usize::MAX;

        while let Some(val) = parse_option(&self.inner, &mut len, args, self.catch)? {
            res.push(val);
        }

        if res.is_empty() {
            Err(Error(Message::ParseSome(self.message)))
        } else {
            Ok(res)
        }
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Required(Box::new(self.inner.meta()))))
    }
}

/// Apply inner parser several times and collect results into `FromIterator`, created with
/// [`collect`](Parser::collect),
/// Implements [`catch`](ParseCollect::catch)
pub struct ParseCollect<P, C, T> {
    pub(crate) inner: P,
    pub(crate) catch: bool,
    pub(crate) ctx: PhantomData<(C, T)>,
}

impl<T, C, P> ParseCollect<P, C, T> {
    #[must_use]
    /// Handle parse failures
    ///
    /// Can be useful to decide to skip parsing of some items on a command line
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several structures that implement this attribute: [`ParseOptional`], [`ParseMany`]
    /// and [`ParseSome`], behavior should be identical for all of them.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/some_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

impl<T, C, P> Parser<C> for ParseCollect<P, C, T>
where
    P: Parser<T>,
    C: FromIterator<T>,
{
    fn eval(&self, args: &mut State) -> Result<C, Error> {
        let mut len = usize::MAX;
        std::iter::from_fn(|| parse_option(&self.inner, &mut len, args, self.catch).transpose())
            .collect::<Result<C, Error>>()
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Required(Box::new(self.inner.meta()))))
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comps = Vec::new();

        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comps);

        #[allow(clippy::let_and_return)]
        let res = self.inner.eval(args);

        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comps);
        if let Err(Error(Message::Missing(_))) = res {
            Err(Error(Message::Missing(Vec::new())))
        } else {
            res
        }
    }

    fn meta(&self) -> Meta {
        Meta::Skip
    }
}

/// Parser that hides inner parser from usage line
///
/// No other changes to the inner parser
pub struct ParseUsage<P> {
    pub(crate) inner: P,
    pub(crate) usage: Doc,
}
impl<T, P> Parser<T> for ParseUsage<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        Meta::CustomUsage(Box::new(self.inner.meta()), Box::new(self.usage.clone()))
    }
}

/// Parser that tries to either of two parsers and uses one that succeeeds, created with
/// [`Parser::or_else`].
pub struct ParseOrElse<T> {
    pub(crate) this: Box<dyn Parser<T>>,
    pub(crate) that: Box<dyn Parser<T>>,
}

impl<T> Parser<T> for ParseOrElse<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comp_items = Vec::new();
        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comp_items);

        // create forks for both branches
        // if they both fail - fallback to the original arguments
        // if they both succed - pick the one that consumes left, remember the second one
        // if one succeeds - pick that, forget the remaining one unless we are doing completion
        let mut args_a = args.clone();
        let mut args_b = args.clone();

        // run both parsers, expand Result<T, Error> into Option<T> + Option<Error>
        // so that code that does a bunch of comparing logic can be shared across
        // all invocations of parsers rather than being inlined into each one.

        let (res_a, err_a) = match self.this.eval(&mut args_a) {
            Ok(ok) => (Some(ok), None),
            Err(err) => (None, Some(err)),
        };

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
            Ok(res_a.unwrap())
        } else {
            Ok(res_b.unwrap())
        }
    }

    fn meta(&self) -> Meta {
        self.this.meta().or(self.that.meta())
    }
}

/// Given two possible errors along with to sets of arguments produce a new error or an instruction
/// to pick between two answers. Updates arguments state to match the results
fn this_or_that_picks_first(
    err_a: Option<Error>,
    err_b: Option<Error>,
    args: &mut State,
    args_a: &mut State,
    args_b: &mut State,

    #[cfg(feature = "autocomplete")] mut comp_stash: Vec<crate::complete_gen::Comp>,
) -> Result<bool, Error> {
    // if higher depth parser succeeds - it takes a priority
    // completion from different depths should never mix either
    match Ord::cmp(&args_a.depth(), &args_b.depth()) {
        std::cmp::Ordering::Less => {
            std::mem::swap(args, args_b);
            #[cfg(feature = "autocomplete")]
            if let Some(comp) = args.comp_mut() {
                comp.extend_comps(comp_stash);
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
            if let Some(comp) = args.comp_mut() {
                comp.extend_comps(comp_stash);
            }
            return match err_a {
                Some(err) => Err(err),
                None => Ok(true),
            };
        }
    }

    // otherwise pick based on the left most or successful one
    #[allow(clippy::let_and_return)] // <- it is without autocomplete only
    let res = match (err_a, err_b) {
        (None, None) => {
            if args.len() == args_a.len() && args.len() == args_b.len() {
                Ok((true, None))
            } else {
                Ok(args_a.pick_winner(args_b))
            }
        }
        (Some(e1), Some(e2)) => Err(e1.combine_with(e2)),
        // otherwise either a or b are success, true means a is success
        (a_ok, _) => Ok((a_ok.is_none(), None)),
    };

    #[cfg(feature = "autocomplete")]
    {
        let mut keep_a = true;
        let mut keep_b = true;
        if args_a.len() != args_b.len() {
            // If neither parser consumed anything - both can produce valid completions, otherwise
            // look for the first "significant" consume and keep that parser
            //
            // This is needed to preserve completion from a choice between a positional and a flag
            // See https://github.com/pacak/bpaf/issues/303 for more details
            if let (Some(_), Some(_)) = (args_a.comp_mut(), args_b.comp_mut()) {
                'check: for (ix, arg) in args_a.items.iter().enumerate() {
                    // During completion process named and unnamed arguments behave
                    // different - `-` and `--` are positional arguments, but we want to produce
                    // named items too. An empty string is also a special type of item that
                    // gets passed when user starts completion without passing any actual data.
                    //
                    // All other strings are either valid named items or valid positional items
                    // those are hopefully follow the right logic for being parsed/not parsed
                    if ix + 1 == args_a.items.len() {
                        let os = arg.os_str();
                        if os.is_empty() || os == "-" || os == "--" {
                            break 'check;
                        }
                    }
                    if let (Some(a), Some(b)) = (args_a.present(ix), args_b.present(ix)) {
                        match (a, b) {
                            (false, true) => {
                                keep_b = false;
                                break 'check;
                            }
                            (true, false) => {
                                keep_a = false;
                                break 'check;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let (Some(a), Some(b)) = (args_a.comp_mut(), args_b.comp_mut()) {
            if keep_a {
                comp_stash.extend(a.drain_comps());
            }
            if keep_b {
                comp_stash.extend(b.drain_comps());
            }
        }
    }

    match res {
        Ok((true, ix)) => {
            if let Some(win) = ix {
                args_a.save_conflicts(args_b, win);
            }
            std::mem::swap(args, args_a);
        }
        Ok((false, ix)) => {
            if let Some(win) = ix {
                args_b.save_conflicts(args_a, win);
            }
            std::mem::swap(args, args_b);
        }
        // no winner, keep the completions but don't touch args otherwise
        Err(_) => {}
    }

    #[cfg(feature = "autocomplete")]
    if let Some(comp) = args.comp_mut() {
        comp.extend_comps(comp_stash);
    }

    Ok(res?.0)
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
    fn eval(&self, args: &mut State) -> Result<R, Error> {
        let t = self.inner.eval(args)?;
        match (self.parse_fn)(t) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error(Message::ParseFailed(args.current, e.to_string()))),
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
    pub(crate) value_str: String,
}

impl<P, T> Parser<T> for ParseFallback<P, T>
where
    P: Parser<T>,
    T: Clone,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let mut clone = args.clone();
        match self.inner.eval(&mut clone) {
            Ok(ok) => {
                std::mem::swap(args, &mut clone);
                Ok(ok)
            }
            Err(Error(e)) => {
                #[cfg(feature = "autocomplete")]
                args.swap_comps(&mut clone);
                if e.can_catch() {
                    Ok(self.value.clone())
                } else {
                    Err(Error(e))
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        let m = Meta::Optional(Box::new(self.inner.meta()));
        if self.value_str.is_empty() {
            m
        } else {
            let buf = Doc::from(self.value_str.as_str());
            Meta::Suffix(Box::new(m), Box::new(buf))
        }
    }
}

/// An implementation detail for [`ParseFallback::format_fallback`] and
/// [`ParseFallbackWith::format_fallback`], to allow for custom fallback formatting.
struct DisplayWith<'a, T, F>(&'a T, F);

impl<'a, T, F: Fn(&'a T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result> std::fmt::Display
    for DisplayWith<'a, T, F>
{
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value, display) = self;
        display(value, f)
    }
}

impl<P, T: std::fmt::Display> ParseFallback<P, T> {
    /// Show [`fallback`](Parser::fallback) value in `--help` using [`Display`](std::fmt::Display)
    /// representation
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/dis_fallback.md"))]
    #[must_use]
    pub fn display_fallback(mut self) -> Self {
        self.value_str = format!("[default: {}]", self.value);
        self
    }
}

impl<P, T: std::fmt::Debug> ParseFallback<P, T> {
    /// Show [`fallback`](Parser::fallback) value in `--help` using [`Debug`](std::fmt::Debug)
    /// representation
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/deb_fallback_with.md"))]
    #[must_use]
    pub fn debug_fallback(mut self) -> Self {
        self.value_str = format!("[default: {:?}]", self.value);
        self
    }
}

impl<P, T> ParseFallback<P, T> {
    /// Show [`fallback`](Parser::fallback) value in `--help` using the provided formatting
    /// function.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/format_fallback.md"))]
    #[must_use]
    pub fn format_fallback(
        mut self,
        format: impl Fn(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Self {
        self.value_str = format!("[default: {}]", DisplayWith(&self.value, format));
        self
    }
}

impl<P, T: std::fmt::Display, F, E> ParseFallbackWith<T, P, F, E>
where
    F: Fn() -> Result<T, E>,
{
    /// Show [`fallback_with`](Parser::fallback_with) value in `--help` using [`Display`](std::fmt::Display)
    /// representation
    ///
    /// If fallback function fails - no value will show up
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/dis_fallback_with.md"))]
    #[must_use]
    pub fn display_fallback(mut self) -> Self {
        if let Ok(val) = (self.fallback)() {
            self.value_str = format!("[default: {}]", val);
        }
        self
    }
}

impl<P, T: std::fmt::Debug, F, E> ParseFallbackWith<T, P, F, E>
where
    F: Fn() -> Result<T, E>,
{
    /// Show [`fallback_with`](Parser::fallback_with) value in `--help` using [`Debug`](std::fmt::Debug)
    /// representation
    ///
    /// If fallback function fails - no value will show up
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/deb_fallback.md"))]
    #[must_use]
    pub fn debug_fallback(mut self) -> Self {
        if let Ok(val) = (self.fallback)() {
            self.value_str = format!("[default: {:?}]", val);
        }
        self
    }
}

impl<P, T, F, E> ParseFallbackWith<T, P, F, E>
where
    F: Fn() -> Result<T, E>,
{
    /// Show [`fallback_with`](Parser::fallback_with) value in `--help` using the provided
    /// formatting function.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/format_fallback_with.md"))]
    #[must_use]
    pub fn format_fallback(
        mut self,
        format: impl Fn(&T, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Self {
        if let Ok(val) = (self.fallback)() {
            self.value_str = format!("[default: {}]", DisplayWith(&val, format));
        }
        self
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let t = self.inner.eval(args)?;
        if (self.check)(&t) {
            Ok(t)
        } else {
            Err(Error(Message::GuardFailed(args.current, self.message)))
        }
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/// Apply inner parser as many times as it succeeds while consuming something and return this
/// number
pub struct ParseCount<P, T> {
    pub(crate) inner: P,
    pub(crate) ctx: PhantomData<T>,
}

impl<T, P> Parser<usize> for ParseCount<P, T>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<usize, Error> {
        let mut res = 0;
        let mut current = args.len();
        let mut len = usize::MAX;
        while (parse_option(&self.inner, &mut len, args, false)?).is_some() {
            res += 1;
            if current == args.len() {
                break;
            }
            current = args.len();
        }
        Ok(res)
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Optional(Box::new(self.inner.meta()))))
    }
}

/// Apply inner parser as many times as it succeeds while consuming something and return this
/// number
pub struct ParseLast<P> {
    pub(crate) inner: P,
}

impl<T, P> Parser<T> for ParseLast<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let mut last = None;
        let mut current = args.len();
        let mut len = usize::MAX;
        while let Some(val) = parse_option(&self.inner, &mut len, args, false)? {
            last = Some(val);
            if current == args.len() {
                break;
            }
            current = args.len();
        }
        if let Some(last) = last {
            Ok(last)
        } else {
            self.inner.eval(args)
        }
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Required(Box::new(self.inner.meta()))))
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
    fn eval(&self, args: &mut State) -> Result<Option<T>, Error> {
        let mut len = usize::MAX;
        parse_option(&self.inner, &mut len, args, self.catch)
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

impl<P> ParseOptional<P> {
    #[must_use]
    /// Handle parse failures for optional parsers
    ///
    /// Can be useful to decide to skip parsing of some items on a command line.
    /// When parser succeeds - `catch` version would return a value as usual
    /// if it fails - `catch` would restore all the consumed values and return None.
    ///
    /// There's several structures that implement this attribute: [`ParseOptional`], [`ParseMany`]
    /// and [`ParseSome`], behavior should be identical for all of them.
    ///
    /// Those examples are very artificial and designed to show what difference `catch` makes, to
    /// actually parse arguments like in examples you should [`parse`](Parser::parse) or construct
    /// enum with alternative branches
    #[cfg_attr(not(doctest), doc = include_str!("docs2/optional_catch.md"))]
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
    #[cfg_attr(not(doctest), doc = include_str!("docs2/many_catch.md"))]
    pub fn catch(mut self) -> Self {
        self.catch = true;
        self
    }
}

/// try to parse
fn parse_option<P, T>(
    parser: &P,
    len: &mut usize,
    args: &mut State,
    catch: bool,
) -> Result<Option<T>, Error>
where
    P: Parser<T>,
{
    let mut orig_args = args.clone();
    match parser.eval(args) {
        // we keep including values for as long as we consume values from the argument
        // list or at least one value
        Ok(val) => Ok(if args.len() < *len {
            *len = args.len();
            Some(val)
        } else {
            None
        }),
        Err(Error(err)) => {
            // this is safe to return Ok(None) in following scenarios
            // when inner parser never consumed anything and
            // 1. produced Error::Missing
            // 2. produced Error::Message(_, true)
            // 3. produced Error::Message and catch is enabled
            //
            // In all other scenarios we should return the original error
            //
            // When parser returns Ok(None) we should return the original arguments so if there's
            // anything left unconsumed - this won't be lost.

            let missing = matches!(err, Message::Missing(_));

            if catch || (missing && orig_args.len() == args.len()) || (!missing && err.can_catch())
            {
                std::mem::swap(&mut orig_args, args);
                #[cfg(feature = "autocomplete")]
                if orig_args.comp_mut().is_some() {
                    args.swap_comps(&mut orig_args);
                }
                Ok(None)
            } else {
                Err(Error(err))
            }
        }
    }
}

impl<T, P> Parser<Vec<T>> for ParseMany<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut State) -> Result<Vec<T>, Error> {
        let mut len = usize::MAX;
        std::iter::from_fn(|| parse_option(&self.inner, &mut len, args, self.catch).transpose())
            .collect::<Result<Vec<T>, Error>>()
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Optional(Box::new(self.inner.meta()))))
    }
}

/// Parser that returns a given value without consuming anything, created with
/// [`pure`](crate::pure).
pub struct ParsePure<T>(pub(crate) T);
impl<T: Clone + 'static> Parser<T> for ParsePure<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
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
    fn eval(&self, _args: &mut State) -> Result<T, Error> {
        match (self.0)() {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error(Message::PureFailed(e.to_string()))),
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        args.current = None;
        Err(Error(Message::ParseFail(self.field1)))
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
    fn eval(&self, args: &mut State) -> Result<R, Error> {
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
    /// To produce a better error messages while parsing constructed values
    /// we want to look at all the items so values that can be consumed are consumed
    /// autocomplete relies on the same logic
    ///
    /// However when dealing with adjacent restriction detecting the first item relies on failing
    /// fast
    pub failfast: bool,
}

impl<T, P> Parser<T> for ParseCon<P>
where
    P: Fn(bool, &mut State) -> Result<T, Error>,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let res = (self.inner)(self.failfast, args);
        args.current = None;
        res
    }

    fn meta(&self) -> Meta {
        self.meta.clone()
    }
}

impl<T> ParseCon<T> {
    #[must_use]

    /// Automagically restrict the inner parser scope to accept adjacent values only
    ///
    /// `adjacent` can solve surprisingly wide variety of problems: sequential command chaining,
    /// multi-value arguments, option-structs to name a few. If you want to run a parser on a
    /// sequential subset of arguments - `adjacent` might be able to help you. Check the examples
    /// for better intuition.
    ///
    /// Let's consider two examples with consumed items marked in bold and constructor containing
    /// parsers for `-c` and `-d`.
    ///
    /// - <code>**-a** -b **-c** -d</code>
    /// - <code>**-a** **-c** -b -d</code>
    ///
    /// In the first example `-b` breaks the adjacency for all the consumed items so parsing will fail,
    /// while here in the second one all the consumed items are adjacent to each other so
    /// parsing will succeed.
    ///
    /// # Multi-value arguments
    ///
    /// Parsing things like `--point X Y Z`
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_0.md"))]
    ///
    /// # Structure groups
    ///
    /// Parsing things like `--rect --width W --height H --rect --height H --width W`
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_1.md"))]
    ///
    /// # Chaining commands
    /// This example explains [`adjacent`](crate::params::ParseCommand::adjacent), but the same idea holds.
    /// Parsing things like `cmd1 --arg1 cmd2 --arg2 --arg3 cmd3 --flag`
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_command.md"))]
    ///
    /// # Capturing everything between markers
    ///
    /// Parsing things like `find . --exec foo {} -bar ; --more`
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_3.md"))]
    ///
    /// # Multi-value arguments with optional flags
    ///
    /// Parsing things like `--foo ARG1 --flag --inner ARG2`
    ///
    /// So you can parse things while parsing things. Not sure why you might need this, but you can
    /// :)
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_4.md"))]
    ///
    /// # Performance and other considerations
    ///
    /// `bpaf` can run adjacently restricted parsers multiple times to refine the guesses. It's
    /// best not to have complex inter-fields verification since they might trip up the detection
    /// logic: instead of restricting, for example "sum of two fields to be 5 or greater" *inside* the
    /// `adjacent` parser, you can restrict it *outside*, once `adjacent` done the parsing.
    ///
    /// There's also similar method [`adjacent`](crate::parsers::ParseArgument) that allows to restrict argument
    /// parser to work only for arguments where both key and a value are in the same shell word:
    /// `-f=bar` or `-fbar`, but not `-f bar`.
    pub fn adjacent(mut self) -> ParseAdjacent<Self> {
        self.failfast = true;
        ParseAdjacent { inner: self }
    }
}

/// Parser that replaces metavar placeholders with actual info in shell completion
#[cfg(feature = "autocomplete")]
pub struct ParseComp<P, F> {
    pub(crate) inner: P,
    pub(crate) op: F,
    pub(crate) group: Option<String>,
}

#[cfg(feature = "autocomplete")]
impl<P, F> ParseComp<P, F> {
    #[must_use]
    /// Attach group name to parsed values
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

#[cfg(feature = "autocomplete")]
impl<P, T, F, M> Parser<T> for ParseComp<P, F>
where
    P: Parser<T> + Sized,
    M: Into<String>,
    F: Fn(&T) -> Vec<(M, Option<M>)>,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        // stash old
        let mut comp_items = Vec::new();
        args.swap_comps_with(&mut comp_items);

        let res = self.inner.eval(args);

        // restore old, now metavars added by inner parser, if any, are in comp_items
        args.swap_comps_with(&mut comp_items);

        if let Some(comp) = &mut args.comp_mut() {
            if res.is_err() {
                comp.extend_comps(comp_items);
                return res;
            }
        }

        let res = res?;

        // completion function generates suggestions based on the parsed inner value, for
        // that `res` must contain a parsed value
        let depth = args.depth();
        if let Some(comp) = &mut args.comp_mut() {
            for ci in comp_items {
                let is_meta = ci.is_metavar();
                if let Some(is_arg) = is_meta {
                    let suggestions = (self.op)(&res);
                    // strip metavar when completion makes a single good suggestion
                    if suggestions.len() != 1 {
                        comp.push_comp(ci);
                    }
                    for (replacement, description) in suggestions {
                        let group = self.group.clone();
                        comp.push_value(
                            replacement.into(),
                            description.map(Into::into),
                            group,
                            depth,
                            is_arg,
                        );
                    }
                } else {
                    comp.push_comp(ci);
                }
            }
        }
        Ok(res)
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

/*
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let mut comp_items = Vec::new();
        args.swap_comps_with(&mut comp_items);
        let res = self.inner.eval(args);
        args.swap_comps_with(&mut comp_items);
        args.extend_with_style(self.style, &mut comp_items);
        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}*/

pub struct ParseAdjacent<P> {
    pub(crate) inner: P,
}
impl<P, T> Parser<T> for ParseAdjacent<P>
where
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        let original_scope = args.scope();

        let first_item;
        let inner_meta = self.inner.meta();
        let mut best_error = if let Some(item) = Meta::first_item(&inner_meta) {
            first_item = item;
            let missing_item = MissingItem {
                item: item.clone(),
                position: original_scope.start,
                scope: original_scope.clone(),
            };
            Message::Missing(vec![missing_item])
        } else {
            unreachable!("bpaf usage BUG: adjacent should start with a required argument");
        };
        let mut best_args = args.clone();
        let mut best_consumed = 0;

        for (start, width, mut this_arg) in args.ranges(first_item) {
            // since we only want to parse things to the right of the first item we perform
            // parsing in two passes:
            // - try to run the parser showing only single argument available at all the indices
            // - try to run the parser showing starting at that argument and to the right of it
            // this means constructing argument parsers from req flag and positional works as
            // expected:
            // consider examples "42 -n" and "-n 42"
            // without multi step approach first command line also parses into 42
            let mut scratch = this_arg.clone();
            scratch.set_scope(start..start + width);
            let before = scratch.len();

            // nothing to consume, might as well skip this segment right now
            // it will most likely fail, but it doesn't matter, we are only looking for the
            // left most match
            if before == 0 {
                continue;
            }

            let _ = self.inner.eval(&mut scratch);

            if before == scratch.len() {
                // failed to consume anything which means we don't start parsing at this point
                continue;
            }

            this_arg.set_scope(start..original_scope.end);
            let before = this_arg.len();

            // values consumed by adjacent must be actually adjacent - if a scope contains
            // already parsed values inside we need to trim it
            if original_scope.end - start > before {
                this_arg.set_scope(this_arg.adjacently_available_from(start));
            }

            loop {
                match self.inner.eval(&mut this_arg) {
                    Ok(res) => {
                        // there's a smaller adjacent scope, we must try it before returning.
                        if let Some(adj_scope) = this_arg.adjacent_scope(args) {
                            this_arg = args.clone();
                            this_arg.set_scope(adj_scope);
                        } else {
                            std::mem::swap(args, &mut this_arg);
                            args.set_scope(original_scope);
                            return Ok(res);
                        }
                    }
                    Err(Error(err)) => {
                        let consumed = before - this_arg.len();
                        if consumed > best_consumed {
                            best_consumed = consumed;
                            std::mem::swap(&mut best_args, &mut this_arg);
                            best_error = err;
                        }
                        break;
                    }
                }
            }
        }

        std::mem::swap(args, &mut best_args);
        Err(Error(best_error))
    }

    fn meta(&self) -> Meta {
        let meta = self.inner.meta();
        Meta::Adjacent(Box::new(meta))
    }
}

impl<T> Parser<T> for Box<dyn Parser<T>> {
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        self.as_ref().eval(args)
    }
    fn meta(&self) -> Meta {
        self.as_ref().meta()
    }
}
