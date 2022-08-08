#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![doc = include_str!("../README.md")]

use std::marker::PhantomData;

pub mod params;

mod args;
#[doc(hidden)]
pub mod info;
#[doc(hidden)]
pub mod item;
#[doc(hidden)]
pub mod meta;

pub mod structs;
use crate::{info::Error, item::Item};
pub use structs::ParseConstruct;
use structs::*;

#[cfg(test)]
mod tests;
#[doc(inline)]
pub use crate::args::Args;
pub use crate::info::{Info, OptionParser};
pub use crate::meta::Meta;
#[doc(inline)]
pub use crate::params::*;
#[doc(inline)]
#[cfg(feature = "bpaf_derive")]
pub use bpaf_derive::Bpaf;

/// Compose several parsers to produce a single result
///
/// Every parser must succeed in order to produce a result for
/// sequential composition and only one parser needs to succeed
/// for a parallel one (`construct!([a, b, c])`)
///
/// Each parser must be present in a local scope and
/// have the same name as struct field. Alternatively
/// a parser can be present as a function producing a parser
/// bpaf will call this function and use it's result. Later
/// option might be useful when single parser is used in
/// several `construct!` macros
///
/// ```rust
/// # use bpaf::*;
/// struct Res {
///     a: bool,
///     b: u32,
/// }
///
/// // parser defined as a local variable
/// let a = short('a').switch();
///
/// // parser defined as a function
/// fn b() -> impl Parser<u32> {
///     short('b').argument("B").from_str()
/// }
///
/// // resulting parser returns Res and requires both a and b to succeed
/// let res = construct!(Res { a, b() });
/// # drop(res);
/// ```
///
/// `construct!` supports following representations:
///
/// - structs with unnamed fields:
/// ```rust ignore
/// construct!(Res(a, b))
/// ```
/// - structs with named fields:
/// ```ignore
/// construct!(Res {a, b})
/// ```
/// - enums with unnamed fields:
/// ```ignore
/// construct!(Ty::Res(a, b))
/// ```
/// - enums with named fields:
/// ```ignore
/// construct!(Ty::Res {a, b})
/// ```
/// - tuples:
/// ```ignore
/// construct!(a, b)
/// ```
/// - parallel composition, a equivalent of `a.or_else(b).or_else(c)`
/// ```ignore
/// construct!([a, b, c])
/// ```
#[macro_export]
macro_rules! construct {
    // construct!(Enum::Cons { a, b, c })
    ($ns:ident $(:: $con:ident)* { $($tokens:tt)* }) => {{ $crate::construct!(@prepare [named [$ns $(:: $con)*]] [] $($tokens)*) }};
    (:: $ns:ident $(:: $con:ident)* { $($tokens:tt)* }) => {{ $crate::construct!(@prepare [named [:: $ns $(:: $con)*]] [] $($tokens)*) }};
    // construct!(Enum::Cons ( a, b, c ))
    ($ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) => {{ $crate::construct!(@prepare [pos [$ns $(:: $con)*]] [] $($tokens)*) }};
    (:: $ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) => {{ $crate::construct!(@prepare [pos [:: $ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!( a, b, c )
    ($first:ident , $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first , $($tokens)*) }};
    ($first:ident (), $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first (), $($tokens)*) }};

    // construct![a, b, c]
    ([$first:ident $($tokens:tt)*]) => {{ $crate::construct!(@prepare [alt] [] $first $($tokens)*) }};

    (@prepare $ty:tt [$($fields:tt)*] $field:ident (), $($rest:tt)*) => {{
        let $field = $field();
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident () $($rest:tt)*) => {{
        let $field = $field();
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident, $($rest:tt)*) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident $($rest:tt)*) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};

    (@prepare [alt] [$first:ident $($fields:ident)*]) => {{
        use $crate::Parser; $first $(.or_else($fields))*
    }};

    (@prepare $ty:tt [$($fields:tt)*]) => {{
        use $crate::Parser;
        let meta = $crate::Meta::And(vec![ $($fields.meta()),* ]);
        let inner = move |args: &mut $crate::Args| {
            $(let $fields = $fields.run(args)?;)*
            ::std::result::Result::Ok::<_, $crate::info::Error>
                ($crate::construct!(@make $ty [$($fields)*]))
        };
        $crate::ParseConstruct { inner, meta }
    }};

    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ { $($fields),* } };
    (@make [pos   [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ ( $($fields),* ) };
    (@make [pos] [$($fields:ident)*]) => { ( $($fields),* ) };
}

/// Simple or composed argument parser
pub trait Parser<T> {
    /// Parsing function
    fn run(&self, args: &mut Args) -> Result<T, Error>;

    /// Included information about the parser
    fn meta(&self) -> Meta;

    /// Consume zero or more items from a command line
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector which can be empty if no `-n` specified
    /// let n // n: impl Parser<Vec<u32>>
    ///     = short('n').argument("NUM").from_str::<u32>().many();
    /// # drop(n);
    /// ```
    ///
    /// # Panics
    /// Panics if parser succeeds without consuming any input: any parser modified with
    /// `many` must consume something,
    fn many(self) -> ParseMany<Self>
    where
        Self: Sized,
    {
        ParseMany { inner: self }
    }

    /// Parse stored [`String`] using [`FromStr`] instance
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let speed = short('s').argument("SPEED").from_str::<f64>();
    /// // at this point program would accept things like "-s 3.1415"
    /// // but reject "-s pi"
    /// # drop(speed)
    /// ```
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    fn from_str<R>(self) -> ParseFromStr<Self, R>
    where
        Self: Sized + Parser<T>,
    {
        ParseFromStr {
            inner: self,
            ty: PhantomData,
        }
    }

    /// Turn a required parser into optional
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // n: impl Parser<u32>
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// // if `-n` is not specified - parser will return `None`
    /// // n: impl Parser<Option<u32>>
    /// let n = n.optional();
    /// # drop(n);
    /// ```
    #[must_use]
    fn optional(self) -> ParseOptional<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseOptional { inner: self }
    }

    /// Validate or fail with a message
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// // Parser will reject values greater than 10
    /// let n = n.guard(|v| *v <= 10, "Values greater than 10 are only available in the DLC pack!");
    /// // n: impl Parser<u32>
    /// # drop(n);
    /// ```
    #[must_use]
    fn guard<F>(self, check: F, message: &'static str) -> ParseGuard<Self, F>
    where
        Self: Sized + Parser<T>,
        F: Fn(&T) -> bool,
    {
        ParseGuard {
            inner: self,
            check,
            message,
        }
    }

    /// Use this value as default if value is not present on a command line
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().fallback(42);
    /// # drop(n)
    /// ```
    #[must_use]
    fn fallback(self, value: T) -> ParseFallback<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        ParseFallback { inner: self, value }
    }

    /// Use value produced by this function as default if value is not present
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// let n = n.fallback_with(|| Result::<u32, String>::Ok(42));
    /// # drop(n)
    /// ```
    #[must_use]
    fn fallback_with<F, E>(self, fallback: F) -> ParseFallbackWith<T, Self, F, E>
    where
        Self: Sized + Parser<T>,
        F: Fn() -> Result<T, E>,
        E: ToString,
    {
        ParseFallbackWith {
            inner: self,
            inner_res: PhantomData,
            fallback,
            err: PhantomData,
        }
    }

    /// Parse `T` or fallback to `T::default()`
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().or_default();
    /// # drop(n)
    /// ```
    #[must_use]
    fn or_default(self) -> ParseDefault<T, Self>
    where
        Self: Sized + Parser<T>,
        T: Default + 'static + Clone,
    {
        ParseDefault {
            inner: self,
            inner_res: PhantomData,
        }
    }

    /// Apply a failing transformation
    ///
    /// See also [`from_str`][Parser::from_str]
    /// ```rust
    /// # use bpaf::*;
    /// let s = short('n').argument("NUM");
    /// // Try to parse String into u32 or fail during the parsing
    /// use std::str::FromStr;
    /// let n = s.map(|s| u32::from_str(&s));
    /// // n: impl Parser<u32>
    /// # drop(n);
    /// ```
    fn parse<F, E, R>(self, f: F) -> ParseWith<T, Self, F, E, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> Result<R, E>,
        E: ToString,
    {
        ParseWith {
            inner: self,
            inner_res: PhantomData,
            parse_fn: f,
            res: PhantomData,
            err: PhantomData,
        }
    }

    /// If first parser fails - try the second one
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let a = short('a').switch();
    /// let b = short('b').switch();
    ///
    /// // Parser will accept either `-a` or `-b` on a command line but not both at once.
    /// let a_or_b = a.or_else(b); // impl Parser<bool>
    /// # drop(a_or_b);
    /// ```
    ///
    /// # Performance
    ///
    /// If first parser succeeds - second one will be called anyway to produce a
    /// better error message for combinations of mutually exclusive parsers:
    ///
    /// Suppose program accepts one of two mutually exclusive switches `-a` and `-b`
    /// and both are present error message should point at the second flag
    ///
    /// [`construct!`] can be used to perform a similar task and might generate better code if
    /// combines more than two parsers. Those two invocations are equivalent:
    ///
    /// ```ignore
    /// let abc = a.or_else(b).or_else(c);
    /// ```
    /// ```ignore
    /// let abc = construct!([a, b, c]);
    /// ```
    ///
    fn or_else<P>(self, alt: P) -> ParseOrElse<Self, P>
    where
        Self: Sized + Parser<T>,
        P: Sized + Parser<T>,
    {
        ParseOrElse {
            this: self,
            that: alt,
        }
    }

    /// Apply a pure transformation to a contained value
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>(); // impl Parser<u32>
    /// // produced value is now twice as large
    /// let n = n.map(|v| v * 2);
    /// # drop(n);
    /// ```
    fn map<F, R>(self, map: F) -> ParseMap<T, Self, F, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> R + 'static,
    {
        ParseMap {
            inner: self,
            inner_res: PhantomData,
            map_fn: map,
            res: PhantomData,
        }
    }

    /// Ignore this parser during any sort of help generation
    ///
    /// Best used for optional parsers or parsers with a defined fallback
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // bpaf will accept `-w` but won't show it during help generation
    /// let width = short('w').argument("PX").from_str::<u32>().fallback(10).hide();
    /// let height = short('h').argument("PX").from_str::<u32>();
    /// let rect = construct!(width, height);
    /// # drop(rect);
    /// ```
    ///
    /// See also `examples/cargo-cmd.rs`
    fn hide(self) -> ParseHide<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHide { inner: self }
    }

    /// Consume one or more items from a command line
    ///
    /// Takes a string literal that will be used as an
    /// error message if there's not enough parameters specified
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector. At least one `-n` argument is required.
    /// // n: impl Parser<Vec<u32>>
    /// let n = short('n').argument("NUM")
    ///     .from_str::<u32>().some("You need to specify at least one number");
    /// # drop(n);
    /// ```
    #[must_use]
    fn some(self, message: &'static str) -> ParseSome<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseSome {
            inner: self,
            message,
        }
    }

    /// Attach help message to a complex parser
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let width = short('w').argument("PX").from_str::<u32>();
    /// let height = short('h').argument("PX").from_str::<u32>();
    /// let rect = construct!(width, height).group_help("take a rectangle");
    /// # drop(rect);
    /// ```
    /// See `examples/rectangle.rs` for a complete example
    fn group_help(self, message: &'static str) -> ParseGroupHelp<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseGroupHelp {
            inner: self,
            message,
        }
    }
}

