#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::redundant_else)] // not useful
#![allow(rustdoc::redundant_explicit_links)] // two random markdown parsers I tried only supports explicit links
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Lightweight and flexible command line argument parser with derive and combinatoric style API

//! # Quick links
//! - [Introduction](_documentation::_0_intro) - features, design goals, restrictions
//! - [Tutorials](_documentation::_1_tutorials) - practical learning oriented information and
//!   examples to get you started
//!   + [Types of arguments](_documentation::_1_tutorials::_0_types_of_arguments) -
//!     common types of line options and conventions (optional)
//!   + [Combinatoric API](_documentation::_1_tutorials::_1_combinatoric_api)  -
//!     Parse arguments without using proc macros
//!   + [Derive API](_documentation::_1_tutorials::_2_derive_api) -
//!     Create a parser by defining a structure
//! - [How-to and guides](_documentation::_2_howto) - assumes familiarity with the basics and
//!   explains how to concrete tasks
//! - [Explanations](_documentation::_4_explanation) - theoretical information about abstractions
//!   used by the library, oriented for understanding
//! - [FAQ](https://github.com/pacak/bpaf/discussions) - questions from library users

//! # A quick start
//!
//! Add `bpaf`, optionally with derive enabled
//!
//! ```text
//! $ cargo add bpaf -F derive,dull_color
//! ```
//!
//! Use either derive or combinatoric API and try running it
//!
#![cfg_attr(not(doctest), doc = include_str!("docs2/intro.md"))]

//!
//! ## Consuming items - making `Parser`
//!
//! `bpaf` allows you to describe the parsers using a mix of two APIs: combinatoric and derive.
//! Both APIs can achieve the same results, you can use one that better suits your needs. You can
//! find documentation with more examples following those links.
//!
//! - For an argument with a name you define [`NamedArg`] using a combination of [`short`],
//!   [`long`] and [`env`](crate::env()). At the same time you can attach
//!   [`help`](NamedArg::help).
//! - [`NamedArg::switch`] - simple switch that returns `true` if it's present on a command
//!   line and `false` otherwise.
//! - [`NamedArg::flag`] - a variant of `switch` that lets you return one of two custom
//!   values, for example `Color::On` and `Color::Off`.
//! - [`NamedArg::req_flag`] - a variant of `switch` that only only succeeds when it's name
//!   is present on a command line
//! - [`NamedArg::argument`] - named argument containing a value, you can further
//!   customize it with [`adjacent`](crate::parsers::ParseArgument::adjacent)
//! - [`positional`] - positional argument, you can further customize it with
//!   [`strict`](ParsePositional::strict)
//! - [`OptionParser::command`] - subcommand parser.
//! - [`any`] and its specialized version [`literal`] are escape hatches that can parse anything
//!   not fitting into usual classification.
//! - [`pure`] and [`pure_with`] - a way to generate a value that can be composed without parsing
//!   it from the command line.
//!
//! ## Transforming and changing parsers
//!
//! By default primitive parsers gives you back a single `bool`, a single `PathBuf` or a single
//! value produced by [`FromStr`] trait, etc. You can further transform it by chaining methods from
//! [`Parser`] trait, some of those methods are applied automagically if you are using derive API.
//!
//! `bpaf` distinguishes two types of parse failures - "value is absent" and
//! "value is present but invalid", most parsers listed in this section only handle the first
//! type of failure by default, but you can use their respective `catch` method to handle the later
//! one.
//!
//! - [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with) - return a
//!   different value if parser fails to find what it is looking for. Generated help for former
//!   can be updated to include default value using
//!   [`display_fallback`](ParseFallback::display_fallback),
//!   [`debug_fallback`](ParseFallback::debug_fallback), or
//!   [`format_fallback`](ParseFallback::format_fallback).
//! - [`optional`](Parser::optional) - return `None` if value is missing instead of failing, see
//!   also [`catch`](ParseOptional::catch) .
//! - [`many`](Parser::many), [`some`](Parser::some) and [`collect`](Parser::collect) - collect
//!   multiple values into a collection, usually a vector, see their respective
//!   [`catch`](ParseMany::catch), [`catch`](ParseSome::catch) and [`catch`](ParseCollect::catch).
//! - [`map`](Parser::map), [`parse`](Parser::parse) and [`guard`](Parser::guard) - transform
//!   and/or validate value produced by a parser
//! - [`to_options`](Parser::to_options) - finalize the parser and prepare to run it
//!
//! ## Combining multiple parsers together
//!
//! Once you have parsers for all the primitive fields figured out you can start combining them
//! together to produce a parser for a final result - data type you designed in the step one.
//! For derive API you apply annotations to data types with `#[derive(Bpaf)`] and `#[bpaf(..)]`,
//! with combinatoric API you use [`construct!`](crate::construct!) macro.
//!
//! All fields in a struct needs to be successfully parsed in order for the parser to succeed
//! and only one variant from enum will consume its values at a time.
//!
//! You can use [`adjacent`](ParseCon::adjacent) annotation to parse multiple flags as an adjacent
//! group allowing for more unusual scenarios such as multiple value arguments or chained commands.
//!
//! ## Improving user experience
//!
//! `bpaf` would use doc comments on fields and structures in derive mode and and values passed
//! in various `help` methods to generate `--help` documentation, you can further improve it
//! using those methods:
//!
//! - [`hide_usage`](Parser::hide_usage) and [`hide`](Parser::hide) - hide the parser from
//!   generated *Usage* line or whole generated help
//! - [`group_help`](Parser::group_help) and [`with_group_help`](Parser::with_group_help) -
//!   add a common description shared by several parsers
//! - [`custom_usage`](Parser::custom_usage) - customize usage for a primitive or composite parser
//! - [`usage`](OptionParser::usage) and [`with_usage`](OptionParser::with_usage) lets you to
//!   customize whole usage line as a whole either by completely overriding it or by building around it.
//!
//! By default with completion enabled `bpaf` would complete names for flags, arguments and
//! commands. You can also generate completion for argument values, possible positionals, etc.
//! This requires enabling **autocomplete** cargo feature.
//!
//! - [`complete`](Parser::complete) and [`complete_shell`](Parser::complete_shell)
//!
//! And finally you can generate documentation for command line in markdown, html and manpage
//! formats using [`render_markdown`](OptionParser::render_markdown),
//! [`render_html`](OptionParser::render_html) and [`render_manpage`](OptionParser::render_manpage),
//! for more detailed info see [`doc`] module
//!
//! ## Testing your parsers and running them
//! - You can [`OptionParser::run`] the parser on the arguments passed on the command line
//! - [`check_invariants`](OptionParser::check_invariants) checks for a few invariants in the
//!   parser `bpaf` relies on
//! - [`run_inner`](OptionParser::run_inner) runs the parser with custom [`Args`] you can create
//!   either explicitly or implicitly using one of the [`From`] implementations, `Args` can be
//!   customized with [`set_comp`](Args::set_comp) and [`set_name`](Args::set_name).
//! - [`ParseFailure`] contains the parse outcome, you can consume it either by hands or using one
//!   of [`exit_code`](ParseFailure::exit_code), [`unwrap_stdout`](ParseFailure::unwrap_stdout) and
//!   [`unwrap_stderr`](ParseFailure::unwrap_stderr)
//!
//! ## Cargo features
//!
//!  - `derive`: adds a dependency on `bpaf_derive` crate and reexport `Bpaf` derive macro. You
//!     need to enable it to use derive API. Disabled by default.
//!
//!  - `batteries`: helpers implemented with public `bpaf` API. Disabled by default.
//!
//!  - `autocomplete`: enables support for shell autocompletion. Disabled by default.
//!
//!
//! - `bright-color`, `dull-color`: use more colors when printing `--help` and such. Enabling
//!   either color feature adds some extra dependencies and might raise MRSV. If you are planning
//!   to use this feature in a published app - it’s best to expose them as feature flags:
//!
//!    ```toml
//!    [features]
//!    bright-color = ["bpaf/bright-color"]
//!    dull-color = ["bpaf/dull-color"]
//!    ```
//!    Disabled by default.
//!
//!  - `docgen`: generate documentation from help declaration, see [`OptionParser::render_markdown`] and [`doc`](crate::doc). Disabled by default.



