//! Structures that implement different methods on [`Parser`] trait
use crate::{
    args::Conflict,
    error::{Message, MissingItem},
    item::Item,
    meta::DecorPlace,
    Args, Error, Meta, Parser,
};
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
        let mut clone = args.clone();
        match self.inner.eval(&mut clone) {
            Ok(ok) => {
                std::mem::swap(args, &mut clone);
                Ok(ok)
            }
            Err(e) => {
                #[cfg(feature = "autocomplete")]
                args.swap_comps(&mut clone);
                if e.can_catch() {
                    match (self.fallback)() {
                        Ok(ok) => Ok(ok),
                        Err(e) => Err(Error::Message(Message::PureFailed(e.to_string()))),
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        Meta::Optional(Box::new(self.inner.meta()))
    }
}

/// Parser with attached message to several fields, created with [`group_help`](Parser::group_help).
pub struct ParseGroupHelp<P> {
    pub(crate) inner: P,
    pub(crate) message: String,
}

impl<T, P> Parser<T> for ParseGroupHelp<P>
where
    P: Parser<T>,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        self.inner.eval(args)
    }

    fn meta(&self) -> Meta {
        let meta = Box::new(self.inner.meta());
        Meta::Decorated(meta, self.message.clone(), DecorPlace::Header)
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
            // we keep including values for as long as we consume values from the argument
            // list or at least one value
            if args.len() < len || res.is_empty() {
                len = args.len();
                res.push(val);
            } else {
                break;
            }
        }

        if res.is_empty() {
            Err(Error::Message(Message::ParseSome(self.message)))
        } else {
            Ok(res)
        }
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
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comps = Vec::new();

        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comps);

        #[allow(clippy::let_and_return)]
        let res = self.inner.eval(args);

        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comps);
        if let Err(Error::Missing(_)) = res {
            Err(Error::Missing(Vec::new()))
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
pub struct ParseOrElse<T> {
    pub(crate) this: Box<dyn Parser<T>>,
    pub(crate) that: Box<dyn Parser<T>>,
}

impl<T> Parser<T> for ParseOrElse<T> {
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        #[cfg(feature = "autocomplete")]
        let mut comp_items = Vec::new();
        #[cfg(feature = "autocomplete")]
        args.swap_comps_with(&mut comp_items);

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
                remember_conflict(args, self.this.meta(), &mut args_b, self.that.meta());
            } else {
                remember_winner(args, self.this.meta());
            }
            Ok(res_a.unwrap())
        } else {
            if res_a.is_some() {
                remember_conflict(args, self.that.meta(), &mut args_a, self.this.meta());
            } else {
                remember_winner(args, self.that.meta());
            }
            Ok(res_b.unwrap())
        }
    }

    fn meta(&self) -> Meta {
        self.this.meta().or(self.that.meta())
    }
}

fn remember_winner(args: &mut Args, meta: Meta) {
    args.conflicts
        .entry(args.head)
        .or_insert(Conflict::Solo(meta));
}