/// Wrap a value into a `Parser`
///
/// Parser will produce `T` without consuming anything from the command line, can be useful
/// with [`construct!`].
///
/// ```rust
/// # use bpaf::*;
/// let a = long("flag-a").switch();
/// let b = pure(42u32);
/// let t = construct!(a, b); // impl Parser<(bool, u32)>
/// # drop(t)
/// ```
#[must_use]
pub fn pure<T>(val: T) -> ParsePure<T> {
    ParsePure(val)
}

/// Fail with a fixed error message
/// ```rust
/// # use bpaf::*;
/// let a = short('a').switch();
/// let no_a = fail("Custom error message for missing -a");
///
/// // Parser will produce a custom error message if `-a` is not specified
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

/*
impl<T> Parser<T> {
    /// Wrap a value into a `Parser`
    ///
    /// Parser will produce `T` without consuming anything from the command line, can be useful
    /// with [`construct!`].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let a = long("flag-a").switch();
    /// let b = Parser::pure(42u32);
    /// let t: Parser<(bool, u32)> = construct!(a, b);
    /// # drop(t)
    /// ```
    #[must_use]
    pub fn pure(val: T) -> Parser<T>
    where
        T: 'static + Clone,
    {
        let parse = move |i| Ok((val.clone(), i));
        Parser {
            parse: Rc::new(parse),
            meta: Meta::Skip,
        }
    }

    #[doc(hidden)]
    /// <*>
    #[must_use]
    pub fn ap<A, B>(self, other: Parser<A>) -> Parser<B>
    where
        T: Fn(A) -> B + 'static,
        A: 'static,
    {
        let parse = move |i| {
            let (t, rest) = (self.parse)(i)?;
            let (a, rest) = (other.parse)(rest)?;
            Ok((t(a), rest))
        };
        Parser {
            parse: Rc::new(parse),
            meta: Meta::And(vec![self.meta, other.meta]),
        }
    }

    /// If first parser fails - try the second one
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let a = short('a').switch();
    /// let b = short('b').switch();
    ///
    /// // Parser will accept either `-a` or `-b` on a command line but not both at once.
    /// let a_or_b: Parser<bool> = a.or_else(b);
    /// # drop(a_or_b);
    /// ```
    ///
    /// # Performance
    ///
    /// If first parser succeeds - second one will be called anyway to produce a
    /// better error message for combinations of mutually exclusive parsers:
    ///
    /// Suppose program accepts one of two mutually exclusive switches `-a` and `-b`
    /// and both are present error message should point at the second flag
    ///
    /// [`construct!`] can be used to perform a similar task and might generate better code if
    /// combines more than two parsers. Those two invocations are equivalent:
    ///
    /// ```ignore
    /// let abc = a.or_else(b).or_else(c);
    /// ```
    /// ```ignore
    /// let abc = construct!([a, b, c]);
    /// ```
    ///
    #[must_use]
    pub fn or_else(self, other: Parser<T>) -> Parser<T>
    where
        T: 'static,
    {
        let parse = move |mut i: Args| -> Result<(T, Args), Error> {
            i.head = usize::MAX;
            // To generate less confusing error messages give priority to the left most flag/argument
            // from the command line:
            // So if program accepts only one of 3 flags: -a, -b and -c and all 3 are present
            // take the first one and reject the remaining ones.
            let (res, new_args) = match ((self.parse)(i.clone()), (other.parse)(i)) {
                // side channel (--help) reporting takes priority
                (e @ Err(Error::Stdout(_)), _) | (_, e @ Err(Error::Stdout(_))) => e,
                (Ok((r1, a1)), Ok((r2, a2))) => {
                    if a1.head < a2.head {
                        Ok((r1, a1))
                    } else {
                        Ok((r2, a2))
                    }
                }
                (Ok(ok), Err(_)) | (Err(_), Ok(ok)) => Ok(ok),
                (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
            }?;
            Ok((res, new_args))
        };

        Parser {
            parse: Rc::new(parse),
            meta: self.meta.or(other.meta),
        }
    }

    /// Fail with a fixed error message
    /// ```rust
    /// # use bpaf::*;
    /// let a = short('a').switch();
    /// let no_a = Parser::fail("Custom error message for missing -a");
    ///
    /// // Parser will produce a custom error message if `-a` is not specified
    /// let a_: Parser<bool> = a.or_else(no_a);
    /// # drop(a_);
    /// ```
    #[must_use]
    pub fn fail<M>(msg: M) -> Parser<T>
    where
        String: From<M>,
        M: Clone + 'static,
    {
        Parser {
            meta: Meta::Skip,
            parse: Rc::new(move |_| Err(Error::Stderr(String::from(msg.clone())))),
        }
    }

    /// Consume zero or more items from a command line
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector which can be empty if no `-n` specified
    /// let n: Parser<Vec<u32>> = short('n').argument("NUM").from_str::<u32>().many();
    /// # drop(n);
    /// ```
    ///
    /// # Panics
    /// Panics if parser succeeds without consuming any input: any parser modified with
    /// `many` must consume something,
    #[must_use]
    pub fn many(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        let parse = move |mut i: Args| {
            let mut res = Vec::new();
            let mut size = i.len();
            while let Ok((elt, new_i)) = (self.parse)(i.clone()) {
                let new_size = new_i.len();
                #[allow(clippy::panic)]
                if new_size < size {
                    size = new_size;
                } else {
                    panic!("many can't be used with non failing parser")
                }
                i = new_i;
                res.push(elt);
            }
            Ok((res, i))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta.many(),
        }
    }

    /// Validate or fail with a message
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// // Parser will reject values greater than 10
    /// let n = n.guard(|v| *v <= 10, "Values greater than 10 are only available in the DLC pack!");
    /// # drop(n);
    /// ```
    #[must_use]
    pub fn guard<F>(self, m: F, message: &'static str) -> Parser<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let parse = move |i: Args| match (self.parse)(i) {
            Ok((ok, i)) if m(&ok) => Ok((ok, i)),
            Ok(_) => Err(Error::Stderr(message.to_owned())), // TODO - see what exactly we tried to parse
            Err(err) => Err(err),
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    /// Consume one or more items from a command line
    ///
    /// Takes a string literal that will be used as an
    /// error message if there's not enough parameters specified
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector. At least one `-n` argument is required.
    /// let n: Parser<Vec<u32>> = short('n').argument("NUM")
    ///     .from_str::<u32>().some("You need to specify at least one number");
    /// # drop(n);
    /// ```
    #[must_use]
    pub fn some(self, msg: &'static str) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        self.many().guard(|x| !x.is_empty(), msg)
    }

    /// Turn a required parser into optional
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n: Parser<u32> = short('n').argument("NUM").from_str();
    /// // if `-n` is not specified - parser will return `None`
    /// let n: Parser<Option<u32>> = n.optional();
    /// # drop(n);
    /// ```
    pub fn optional(self) -> Parser<Option<T>>
    where
        T: 'static + Clone,
    {
        self.map(Some).fallback(None)
    }

    /// Apply a pure transformation to a contained value
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n: Parser<u32> = short('n').argument("NUM").from_str();
    /// // produced value is now twice as large
    /// let n = n.map(|v| v * 2);
    /// # drop(n);
    /// ```
    pub fn map<F, B>(self, map: F) -> Parser<B>
    where
        F: Fn(T) -> B + 'static,
        T: 'static,
    {
        let parse = move |args: Args| {
            let (t, args) = (self.parse)(args)?;
            Ok((map(t), args))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    /// Apply a failing transformation
    ///
    /// See also [`from_str`][Parser::from_str]
    /// ```rust
    /// # use bpaf::*;
    /// let s: Parser<String> = short('n').argument("NUM");
    /// // Try to parse String into u32 or fail during the parsing
    /// use std::str::FromStr;
    /// let n = s.map(|s| u32::from_str(&s));
    /// # drop(n);
    /// ```
    pub fn parse<F, B, E>(self, map: F) -> Parser<B>
    where
        F: Fn(T) -> Result<B, E> + 'static,
        T: 'static,
        E: ToString,
    {
        let parse = move |args: Args| {
            let (t, args) = (self.parse)(args)?;

            match map(t) {
                Ok(ok) => Ok((ok, args)),
                Err(e) => Err(Error::Stderr(
                    if let Some(Word { utf8: Some(w), .. }) = args.current {
                        format!("Couldn't parse {:?}: {}", w, e.to_string())
                    } else {
                        format!("Couldn't parse: {}", e.to_string())
                    },
                )),
            }
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    /// Use this value as default if value is not present on a command line
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().fallback(42);
    /// # drop(n)
    /// ```
    #[must_use]
    pub fn fallback(self, val: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        let parse = move |i: Args| match (self.parse)(i.clone()) {
            Ok(ok) => Ok(ok),
            e @ Err(Error::Stderr(_) | Error::Stdout(_)) => e,
            Err(_) => Ok((val.clone(), i)),
        };
        Parser {
            parse: Rc::new(parse),
            meta: Meta::optional(self.meta),
        }
    }

    /// Use value produced by this function as default if value is not present
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// let n = n.fallback_with(|| Result::<u32, String>::Ok(42));
    /// # drop(n)
    /// ```
    #[must_use]
    pub fn fallback_with<F, E>(self, val: F) -> Parser<T>
    where
        F: Fn() -> Result<T, E> + Clone + 'static,
        E: ToString,
        T: Clone + 'static,
    {
        let parse = move |i: Args| match (self.parse)(i.clone()) {
            Ok(ok) => Ok(ok),
            e @ Err(Error::Stderr(_) | Error::Stdout(_)) => e,
            Err(_) => match val() {
                Ok(ok) => Ok((ok, i)),
                Err(e) => Err(Error::Stderr(e.to_string())),
            },
        };
        Parser {
            parse: Rc::new(parse),
            meta: Meta::optional(self.meta),
        }
    }

    /// Parse `T` or fallback to `T::default()`
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().default();
    /// # drop(n)
    /// ```
    #[must_use]
    pub fn default(self) -> Parser<T>
    where
        T: Default + 'static + Clone,
    {
        self.fallback(T::default())
    }

    /// Attach help message to a complex parser
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let width = short('w').argument("PX").from_str::<u32>();
    /// let height = short('h').argument("PX").from_str::<u32>();
    /// let rect = construct!(width, height).group_help("take a rectangle");
    /// # drop(rect);
    /// ```
    /// See `examples/rectangle.rs` for a complete example
    #[must_use]
    pub fn group_help(self, msg: &'static str) -> Parser<T> {
        Self {
            parse: self.parse,
            meta: Meta::decorate(self.meta, msg),
        }
    }

    /// Ignore this parser during any sort of help generation
    ///
    /// Best used for optional parsers or parsers with a defined fallback
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // bpaf will accept `-w` but won't show it during help generation
    /// let width = short('w').argument("PX").from_str::<u32>().fallback(10).hide();
    /// let height = short('h').argument("PX").from_str::<u32>();
    /// let rect = construct!(width, height);
    /// # drop(rect);
    /// ```
    ///
    /// See also `examples/cargo-cmd.rs`
    #[must_use]
    pub fn hide(self) -> Parser<T>
    where
        T: 'static,
    {
        Self {
            parse: Rc::new(move |args: Args| {
                (self.parse)(args).map_err(|_| Error::Missing(Vec::new()))
            }),
            meta: Meta::Skip,
        }
    }
}
*/