#[cfg(feature = "extradocs")]
#[rustfmt::skip]
#[allow(unused_imports)]
pub mod _documentation;

mod arg;
mod args;
#[cfg(feature = "batteries")]
pub mod batteries;
mod buffer;
#[cfg(feature = "autocomplete")]
mod complete_gen;
#[cfg(feature = "autocomplete")]
mod complete_run;
#[cfg(feature = "autocomplete")]
mod complete_shell;
pub mod doc;
mod error;
mod from_os_str;
mod info;
mod item;
mod meta;
mod meta_help;
mod meta_youmean;
pub mod params;
mod structs;
#[cfg(test)]
mod tests;

pub mod parsers {
    //! This module exposes parsers that accept further configuration with builder pattern
    //!
    //! In most cases you won't be using those names directly, they're only listed here to provide
    //! access to documentation
    #[cfg(feature = "autocomplete")]
    #[doc(inline)]
    pub use crate::complete_shell::ParseCompShell;
    #[doc(inline)]
    pub use crate::params::{
        NamedArg, ParseAny, ParseArgument, ParseCommand, ParseFlag, ParsePositional,
    };
    #[doc(inline)]
    pub use crate::structs::{
        ParseCollect, ParseCon, ParseCount, ParseFallback, ParseFallbackWith, ParseLast, ParseMany,
        ParseOptional, ParseSome,
    };
}

// -------------------------------------------------------------------

#[doc(inline)]
pub use crate::{args::Args, buffer::Doc, error::ParseFailure, info::OptionParser};

#[doc(hidden)]
// used by construct macro, not part of public API
pub use crate::{args::State, error::Error, meta::Meta, structs::ParseCon};

use std::{marker::PhantomData, str::FromStr};

use crate::{
    buffer::{MetaInfo, Style},
    item::Item,
    params::build_positional,
    parsers::{NamedArg, ParseAny, ParseCommand, ParsePositional},
    structs::{
        ParseCollect, ParseCount, ParseFail, ParseFallback, ParseFallbackWith, ParseGroupHelp,
        ParseGuard, ParseHide, ParseLast, ParseMany, ParseMap, ParseOptional, ParseOrElse,
        ParsePure, ParsePureWith, ParseSome, ParseUsage, ParseWith, ParseWithGroupHelp,
    },
};

#[cfg(feature = "autocomplete")]
pub use crate::complete_shell::ShellComp;
#[cfg(feature = "autocomplete")]
use structs::ParseComp;

#[doc(inline)]
#[cfg(feature = "bpaf_derive")]
pub use bpaf_derive::Bpaf;