fn remember_conflict(args: &mut Args, winner: Meta, failed: &mut Args, loser: Meta) {
    let winner = args
        .conflicts
        .entry(args.head)
        .or_insert(Conflict::Solo(winner))
        .winner()
        .clone();

    let loser = failed
        .conflicts
        .get(&failed.head)
        .map(Conflict::winner)
        .cloned()
        .unwrap_or(loser);

    args.conflicts
        .entry(failed.head)
        .or_insert(Conflict::Conflicts(winner, loser));
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

    #[cfg(feature = "autocomplete")]
    if let (Some(a), Some(b)) = (args_a.comp_mut(), args_b.comp_mut()) {
        comp_stash.extend(a.drain_comps());
        comp_stash.extend(b.drain_comps());
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
    if let Some(comp) = args.comp_mut() {
        comp.extend_comps(comp_stash);
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
            Err(e) => Err(Error::Message(Message::ParseFailed(
                args.current,
                e.to_string(),
            ))),
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
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let mut clone = args.clone();
        match self.inner.eval(&mut clone) {
            Ok(ok) => {
                std::mem::swap(args, &mut clone);
                Ok(ok)
            }
            Err(e) => {
                #[cfg(feature = "autocomplete")]
                args.swap_comps(&mut clone);
                if e.can_catch() {
                    Ok(self.value.clone())
                } else {
                    Err(e)
                }
            }
        }
    }

    fn meta(&self) -> Meta {
        let m = Meta::Optional(Box::new(self.inner.meta()));
        if self.value_str.is_empty() {
            m
        } else {
            Meta::Decorated(
                Box::from(m),
                self.value_str.clone(),
                crate::meta::DecorPlace::Suffix,
            )
        }
    }
}

impl<P, T: std::fmt::Display> ParseFallback<P, T> {
    /// Show [`fallback`](Parser::fallback) value in `--help` using [`Display`](std::fmt::Display)
    /// representation
    #[must_use]
    pub fn display_fallback(mut self) -> Self {
        self.value_str = format!("[default: {}]", self.value);
        self
    }
}

impl<P, T: std::fmt::Debug> ParseFallback<P, T> {
    /// Show [`fallback`](Parser::fallback) value in `--help` using [`Debug`](std::fmt::Debug)
    /// representation
    #[must_use]
    pub fn debug_fallback(mut self) -> Self {
        self.value_str = format!("[default: {:?}]", self.value);
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
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        let t = self.inner.eval(args)?;
        if (self.check)(&t) {
            Ok(t)
        } else {
            Err(Error::Message(Message::ValidateFailed(
                args.current,
                self.message.to_string(),
            )))
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

/// try to parse
fn parse_option<P, T>(parser: &P, args: &mut Args, catch: bool) -> Result<Option<T>, Error>
where
    P: Parser<T>,
{
    let mut orig_args = args.clone();
    match parser.eval(args) {
        Ok(val) => Ok(Some(val)),
        Err(err) => {
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

            let res = match &err {
                Error::Message(msg) => {
                    if msg.can_catch() || catch {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                }
                Error::ParseFailure(_) => Err(err),
                Error::Missing(_) => {
                    if orig_args.len() == args.len() || catch {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                }
            };
            if res.is_ok() {
                std::mem::swap(&mut orig_args, args);

                #[cfg(feature = "autocomplete")]
                if orig_args.comp_mut().is_some() {
                    args.swap_comps(&mut orig_args);
                }
            }
            res
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
            // we keep including values for as long as we consume values from the argument
            // list or at least one value
            if args.len() < len || res.is_empty() {
                len = args.len();
                res.push(val);
            } else {
                break;
            }
        }
        Ok(res)
    }

    fn meta(&self) -> Meta {
        Meta::Many(Box::new(Meta::Optional(Box::new(self.inner.meta()))))
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
            Err(e) => Err(Error::Message(Message::PureFailed(e.to_string()))),
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
        Err(Error::Message(Message::ParseFail(self.field1)))
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

impl<T> ParseCon<T> {
    #[must_use]

    /// Automagically restrict the inner parser scope to accept adjacent values only
    ///
    /// `adjacent` can solve surprisingly wide variety of problems: sequential command chaining,
    /// multi-value arguments, option-structs to name a few. If you want to run a parser on a
    /// sequential subset of arguments - `adjacent` might be able to help you. Check the examples
    /// for better intuition.
    ///
    /// # Multi-value arguments
    ///
    /// Parsing things like `--foo ARG1 ARG2 ARG3`
    #[doc = include_str!("docs/adjacent_0.md")]
    ///
    /// # Structure groups
    ///
    /// Parsing things like `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`
    #[doc = include_str!("docs/adjacent_1.md")]
    ///
    /// # Chaining commands
    ///
    /// Parsing things like `cmd1 --arg1 cmd2 --arg2 --arg3 cmd3 --flag`
    ///
    #[doc = include_str!("docs/adjacent_2.md")]
    ///
    /// # Start and end markers
    ///
    /// Parsing things like `find . --exec foo {} -bar ; --more`
    ///
    #[doc = include_str!("docs/adjacent_3.md")]
    ///
    /// # Multi-value arguments with optional flags
    ///
    /// Parsing things like `--foo ARG1 --flag --inner ARG2`
    ///
    /// So you can parse things while parsing things. Not sure why you might need this, but you can
    /// :)
    ///
    #[doc = include_str!("docs/adjacent_4.md")]
    ///
    /// # Performance and other considerations
    ///
    /// `bpaf` can run adjacently restricted parsers multiple times to refine the guesses. It's
    /// best not to have complex inter-fields verification since they might trip up the detection
    /// logic: instead of destricting, for example "sum of two fields to be 5 or greater" *inside* the
    /// `adjacent` parser, you can restrict it *outside*, once `adjacent` done the parsing.
    ///
    /// `adjacent` is available on a trait for better discoverability, it doesn't make much sense to
    /// use it on something other than [`command`](crate::OptionParser::command) or [`construct!`](crate::construct!)
    /// encasing several fields.
    ///
    /// There's also similar method [`adjacent`](crate::parsers::ParseArgument) that allows to restrict argument
    /// parser to work only for arguments where both key and a value are in the same shell word:
    /// `-f=bar` or `-fbar`, but not `-f bar`.
    pub fn adjacent(self) -> ParseAdjacent<Self> {
        ParseAdjacent { inner: self }
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
        let depth = args.depth;
        if let Some(comp) = &mut args.comp_mut() {
            for ci in comp_items {
                if let Some(is_arg) = ci.meta_type() {
                    for (replacement, description) in (self.op)(&res) {
                        comp.push_value(
                            replacement.into(),
                            description.map(Into::into),
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
        args.swap_comps_with(&mut comp_items);
        let res = self.inner.eval(args);
        args.swap_comps_with(&mut comp_items);
        args.extend_with_style(self.style, &mut comp_items);
        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
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
        let original_scope = args.scope();

        let mut best_error = if let Some(item) = first_item(&self.inner.meta()) {
            let missing_item = MissingItem {
                item,
                position: original_scope.start,
                scope: original_scope.clone(),
            };
            Error::Missing(vec![missing_item])
        } else {
            unreachable!("bpaf usage BUG: adjacent should start with a required argument");
        };
        let mut best_args = args.clone();
        let mut best_consumed = 0;

        for (start, mut this_arg) in args.ranges() {
            // since we only want to parse things to the right of the first item we perform
            // parsing in two passes:
            // - try to run the parser showing only single argument available at all the indices
            // - try to run the parser showing starting at that argument and to the right of it
            // this means constructing argument parsers from req flag and positional works as
            // expected:
            // consider examples "42 -n" and "-n 42"
            // without multi step approach first command line also parses into 42
            let mut scratch = this_arg.clone();
            scratch.set_scope(start..start + 1);
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
                if let Some(adj_scope) = this_arg.adjacently_available_from(start) {
                    this_arg.set_scope(adj_scope);
                } else {
                    continue;
                }
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
                    Err(err) => {
                        let consumed = before - this_arg.len();
                        if consumed > best_consumed {
                            best_consumed = consumed;
                            std::mem::swap(&mut best_args, &mut this_arg);
                        }
                        best_error = err;
                        break;
                    }
                }
            }
        }

        fn first_item(meta: &Meta) -> Option<Item> {
            match meta {
                Meta::And(xs) => xs.first().and_then(first_item),
                Meta::Item(item) => Some(*item.clone()),
                Meta::Skip | Meta::Or(_) => None,
                Meta::Optional(x)
                | Meta::Required(x)
                | Meta::Adjacent(x)
                | Meta::Many(x)
                | Meta::Decorated(x, _, _)
                | Meta::HideUsage(x) => first_item(x),
            }
        }

        std::mem::swap(args, &mut best_args);
        Err(best_error)
    }

    fn meta(&self) -> Meta {
        let meta = self.inner.meta();
        Meta::Adjacent(Box::new(meta))
    }
}

/// Create boxed parser
///
/// Boxed parser doesn't expose internal representation in it's type and allows to return
/// different parsers in different conditional branches
///
/// You can create it with a single argument `construct` macro or by using `boxed` annotation
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