/*
impl Parser<String> {
    /// Parse stored [`String`] using [`FromStr`] instance
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let speed = short('s').argument("SPEED").from_str::<f64>();
    /// // at this point program would accept things like "-s 3.1415"
    /// // but reject "-s pi"
    /// # drop(speed)
    /// ```
    #[must_use]
    pub fn from_str<T>(self) -> Parser<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        self.parse(|s| T::from_str(&s))
    }
}*/

/// Unsuccessful command line parsing outcome
///
/// Useful for unit testing for user parsers, intented to
/// be consumed with [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`]
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Terminate and print this to stdout
    Stdout(String),
    /// Terminate and print this to stderr
    Stderr(String),
}

impl ParseFailure {
    /// Returns the contained `stderr` values
    ///
    /// Intended to be used with unit tests
    ///
    /// # Panics
    ///
    /// Will panic if failure contains `stdout`
    #[allow(clippy::must_use_candidate)]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Self::Stderr(err) => err,
            Self::Stdout(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    /// Returns the contained `stdout` values
    ///
    /// Intended to be used with unit tests
    ///
    /// # Panics
    ///
    /// Will panic if failure contains `stderr`
    #[allow(clippy::must_use_candidate)]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Self::Stdout(err) => err,
            Self::Stderr(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }
}

/*
impl<T> OptionParser<T> {
    /// Execute the [`OptionParser`], extract a parsed value or print some diagnostic and exit
    ///
    /// ```no_run
    /// # use bpaf::*;
    /// let verbose = short('v').req_flag(()).many().map(|xs|xs.len());
    /// let info = Info::default().descr("Takes verbosity flag and does nothing else");
    ///
    /// let opt = info.for_parser(verbose).run();
    /// // At this point `opt` contains number of repetitions of `-v` on a command line
    /// # drop(opt)
    /// ```
    #[must_use]
    pub fn run(self) -> T {
        let mut pos_only = false;
        let mut vec = Vec::new();
        for arg in std::env::args_os().skip(1) {
            args::push_vec(&mut vec, arg, &mut pos_only);
        }

        match self.run_inner(Args::from(vec)) {
            Ok(t) => t,
            Err(ParseFailure::Stdout(msg)) => {
                println!("{}", msg);
                std::process::exit(0);
            }
            Err(ParseFailure::Stderr(msg)) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
    }

    /// Execute the [`OptionParser`] and produce a value that can be used in unit tests
    ///
    /// ```
    /// #[test]
    /// fn positional_argument() {
    ///     let p = positional("FILE").help("File to process");
    ///     let parser = Info::default().for_parser(p);
    ///
    ///     let help = parser
    ///         .run_inner(Args::from(&["--help"]))
    ///         .unwrap_err()
    ///         .unwrap_stdout();
    ///     let expected_help = "\
    /// Usage: <FILE>
    ///
    /// Available options:
    ///     -h, --help   Prints help information
    /// ";
    ///     assert_eq!(expected_help, help);
    /// }
    /// ```
    ///
    /// See also [`Args`] and it's `From` impls to produce input and
    /// [`ParseFailure::unwrap_stderr`] / [`ParseFailure::unwrap_stdout`] for processing results.
    ///
    /// # Errors
    ///
    /// If parser can't produce desired outcome `run_inner` will return [`ParseFailure`]
    /// which represents runtime behavior: one branch to print something to stdout and exit with
    /// success and the other branch to print something to stderr and exit with failure.
    ///
    /// Parser is not really capturing anything. If parser detects `--help` or `--version` it will
    /// always produce something that can be consumed with [`ParseFailure::unwrap_stdout`].
    /// Otherwise it will produce [`ParseFailure::unwrap_stderr`]  generated either by the parser
    /// itself in case someone required field is missing or by user's [`Parser::guard`] or
    /// [`Parser::parse`] functions.
    ///
    /// API for those is constructed to only produce a [`String`]. If you try to print something inside
    /// [`Parser::map`] or [`Parser::parse`] - it will not be captured. Depending on a test case
    /// you'll know what to use: `unwrap_stdout` if you want to test generated help or `unwrap_stderr`
    /// if you are testing `parse` / `guard` / missing parameters.
    ///
    /// Exact string reperentations may change between versions including minor releases.
    pub fn run_inner(self, args: Args) -> Result<T, ParseFailure> {
        match (self.parse)(args) {
            Ok((t, rest)) if rest.is_empty() => Ok(t),
            Ok((_, rest)) => Err(ParseFailure::Stderr(format!("unexpected {:?}", rest))),
            Err(Error::Missing(metas)) => Err(ParseFailure::Stderr(format!(
                "Expected {}, pass --help for usage information",
                Meta::Or(metas)
            ))),
            Err(Error::Stdout(stdout)) => Err(ParseFailure::Stdout(stdout)),
            Err(Error::Stderr(stderr)) => Err(ParseFailure::Stderr(stderr)),
        }
    }
}
*/
/// Strip a command name if present at the front when used as a cargo command
///
/// This helper should be used on a top level parser
///
/// ```rust
/// # use bpaf::*;
/// let width = short('w').argument("PX").from_str::<u32>();
/// let height = short('h').argument("PX").from_str::<u32>();
/// let parser = cargo_helper("cmd", construct!(width, height)); // impl Parser<(u32, u32)>
/// # drop(parser);
/// ```
#[must_use]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    T: 'static,
    P: Parser<T>,
{
    let skip = positional_if("", move |s| cmd == s).hide();
    construct!(skip, parser).map(|x| x.1)
}

/*
fn foo() -> impl {
    use crate::Parser;
    let meta = crate::Meta::And((<[_]>::into_vec(box [(skip.meta()), (parser.meta())])));
    let inner = move |args: &mut crate::Args| {
        let skip = skip.run(args)?;
        let parser = parser.run(args)?;
        ::std::result::Result::Ok::<_, crate::info::Error>((args, ((skip, parser))))
    };
    crate::ParseConstruct { inner, meta }
}*/