/// Compose several parsers to produce a single result
///
/// # Usage reference
/// ```rust
/// # use bpaf::*;
/// # { struct Res(bool, bool, bool);
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // structs with unnamed fields:
/// construct!(Res(a, b, c));
/// # }
///
/// # { struct Res { a: bool, b: bool, c: bool }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // structs with named fields:
/// construct!(Res {a, b, c});
/// # }
///
/// # { enum Ty { Res(bool, bool, bool) }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // enums with unnamed fields:
/// construct!(Ty::Res(a, b, c));
/// # }
///
/// # { enum Ty { Res { a: bool, b: bool, c: bool } }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // enums with named fields:
/// construct!(Ty::Res {a, b, c});
/// # }
///
/// # { let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // tuples:
/// construct!(a, b, c);
/// # }
///
/// # { let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // parallel composition, tries all parsers, picks one that consumes the left most value,
/// // or if they consume the same (or not at all) - the left most in a list
/// construct!([a, b, c]);
/// # }
///
/// // defining primitive parsers inside construct macro :)
/// construct!(a(short('a').switch()), b(long("arg").argument::<usize>("ARG")));
///
/// # { let a = short('a').switch();
/// // defining a boxed parser
/// construct!(a);
/// # }
/// ```
///
/// # Combinatoric usage
/// `construct!` can compose parsers sequentially or in parallel.
///
/// Sequential composition runs each parser and if all of them succeed you get a parser object of a
/// new type back. Placeholder names for values inside `construct!` macro must correspond to both
/// struct/enum names and parser names present in scope. In examples below `a` corresponds to a
/// function and `b` corresponds to a variable name. Note parens in `a()`, you must to use them to
/// indicate function parsers.
///
/// Inside the parens you can put a whole expression to use instead of
/// having to define them in advance: `a(positional::<String>("POS"))`. Probably a good idea to use this
/// approach only for simple parsers.
///
/// ```rust
/// # use bpaf::*;
/// struct Res (u32, u32);
/// enum Ul { T { a: u32, b: u32 } }
///
/// // You can share parameters across multiple construct invocations
/// // if defined as functions
/// fn a() -> impl Parser<u32> {
///     short('a').argument::<u32>("N")
/// }
///
/// // You can construct structs or enums with unnamed fields
/// fn res() -> impl Parser<Res> {
///     let b = short('b').argument::<u32>("n");
///     construct!(Res ( a(), b ))
/// }
///
/// // You can construct structs or enums with named fields
/// fn ult() -> impl Parser<Ul> {
///     let b = short('b').argument::<u32>("n");
///     construct!(Ul::T { a(), b })
/// }
///
/// // You can also construct simple tuples
/// fn tuple() -> impl Parser<(u32, u32)> {
///     let b = short('b').argument::<u32>("n");
///     construct!(a(), b)
/// }
///
/// // You can create boxed version of parsers so the type matches as long
/// // as return type is the same - can be useful for all sort of dynamic parsers
/// fn boxed() -> Box<dyn Parser<u32>> {
///     let a = short('a').argument::<u32>("n");
///     construct!(a)
/// }
///
/// // In addition to having primitives defined before using them - you can also define
/// // them directly inside construct macro. Probably only a good idea if you have only simple
/// // components
/// struct Options {
///     arg: u32,
///     switch: bool,
/// }
///
/// fn coyoda() -> impl Parser<Options> {
///     construct!(Options {
///         arg(short('a').argument::<u32>("ARG")),
///         switch(short('s').switch())
///     })
/// }
/// ```
///
/// Parallel composition picks one of several available parsers (result types must match) and returns a
/// parser object of the same type. Similar to sequential composition you can use parsers from variables
/// or functions:
///
/// ```rust
/// # use bpaf::*;
/// fn b() -> impl Parser<u32> {
///     short('b').argument::<u32>("NUM")
/// }
///
/// fn a_or_b() -> impl Parser<u32> {
///     let a = short('a').argument::<u32>("NUM");
///     // equivalent way of writing this would be `a.or_else(b())`
///     construct!([a, b()])
/// }
/// ```
///
/// # Derive usage
///
/// `bpaf` would combine fields of struct or enum constructors sequentially and enum
/// variants in parallel.
/// ```rust
/// # use bpaf::*;
/// // to satisfy this parser user needs to pass both -a and -b
/// #[derive(Debug, Clone, Bpaf)]
/// struct Res {
///     a: u32,
///     b: u32,
/// }
///
/// // to satisfy this parser user needs to pass one (and only one) of -a, -b, -c or -d
/// #[derive(Debug, Clone, Bpaf)]
/// enum Enumeraton {
///     A { a: u32 },
///     B { b: u32 },
///     C { c: u32 },
///     D { d: u32 },
/// }
///
/// // here user needs to pass either both -a AND -b or both -c AND -d
/// #[derive(Debug, Clone, Bpaf)]
/// enum Ult {
///     AB { a: u32, b: u32 },
///     CD { c: u32, d: u32 }
/// }
/// ```

#[macro_export]
macro_rules! construct {
    // construct!(Enum::Cons { a, b, c })
    ($(::)? $ns:ident $(:: $con:ident)* { $($tokens:tt)* }) => {{ $crate::construct!(@prepare [named [$ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!(Enum::Cons ( a, b, c ))
    ($(::)? $ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) => {{ $crate::construct!(@prepare [pos [$ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!( a, b, c )
    ($first:ident $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first $($tokens)*) }};

    // construct![a, b, c]
    ([$first:ident $($tokens:tt)*]) => {{ $crate::construct!(@prepare [alt] [] $first $($tokens)*) }};

    (@prepare $ty:tt [$($fields:tt)*] $field:ident () $(, $($rest:tt)*)? ) => {{
        let $field = $field();
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)*)?)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident ($expr:expr) $(, $($rest:tt)*)?) => {{
        let $field = $expr;
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)*)?)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)* )?)
    }};

    (@prepare [alt] [$first:ident $($fields:ident)*]) => {
        #[allow(deprecated)]
        { use $crate::Parser; $first $(.or_else($fields))* }
    };

    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::construct!(@fin $ty [ $($fields)* ])
    };

    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ { $($fields),* } };
    (@make [pos   [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ ( $($fields),* ) };
    (@make [pos] [$($fields:ident)*]) => { ( $($fields),* ) };


    (@fin [named [$($con:tt)+]] []) => { $crate::pure($($con)+ { })};
    (@fin [pos   [$($con:tt)+]] []) => { $crate::pure($($con)+ ( ))};

    (@fin [pos] [$field:ident]) => {{
        use $crate::Parser;
        $field.boxed()
    }};

    (@fin $ty:tt [$front:ident $($fields:ident)*]) => {{
        use $crate::Parser;
        let meta = $crate::Meta::And(vec![ $front.meta(), $($fields.meta()),* ]);
        let inner = move |failfast: bool, args: &mut $crate::State| {
            let mut $front = $front.eval(args);
            if failfast {
                $front = Ok($front?);
            }
            $(let $fields = $fields.eval(args);)*
            let $front = $front?;
            $(let $fields = $fields?;)*

            args.current = None;
            ::std::result::Result::Ok::<_, $crate::Error>
                ($crate::construct!(@make $ty [$front $($fields)*]))
        };
        $crate::ParseCon { inner, meta, failfast: false }
    }};
}

/// Simple or composed argument parser
///
/// # Overview
///
/// It's best to think of an object implementing [`Parser`] trait as a container with a value
/// inside that is composable with other `Parser` containers using [`construct!`] and the only
/// way to extract this value is by transforming it to [`OptionParser`] with
/// [`to_options`](Parser::to_options) and running it with [`run`](OptionParser::run). At which
/// point you either get your value out or `bpaf` would generate a message describing a problem
/// (missing argument, validation failure, user requested help, etc) and the program would
/// exit.
///
/// Values inside can be of any type for as long as they implement `Debug`, `Clone` and
/// there are no lifetimes other than static.
///
/// When consuming the values you can jump straight to a value that implements
/// [`FromStr`] trait and then transform it into something that your program would use. Alternatively,
/// you can consume either `String` or `OsString` and parse that by hand. It's better to perform
/// as much parsing and validation inside the `Parser` as possible so the program itself gets
/// strictly typed and correct value while the user gets immediate feedback on what's wrong with the
/// arguments they pass.
///
/// Order of operations matters, each subsequent parser gets the output of the earlier one. Both
/// parsers `a` and `b` would consume multiple numeric values, each less than 10, but `a`
/// validates a single value and then consumes multiple of them already validated, while `b` first
/// consumes and then performs validation. The former approach is usually more readable.
/// ```rust
/// # use bpaf::*;
/// # fn simple() {
/// let a = short('a').argument::<usize>("N")
///     .guard(|&a| a < 10, "`a` must be below 10")
///     .many();
/// let b = short('b').argument::<usize>("N")
///     .many()
///     .guard(|bs| bs.iter().all(|&b| b < 10), "`b` must be below 10");
/// # }
/// ```
///
/// The same logic applies to derive API - the current type depends on the order of annotations:
/// ```rust
/// # use bpaf::*;
/// # fn less_than_10(a: &usize) -> bool { true }
/// # fn all_less_than_10(a: &Vec<usize>) -> bool { true }
/// #[derive(Bpaf, Debug, Clone)]
/// struct Simple {
///     #[bpaf(argument("N"), guard(less_than_10, "`a` must be below 10"), many)]
///     a: Vec<usize>,
///     #[bpaf(argument("N"), many, guard(all_less_than_10, "`b` must be below 10"))]
///     b: Vec<usize>,
/// }
/// ```
///
/// For example suppose your program needs the user to specify dimensions of a rectangle, with sides
/// being 1..20 units long and the total area must not exceed 200 units square. A parser that
/// consumes it might look like this:
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Copy, Clone)]
/// struct Rectangle {
///     width: u32,
///     height: u32,
/// }
///
/// fn rectangle() -> impl Parser<Rectangle> {
///     let invalid_size = "Sides of a rectangle must be 1..20 units long";
///     let invalid_area = "Area of a rectangle must not exceed 200 units square";
///     let width = long("width")
///         .help("Width of the rectangle")
///         .argument::<u32>("PX")
///         .guard(|&x| 1 <= x && x <= 10, invalid_size);
///     let height = long("height")
///         .help("Height of the rectangle")
///         .argument::<u32>("PX")
///         .guard(|&x| 1 <= x && x <= 10, invalid_size);
///     construct!(Rectangle { width, height })
///         .guard(|&r| r.width * r.height <= 400, invalid_area)
/// }
/// ```
///
///
/// # Derive specific considerations
///
/// Every method defined on this trait belongs to the `postprocessing` section of the field
/// annotation. `bpaf` would try to figure out what chain to use for as long as there are no
/// options changing the type: you can use [`fallback`](Parser::fallback_with),
/// [`fallback_with`](Parser::fallback_with), [`guard`](Parser::guard), [`hide`](Parser::hide`) and
/// [`group_help`](Parser::group_help) but not the rest of them.
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     // no annotation at all - `bpaf` inserts implicit `argument` and gets the right type
///     number_1: u32,
///
///     // fallback isn't changing the type so `bpaf` still handles it
///     #[bpaf(fallback(42))]
///     number_2: u32,
///
///     // `bpaf` inserts implicit `argument`, `optional` and the right type
///     number_3: Option<u32>,
///
///     // fails to compile: you need to specify `argument`
///     // #[bpaf(optional)]
///     // number_4: Option<u32>,
///
///     #[bpaf(argument("N"), optional)]
///     number_5: Option<u32>,
///
///     // explicit consumer and a full postprocessing chain
///     #[bpaf(argument::<u32>("N"), optional)]
///     number_6: Option<u32>,
/// }
/// ```
pub trait Parser<T> {
    /// Evaluate inner function
    ///
    /// Mostly internal implementation details, you can try using it to test your parsers
    // it's possible to move this function from the trait to the structs but having it
    // in the trait ensures the composition always works
    #[doc(hidden)]
    fn eval(&self, args: &mut State) -> Result<T, Error>;

    /// Included information about the parser
    ///
    /// Mostly internal implementation details, you can try using it to test your parsers
    // it's possible to move this function from the trait to the structs but having it
    // in the trait ensures the composition always works
    #[doc(hidden)]
    fn meta(&self) -> Meta;

    // change shape
    // {{{ many
    /// Consume zero or more items from a command line and collect them into a [`Vec`]
    ///
    /// `many` preserves any parsing failures and propagates them outwards, with an extra
    /// [`catch`](ParseMany::catch) statement you can instead stop at the first value
    /// that failed to parse and ignore it and all the subsequent ones.
    ///
    /// `many` will collect at most one result that does not consume anything from the argument
    /// list allowing using it in combination with any parsers with a fallback. After the first
    /// one, it will keep collecting the results as long as they consume something.
    ///
    /// For derive usage `bpaf` would insert implicit `many` when the resulting type is a
    /// vector.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/many.md"))]
    ///
    /// # See also
    /// [`some`](Parser::some) also collects results to a vector but requires at least one
    /// element to succeed, [`collect`](Parser::collect) collects results into a [`FromIterator`]
    /// structure
    fn many(self) -> ParseMany<Self>
    where
        Self: Sized,
    {
        ParseMany {
            inner: self,
            catch: false,
        }
    }
    // }}}

    // {{{ collect
    /// Transform parser into a collection parser
    ///
    /// A generic variant of [`many`](Parser::many), instead of collecting into a vector
    /// it collects into any collection that implements [`FromIterator`] trait
    ///
    /// `collect` preserves any parsing failures and propagates them outwards, with extra
    /// [`catch`](ParseCollect::catch) statement you can instead stop at the first value
    /// that failed to parse and ignore it and all the subsequent ones.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/collect.md"))]
    ///
    /// `collect` will collect at most one result that does not consume anything from the argument
    /// list allowing using it in combination of any parsers with a fallback. After the first one
    /// it will keep collecting the results as long as they consume something.
    fn collect<C>(self) -> ParseCollect<Self, C, T>
    where
        C: FromIterator<T>,
        Self: Sized,
    {
        ParseCollect {
            inner: self,
            catch: false,
            ctx: PhantomData,
        }
    }
    // }}}

    // {{{ some
    /// Consume one or more items from a command line and collect them into a [`Vec`]
    ///
    /// Takes a string used as an error message if there are no specified parameters
    ///
    /// `some` preserves any parsing failures and propagates them outwards, with an extra
    /// [`catch`](ParseSome::catch) statement you can instead stop at the first value
    /// that failed to parse and ignore it and all the subsequent ones.
    ///
    /// `some` will collect at most one result that does not consume anything from the argument
    /// list allowing using it in combination with any parsers with a fallback. After the first
    /// one, it will keep collecting the results as long as they consume something.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/some.md"))]
    ///
    /// # See also
    /// [`many`](Parser::many) also collects results to a vector but succeeds with
    /// no matching values. [`collect`](Parser::collect) collects results into a [`FromIterator`]
    /// structure

    #[must_use]
    fn some(self, message: &'static str) -> ParseSome<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseSome {
            inner: self,
            message,
            catch: false,
        }
    }
    // }}}

    // {{{ optional
    /// Turn a required argument into an optional one
    ///
    /// `optional` converts any missing items into `None` and passes the remaining parsing
    /// failures untouched. With an extra [`catch`](ParseOptional::catch) statement, you can handle
    /// those failures too.
    ///
    /// # Derive usage
    ///
    /// By default, `bpaf` would automatically use optional for fields of type `Option<T>`,
    /// for as long as it's not prevented from doing so by present postprocessing options.
    /// But it's also possible to specify it explicitly.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/optional.md"))]
    ///
    #[must_use]
    fn optional(self) -> ParseOptional<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseOptional {
            inner: self,
            catch: false,
        }
    }
    // }}}

    #[must_use]
    /// Count how many times the inner parser succeeds, and return that number.
    ///
    /// When you are dealing with a parser that can succeed without consuming
    /// anything from a command line - `bpaf` will count first such success as well.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/count.md"))]
    fn count(self) -> ParseCount<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        ParseCount {
            inner: self,
            ctx: PhantomData,
        }
    }

    #[must_use]
    /// Apply the inner parser as many times as it succeeds, return the last value
    ///
    /// You can use this to allow users to pick contradicting options
    #[cfg_attr(not(doctest), doc = include_str!("docs2/last.md"))]
    fn last(self) -> ParseLast<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseLast { inner: self }
    }

    // parse
    // {{{ parse
    /// Apply a failing transformation to a contained value
    ///
    /// Transformation preserves the present/absent state of the value: to parse an optional value you
    /// can either first try to `parse` it and then mark it as [`optional`](Parser::optional) or first
    /// deal with the optionality and then parse a value wrapped in [`Option`]. In most cases
    /// the former approach is more concise.
    ///
    /// Similarly, it is possible to parse multiple items with [`many`](Parser::many) or
    /// [`some`](Parser::some) by either parsing a single item first and then turning it into a [`Vec`]
    /// or collecting them into a [`Vec`] first and then parsing the whole vector. The former approach
    /// is more concise.
    ///
    /// This is a most general of transforming parsers and you can express
    /// [`map`](Parser::map) and [`guard`](Parser::guard) in terms of it.
    ///
    /// Examples are a bit artificial, to parse a value from a string you can specify
    /// the type directly in the `argument`'s turbofish and then apply `map`.
    ///
    /// # Derive usage:
    /// `parse` takes a single parameter: function name to call. Function type should match
    /// parameter `F` used by `parse` in combinatoric API.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/parse.md"))]
    ///
    fn parse<F, R, E>(self, f: F) -> ParseWith<T, Self, F, E, R>
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
    // }}}

    // {{{ map
    /// Apply a pure transformation to a contained value
    ///
    /// A common case of the [`parse`](Parser::parse) method, exists mostly for convenience.
    ///
    /// # Derive usage:
    /// The `map` takes a single parameter: function name to call. This function should transform
    /// the value produced by the parser into a new value of the same or different type.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/map.md"))]
    ///
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
    // }}}

    // {{{ guard
    /// Validate or fail with a message
    ///
    /// If the value doesn't satisfy the constraint - the parser fails with the specified error message.
    ///
    /// # Derive usage
    /// Derive variant of the `guard` takes a function name instead of a closure, mostly to keep things
    /// clean. The second argument can be either a string literal or a constant name for a static [`str`].
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/guard.md"))]
    ///
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
    // }}}

    // combine
    // {{{ fallback
    /// Use this value as default if the value isn't present on a command line
    ///
    /// Parser would still fail if the value is present but failure comes from some transformation
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/dis_fallback.md"))]
    ///
    /// # See also
    /// [`fallback_with`](Parser::fallback_with) would allow to try to fallback to a value that
    /// comes from a failing computation such as reading a file. By default, the fallback value will
    /// not be shown in the `--help` output; you can change that by using
    /// [`display_fallback`](ParseFallback::display_fallback),
    /// [`debug_fallback`](ParseFallback::debug_fallback), or
    /// [`format_fallback`](ParseFallback::format_fallback).
    #[must_use]
    fn fallback(self, value: T) -> ParseFallback<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        ParseFallback {
            inner: self,
            value,
            value_str: String::new(),
        }
    }
    // }}}

    // {{{ fallback_with
    /// Use value produced by this function as default if the value isn't present
    ///
    /// Would still fail if the value is present but failure comes from some earlier transformation
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/dis_fallback_with.md"))]
    ///
    /// # See also
    /// [`fallback`](Parser::fallback) implements similar logic expect that failures aren't expected.
    /// By default, the fallback value will
    /// not be shown in the `--help` output; you can change that by using
    /// [`display_fallback`](ParseFallbackWith::display_fallback),
    /// [`debug_fallback`](ParseFallbackWith::debug_fallback), or
    /// [`format_fallback`](ParseFallbackWith::format_fallback).
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
            value_str: String::new(),
            err: PhantomData,
        }
    }
    // }}}

    // {{{ or_else
    /// If first parser fails - try the second one
    ///
    /// For parser to succeed eiter of the components needs to succeed. If both succeed - `bpaf`
    /// would use output from one that consumed the left most value. The second flag on the command
    /// line remains unconsumed by `or_else`.
    ///
    /// # Combinatoric usage:
    /// There's two ways to write this combinator with identical results:
    /// ```rust
    /// # use bpaf::*;
    /// fn a() -> impl Parser<u32> {
    ///     short('a').argument::<u32>("NUM")
    /// }
    ///
    /// fn b() -> impl Parser<u32> {
    ///     short('b').argument::<u32>("NUM")
    /// }
    ///
    /// fn a_or_b_comb() -> impl Parser<u32> {
    ///     construct!([a(), b()])
    /// }
    ///
    /// fn a_or_b_comb2() -> impl Parser<u32> {
    ///     a().or_else(b())
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -a 12 -b 3
    /// // 12
    /// $ app -b 3 -a 12
    /// // 3
    /// $ app -b 13
    /// // 13
    /// $ app
    /// // fails asking for either -a NUM or -b NUM
    /// ```
    ///
    /// # Derive usage:
    ///
    /// `bpaf` translates enum into alternative combinations, different shapes of variants
    /// produce different results.
    ///
    ///
    /// ```bpaf
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// enum Flag {
    ///     A { a: u32 }
    ///     B { b: u32 }
    /// }
    /// ```
    ///
    /// ```console
    /// $ app -a 12 -b 3
    /// // Flag::A { a: 12 }
    /// $ app -b 3 -a 12
    /// // Flag::B { b: 3 }
    /// $ app -b 3
    /// // Flag::B { b: 3 }
    /// $ app
    /// // fails asking for either -a NUM or -b NUM
    /// ```
    ///
    /// # Performance
    ///
    /// `bpaf` tries to evaluate both branches regardless of the successes to produce a
    /// better error message for combinations of mutually exclusive parsers:
    /// Suppose program accepts one of two mutually exclusive switches `-a` and `-b`
    /// and both are present error message should point at the second flag
    #[doc(hidden)]
    #[deprecated(
        since = "0.5.0",
        note = "instead of a.or_else(b) you should use construct!([a, b])"
    )]
    fn or_else<P>(self, alt: P) -> ParseOrElse<T>
    where
        Self: Sized + Parser<T> + 'static,
        P: Sized + Parser<T> + 'static,
    {
        ParseOrElse {
            this: Box::new(self),
            that: Box::new(alt),
        }
    }
    // }}}

    // misc
    // {{{ hide
    /// Ignore this parser during any sort of help generation
    ///
    /// Best used for optional parsers or parsers with a defined fallback, usually for implementing
    /// backward compatibility or hidden aliases
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/hide.md"))]
    ///
    fn hide(self) -> ParseHide<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHide { inner: self }
    }
    // }}}

    /// Ignore this parser when generating a usage line
    ///
    /// Parsers hidden from usage will still show up in the available arguments list. Best used on
    /// optional things that augment the main application functionality but not define it.
    /// Alternatively, you can use [`custom_usage`](Parser::custom_usage) to replace a single
    /// option or a group of them with some other text.
    #[cfg_attr(not(doctest), doc = include_str!("docs2/hide_usage.md"))]
    #[must_use]
    fn hide_usage(self) -> ParseUsage<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseUsage {
            inner: self,
            usage: Doc::default(),
        }
    }

    /// Customize how this parser looks like in the usage line
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/custom_usage.md"))]
    #[must_use]
    fn custom_usage<M>(self, usage: M) -> ParseUsage<Self>
    where
        M: Into<Doc>,
        Self: Sized + Parser<T>,
    {
        ParseUsage {
            inner: self,
            usage: usage.into(),
        }
    }

    // {{{ group_help
    /// Attach a help message to a complex parser
    ///
    /// `bpaf` inserts the group help message before the block with all the fields
    /// from the inner parser and an empty line after the block.
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/group_help.md"))]
    fn group_help<M: Into<Doc>>(self, message: M) -> ParseGroupHelp<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseGroupHelp {
            inner: self,
            message: message.into(),
        }
    }
    // }}}

    /// Make a help message for a complex parser from its [`MetaInfo`]
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/with_group_help.md"))]
    fn with_group_help<F>(self, f: F) -> ParseWithGroupHelp<Self, F>
    where
        Self: Sized + Parser<T>,
        F: Fn(MetaInfo) -> Doc,
    {
        ParseWithGroupHelp { inner: self, f }
    }

    // {{{ comp
    /// Dynamic shell completion
    ///
    /// Allows to generate autocompletion information for the shell. Completer places generated input
    /// in place of metavar placeholders, so running `completer` on something that doesn't have a
    /// [`positional`] or an [`argument`](NamedArg::argument) doesn't make much sense.
    ///
    /// Takes a function as a parameter that tries to complete partial input to a full one with an
    /// optional description. `bpaf` would substitute a current positional item or an argument with an empty
    /// string if a value isn't available yet so it's best to run `complete` where parsing can't fail:
    /// right after [`argument`](NamedArg::argument) or [`positional`], but this isn't enforced.
    ///
    /// # Example
    /// ```console
    /// $ app --name L<TAB>
    /// $ app --name Lupusregina _
    /// ```
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/complete.md"))]
    ///
    /// ## A simple example
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/simple_dynamic.md"))]
    ///
    /// ## More detailed example
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/derive_show_asm.md"))]
    ///
    #[cfg(feature = "autocomplete")]
    fn complete<M, F>(self, op: F) -> ParseComp<Self, F>
    where
        M: Into<String>,
        F: Fn(&T) -> Vec<(M, Option<M>)>,
        Self: Sized + Parser<T>,
    {
        ParseComp {
            inner: self,
            op,
            group: None,
        }
    }
    // }}}

    // {{{
    /// Static shell completion
    ///
    /// Allows to ask existing shell completion to provide some information such as a file or
    /// directory names or pass through existing shell completion scripts, see
    /// [`ShellComp`](complete_shell::ShellComp) for accessible functionality
    ///
    /// Places function calls in place of metavar placeholder, so running `complete_shell` on
    /// something that doesn't have a [`positional`] or [`argument`](NamedArg::argument) doesn't
    /// make much sense.
    ///
    /// # Example
    /// ```console
    /// $ app --output C<TAB>
    /// $ app --output Cargo.toml _
    /// ```
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn output() -> impl Parser<String> {
    ///     long("output")
    ///         .help("Cargo.toml file to use as output")
    ///         .argument("OUTPUT")
    ///         .complete_shell(ShellComp::File { mask: Some("*.toml") })
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     /// Cargo.toml file to use as output
    ///     #[bpaf(argument("OUTPUT"), complete_shell(ShellComp::File { mask: Some("*.toml") }))]
    ///     output: String,
    /// }
    /// ```
    ///
    /// For multiple file types correct mask syntax is `"*.(toml|md)"`.
    #[cfg(feature = "autocomplete")]
    fn complete_shell(
        self,
        op: complete_shell::ShellComp,
    ) -> crate::complete_shell::ParseCompShell<Self>
    where
        Self: Sized + Parser<T>,
    {
        crate::complete_shell::ParseCompShell { inner: self, op }
    }
    // }}}

    // consume
    // {{{ to_options
    /// Transform `Parser` into [`OptionParser`] to get ready to [`run`](OptionParser::run) it
    ///
    ///
    /// # Derive usage
    /// Add a top-level `options` annotation to generate [`OptionParser`] instead of default
    /// [`Parser`].
    ///
    /// In addition to `options` annotation, you can also specify either `version` or
    /// `version(value)` annotation. The former uses version from `cargo`, later uses the
    /// specified value which should be an expression of type `&'static str`, see
    /// [`version`](OptionParser::version).
    ///
    #[cfg_attr(not(doctest), doc = include_str!("docs2/to_options.md"))]
    ///
    /// # See also
    /// There's some methods implemented on [`OptionParser`] directly to customize the appearance
    fn to_options(self) -> OptionParser<T>
    where
        Self: Sized + Parser<T> + 'static,
    {
        OptionParser {
            info: info::Info::default(),
            inner: Box::new(self),
        }
    }
    // }}}

    /// Finalize and run the parser
    ///
    /// Generally, you'd want to use [`Parser::to_options`] to finalize the parser and [`OptionParser::run`],
    /// but this also works for simple cases:
    ///
    /// ```no_run
    /// # use bpaf::*;
    /// fn main() {
    ///     let name = short('n').long("name").argument::<String>("USER").run();
    ///     // do things with name
    /// }
    /// ```
    fn run(self) -> T
    where
        Self: Sized + Parser<T> + 'static,
    {
        self.to_options().run()
    }

    /// Create a boxed representation for a parser
    ///
    ///

    /// The boxed parser doesn't expose internal representation in its type and allows to return
    /// of different parsers in different conditional branches
    ///
    /// You can create it with a single argument `construct` macro or by using `boxed` annotation
    #[cfg_attr(not(doctest), doc = include_str!("docs2/boxed.md"))]
    fn boxed(self) -> Box<dyn Parser<T>>
    where
        Self: Sized + Parser<T> + 'static,
    {
        Box::new(self)
    }
}

/// Parser that produces a fixed value
///
/// This parser produces `T` without consuming anything from the command line, which can be useful
/// with [`construct!`]. As with any parsers, `T` should be `Clone` and `Debug`.
///
/// Both `pure` and [`pure_with`] are designed to put values into structures, to generate fallback
/// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
///
/// See also [`pure_with`] for a pure computation that can fail.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/pure.md"))]
#[must_use]
pub fn pure<T>(val: T) -> ParsePure<T> {
    ParsePure(val)
}

/// Wrap a calculated value into a `Parser`
///
/// This parser represents a possibly failing equivalent to [`pure`].
/// It produces `T` by invoking the provided callback without consuming anything from the command
/// line, which can be useful with [`construct!`]. As with any parsers, `T` should be `Clone`
/// and `Debug`.
///
/// Both [`pure`] and `pure_with` are designed to put values into structures, to generate fallback
/// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
///
/// See also [`pure`] for a pure computation that can't fail.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/pure_with.md"))]
pub fn pure_with<T, F, E>(val: F) -> ParsePureWith<T, F, E>
where
    F: Fn() -> Result<T, E>,
    E: ToString,
{
    ParsePureWith(val)
}

/// Fail with a fixed error message
///
/// This parser produces `T` of any type but instead of producing it when asked - it fails
/// with a custom error message. Can be useful for creating custom logic
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn must_agree() -> impl Parser<()> {
///     let a = long("accept").req_flag(());
///     let no_a = fail("You must accept the license agreement with --agree before proceeding");
///     construct!([a, no_a])
/// }
/// ```
///
/// # Example
/// ```console
/// $ app
/// // exits with "You must accept the license agreement with --agree before proceeding"
/// $ app --agree
/// // succeeds
/// ```
#[must_use]
pub fn fail<T>(msg: &'static str) -> ParseFail<T> {
    ParseFail {
        field1: msg,
        field2: PhantomData,
    }
}

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a short name
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env()) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn short(short: char) -> NamedArg {
    NamedArg {
        short: vec![short],
        env: Vec::new(),
        long: Vec::new(),
        help: None,
    }
}

/// Parse a [`flag`](NamedArg::flag)/[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument) that has a long name
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env()) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn long(long: &'static str) -> NamedArg {
    NamedArg {
        short: Vec::new(),
        long: vec![long],
        env: Vec::new(),
        help: None,
    }
}

/// Parse an environment variable
///
/// You can chain multiple [`short`](NamedArg::short), [`long`](NamedArg::long) and
/// [`env`](NamedArg::env()) for multiple names. You can specify multiple names of the same type,
///  `bpaf` would use items past the first one as hidden aliases.
///
/// For [`flag`](NamedArg::flag) and [`switch`](NamedArg::switch) environment variable being present
/// gives the same result as the flag being present, allowing to implement things like `NO_COLOR`
/// variables:
///
/// ```console
/// $ NO_COLOR=1 app --do-something
/// ```
///
/// If you don't specify a short or a long name - whole argument is going to be absent from the
/// help message. Use it combined with a named or positional argument to have a hidden fallback
/// that wouldn't leak sensitive info.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/short_long_env.md"))]
#[must_use]
pub fn env(variable: &'static str) -> NamedArg {
    NamedArg {
        short: Vec::new(),
        long: Vec::new(),
        help: None,
        env: vec![variable],
    }
}

/// Parse a positional argument
///
/// For named flags and arguments ordering generally doesn't matter: most programs would
/// understand `-O2 -v` the same way as `-v -O2`, but for positional items order matters: in *nix
/// `cat hello world` and `cat world hello` would display contents of the same two files but in
/// a different order.
///
/// When using combinatoric API you can specify the type with turbofish, for parsing types
/// that don't implement [`FromStr`] you can use consume a `String`/`OsString` first and parse
/// it by hand.
/// ```no_run
/// # use bpaf::*;
/// fn parse_pos() -> impl Parser<usize> {
///     positional::<usize>("POS")
/// }
/// ```
///
/// # Important restriction
/// To parse positional arguments from a command line you should place parsers for all your
/// named values before parsers for positional items and commands. In derive API fields parsed as
/// positional items or commands should be at the end of your `struct`/`enum`. The same rule applies
/// to parsers with positional fields or commands inside: such parsers should go to the end as well.
///
/// Use [`check_invariants`](OptionParser::check_invariants) in your test to ensure correctness.
///
/// For example for non-positional `non_pos` and positional `pos` parsers
/// ```rust
/// # use bpaf::*;
/// # let non_pos = || short('n').switch();
/// # let pos = ||positional::<String>("POS");
/// let valid = construct!(non_pos(), pos());
/// let invalid = construct!(pos(), non_pos());
/// ```
///
/// **`bpaf` panics during help generation unless this restriction holds**
///
/// Without using `--` `bpaf` would only accept items that don't start with `-` as positional, you
/// can use [`any`] to work around this restriction.
///
/// By default `bpaf` accepts positional items with or without `--` where values permit, you can
/// further restrict the parser to accept positional items only on the right side of `--` using
/// [`strict`](ParsePositional::strict).
#[cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))]
#[must_use]
pub fn positional<T>(metavar: &'static str) -> ParsePositional<T> {
    build_positional(metavar)
}

#[doc(hidden)]
#[deprecated = "You should switch from command(name, sub) to sub.command(name)"]
pub fn command<T>(name: &'static str, subparser: OptionParser<T>) -> ParseCommand<T>
where
    T: 'static,
{
    ParseCommand {
        longs: vec![name],
        shorts: Vec::new(),
        help: subparser.short_descr().map(Into::into),
        subparser,
        adjacent: false,
    }
}

/// Parse a single arbitrary item from a command line
///
/// **`any` is designed to consume items that don't fit into the usual [`flag`](NamedArg::flag)
/// /[`switch`](NamedArg::switch)/[`argument`](NamedArg::argument)/[`positional`]/
/// [`command`](OptionParser::command) classification, in most cases you don't need to use it**
///
/// By default, `any` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct and it will only try to parse the first unconsumed item
/// on the command line. It is possible to lift this restriction by calling
/// [`anywhere`](ParseAny::anywhere) on the parser.
///
/// `check` argument is a function from any type `I` that implements `FromStr` to `T`.
/// Usually this should be `String` or `OsString`, but feel free to experiment. When
/// running `any` tries to parse an item on a command line into that `I` and applies the `check`
/// function. If the `check` succeeds - parser `any` succeeds and produces `T`, otherwise it behaves
/// as if it hasn't seen it. If `any` works in `anywhere` mode - it will try to parse all other
/// unconsumed items, otherwise, `any` fails.
///
/// # Use `any` to capture the remaining arguments
/// Normally you would use [`positional`] with [`strict`](ParsePositional::strict) annotation for
/// that, but using any allows you to blur the boundary between arguments for child process and self
/// process a bit more.
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_simple.md"))]
///
/// # Use `any` to parse a non standard flag
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_switch.md"))]
///
/// # Use `any` to parse a non standard argument
/// Normally `any` would try to display itself as a usual metavariable in the usage line and
/// generated help, you can customize that with [`metavar`](ParseAny::metavar) method:
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_literal.md"))]
///
/// # See also
/// [`literal`] - a specialized version of `any` that tries to parse a fixed literal
#[must_use]
pub fn any<I, T, F>(metavar: &str, check: F) -> ParseAny<T>
where
    I: FromStr + 'static,
    F: Fn(I) -> Option<T> + 'static,
    <I as std::str::FromStr>::Err: std::fmt::Display,
{
    ParseAny {
        metavar: [(metavar, Style::Metavar)][..].into(),
        help: None,
        check: Box::new(move |os: std::ffi::OsString| {
            match crate::from_os_str::parse_os_str::<I>(os) {
                Ok(v) => check(v),
                Err(_) => None,
            }
        }),

        anywhere: false,
    }
}

/// A specialized version of [`any`] that consumes an arbitrary string
///
/// By default `literal` behaves similarly to [`positional`] so you should be using it near the
/// rightmost end of the consumer struct and it will only try to parse the first unconsumed
/// item on the command line. It is possible to lift this restriction by calling
/// [`anywhere`](ParseAny::anywhere) on the parser.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/any_literal.md"))]
///
/// # See also
/// [`any`] - a generic version of `literal` that uses function to decide if value is to be parsed
/// or not.
#[must_use]
pub fn literal(val: &'static str) -> ParseAny<()> {
    any("", move |s: String| if s == val { Some(()) } else { None })
        .metavar(&[(val, crate::buffer::Style::Literal)][..])
}

/// Strip a command name if present at the front when used as a `cargo` command
///
// this is exactly the same as batteries::cargo_helper, but used by derive macro...
#[must_use]
#[doc(hidden)]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    T: 'static,
    P: Parser<T>,
{
    let skip = literal(cmd).optional().hide();
    construct!(skip, parser).map(|x| x.1)
}

/// Choose between several parsers specified at runtime
///
/// You can use this function to create multiple parsers that produce the same type of value at a runtime
/// and let bpaf to pick one that best fits best. This function is designed to work in Combinatoric
/// API, but you can use it in Derive API with `extern`.
///
#[cfg_attr(not(doctest), doc = include_str!("docs2/choice.md"))]
pub fn choice<T: 'static>(parsers: impl IntoIterator<Item = Box<dyn Parser<T>>>) -> impl Parser<T> {
    let mut parsers = parsers.into_iter();
    let mut this = match parsers.next() {
        None => return fail("Invalid choice usage").boxed(),
        Some(p) => p,
    };
    for that in parsers {
        this = Box::new(ParseOrElse { this, that })
    }
    this
}
