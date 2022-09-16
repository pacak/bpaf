#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![allow(clippy::needless_doctest_main)]

//! Lightweight and flexible command line argument parser with derive and combinatoric style API

//! # Derive and combinatoric API
//!
//! `bpaf` supports both combinatoric and derive APIs and it's possible to mix and match both APIs
//! at once. Both APIs provide access to mostly the same features, some things are more convenient
//! to do with derive (usually less typing), some - with combinatoric (usually maximum flexibility
//! and reducing boilerplate structs). In most cases using just one would suffice. Whenever
//! possible APIs share the same keywords and overall structure. Documentation for combinatoric API
//! also explains how to perform the same action in derive style.
//!
//! `bpaf` supports dynamic shell completion for `bash`, `zsh`, `fish` and `elvish`.

//! # Quick links
//!
//! - [Derive tutorial](crate::_derive_tutorial)
//! - [Combinatoric tutorial](crate::_combinatoric_tutorial)
// - [Picking the right words](crate::_flow)
//! - [Batteries included](crate::batteries)
//! - [Q&A](https://github.com/pacak/bpaf/discussions/categories/q-a)

//! # Quick start, derive edition
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//! ```toml
//! [dependencies]
//! bpaf = { version = "0.5", features = ["derive"] }
//! ```
//!
//! 2. Define a structure containing command line attributes and run generated function
//! ```no_run
//! use bpaf::Bpaf;
//!
//! #[derive(Clone, Debug, Bpaf)]
//! #[bpaf(options, version)]
//! /// Accept speed and distance, print them
//! struct SpeedAndDistance {
//!     /// Speed in KPH
//!     speed: f64,
//!     /// Distance in miles
//!     distance: f64,
//! }
//!
//! fn main() {
//!     // #[derive(Bpaf) generates function speed_and_distance
//!     let opts = speed_and_distance().run();
//!     println!("Options: {:?}", opts);
//! }
//! ```
//!
//! 3. Try to run the app
//! ```console
//! % very_basic --help
//! Accept speed and distance, print them
//!
//! Usage: --speed ARG --distance ARG
//!
//! Available options:
//!         --speed <ARG>     Speed in KPH
//!         --distance <ARG>  Distance in miles
//!     -h, --help            Prints help information
//!     -V, --version         Prints version information
//!
//! % very_basic --speed 100
//! Expected --distance ARG, pass --help for usage information
//!
//! % very_basic --speed 100 --distance 500
//! Options: SpeedAndDistance { speed: 100.0, distance: 500.0 }
//!
//! % very_basic --version
//! Version: 0.5.0 (taken from Cargo.toml by default)
//!```

//! # Quick start, combinatoric edition
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//! ```toml
//! [dependencies]
//! bpaf = "0.5"
//! ```
//!
//! 2. Declare parsers for components, combine them and run it
//! ```no_run
//! use bpaf::{construct, long, Parser};
//! #[derive(Clone, Debug)]
//! struct SpeedAndDistance {
//!     /// Dpeed in KPH
//!     speed: f64,
//!     /// Distance in miles
//!     distance: f64,
//! }
//!
//! fn main() {
//!     // primitive parsers
//!     let speed = long("speed")
//!         .help("Speed in KPG")
//!         .argument("SPEED")
//!         .from_str::<f64>();
//!
//!     let distance = long("distance")
//!         .help("Distance in miles")
//!         .argument("DIST")
//!         .from_str::<f64>();
//!
//!     // parser containing information about both speed and distance
//!     let parser = construct!(SpeedAndDistance { speed, distance });
//!
//!     // option parser with metainformation attached
//!     let speed_and_distance
//!         = parser
//!         .to_options()
//!         .descr("Accept speed and distance, print them");
//!
//!     let opts = speed_and_distance.run();
//!     println!("Options: {:?}", opts);
//! }
//! ```
//!
//! 3. Try to run it, output should be similar to derive edition

//! # Design goals: flexibility, reusability, correctness

//! Library allows to consume command line arguments by building up parsers for individual
//! arguments and combining those primitive parsers using mostly regular Rust code plus one macro.
//! For example it's possible to take a parser that requires a single floating point number and
//! transform it to a parser that takes several of them or takes it optionally so different
//! subcommands or binaries can share a lot of the code:

//! ```rust
//! # use bpaf::*;
//! // a regular function that doesn't depend on anything, you can export it
//! // and share across subcommands and binaries
//! fn speed() -> impl Parser<f64> {
//!     long("speed")
//!         .help("Speed in KPH")
//!         .argument("SPEED")
//!         .from_str::<f64>()
//! }
//!
//! // this parser accepts multiple `--speed` flags from a command line when used,
//! // collecting them into a vector
//! fn multiple_args() -> impl Parser<Vec<f64>> {
//!     speed().many()
//! }
//!
//! // this parser checks if `--speed` is present and uses value of 42 if it's not
//! fn with_fallback() -> impl Parser<f64> {
//!     speed().fallback(42.0)
//! }
//! ```
//!
//! At any point you can apply additional validation or fallback values in terms of current parsed
//! state of each subparser and you can have several stages as well:
//!
//! ```rust
//! # use bpaf::*;
//! #[derive(Clone, Debug)]
//! struct Speed(f64);
//! fn speed() -> impl Parser<Speed> {
//!     long("speed")
//!         .help("Speed in KPH")
//!         .argument("SPEED")
//!         // After this point the type is `impl Parser<String>`
//!         .from_str::<f64>()
//!         // `from_str` uses FromStr trait to transform contained value into `f64`
//!
//!         // You can perform additional validation with `parse` and `guard` functions
//!         // in as many steps as required.
//!         // Before and after next two applications the type is still `impl Parser<f64>`
//!         .guard(|&speed| speed >= 0.0, "You need to buy a DLC to move backwards")
//!         .guard(|&speed| speed <= 100.0, "You need to buy a DLC to break the speed limits")
//!
//!         // You can transform contained values, next line gives `impl Parser<Speed>` as a result
//!         .map(|speed| Speed(speed))
//! }
//! ```
//!
//! Library follows parse, don’t validate approach to validation when possible. Usually you parse
//! your values just once and get the results as a rust struct/enum with strict types rather than a
//! stringly typed hashmap with stringly typed values in both combinatoric and derive APIs.

//! # Design goals: restrictions
//!
//! The main restricting library sets is that you can't use parsed values (but not the fact that
//! parser succeeded or failed) to decide how to parse subsequent values. In other words parsers
//! don't have the monadic strength, only the applicative one.
//!
//! To give an example, you can implement this description:
//!
//! > Program takes one of `--stdout` or `--file` flag to specify the output target, when it's `--file`
//! > program also requires `-f` attribute with the filename
//!
//! But not this one:
//!
//! > Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it's `'file'`
//! > program also requires `-f` attribute with the filename
//!
//! This set of restrictions allows to extract information about the structure of the computations
//! to generate help and overall results in less confusing enduser experience
//!
//! `bpaf` performs no parameter names validation, in fact having multiple parameters
//! with the same name is fine and you can combine them as alternatives and performs no fallback
//! other than [`fallback`](Parser::fallback). You need to pay attention to the order of the
//! alternatives inside the macro: parser that consumes the left most available argument on a
//! command line wins, if this is the same - left most parser wins. So to parse a parameter
//! `--test` that can be both [`switch`](NamedArg::switch) and [`argument`](NamedArg::argument) you
//! should put the argument one first.
//!
//! You must place [`positional`] items at the end of a structure in derive API or consume them
//! as last arguments in derive API.

//! # Dynamic shell completion
//!
//! `bpaf` implements shell completion to allow to automatically fill in not only flag and command
//! names, but also argument and positional item values.
//!
//! 1. Enable `autocomplete` feature:
//!    ```toml
//!    bpaf = { version = "0.5.5", features = ["autocomplete"] }
//!    ```
//! 2. Decorate [`argument`](NamedArg::argument) and [`positional`] parsers with
//!    [`complete`](Parser::complete) to autocomplete argument values
//!
//! 3. Depending on your shell generate appropriate completion file and place it to whereever your
//!    shell is going to look for it, name of the file should correspond in some way to name of
//!    your program. Consult manual for your shell for the location and named conventions:
//!    1. **bash**: for the first `bpaf` completion you need to install the whole script
//!        ```console
//!        $ your_program --bpaf-complete-style-bash >> ~/.bash_completion
//!        ```
//!        but since the script doesn't depend on a program name - it's enough to do this for
//!        each next program
//!        ```console
//!        echo "complete -F _bpaf_dynamic_completion your_program" >> ~/.bash_completion
//!        ```
//!    2. **zsh**: note `_` at the beginning of the filename
//!       ```console
//!       $ your_program --bpaf-complete-style-zsh > ~/.zsh/_your_program
//!       ```
//!    3. **fish**
//!       ```console
//!       $ your_program --bpaf-complete-style-fish > ~/.config/fish/completions/your_program.fish
//!       ```
//!    4. **elvish** - not sure where to put it, documentation is a bit cryptic
//!       ```console
//!       $ your_program --bpaf-complete-style-elvish
//!       ```
//! 4. Restart your shell - you need to done it only once or optionally after bpaf major version
//!    upgrade: generated completion files contain only instructions how to ask your program for
//!    possible completions and don't change even if options are different.
//!
//! 5. Generated scripts rely on your program being accessible in $PATH

//! # Design non goals: performance
//!
//! Library aims to optimize for flexibility, reusability and compilation time over runtime
//! performance which means it might perform some additional clones, allocations and other less
//! optimal things. In practice unless you are parsing tens of thousands of different parameters
//! and your app exits within microseconds - this won't affect you. That said - any actual
//! performance related problems with real world applications is a bug.

//! # More examples
//!
//! You can find a bunch more examples here: <https://github.com/pacak/bpaf/tree/master/examples>
//!
//!
//! They're usually documented or at least contain an explanation to important bits and you can see
//! how they work by cloning the repo and running
//! ```shell
//! $ cargo run --example example_name
//! ```

//! # Testing your own parsers
//!
//! You can test your own parsers to maintain compatibility or simply checking expected output
//! with [`run_inner`](OptionParser::run_inner)
//!
//! ```rust
//! # use bpaf::*;
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)]
//! pub struct Options {
//!     pub user: String
//! }
//!
//! #[test]
//! fn test_my_options() {
//!     let help = options()
//!         .run_inner(Args::from(&["--help"]))
//!         .unwrap_err()
//!         .unwrap_stdout();
//!     let expected_help = "\
//! Usage --user <ARG>
//! <skip>
//! ";
//!
//!     assert_eq!(help, expected_help);
//! }
//! ```
//!

//! # Cargo features
//!
//! - `derive`: adds a dependency on [`bpaf_derive`] crate and reexport `Bpaf` derive macro. You
//!   need to enable it to use derive API. Disabled by default.
//!
//! - `extradocs`: used internally to include tutorials to <https://docs.rs/bpaf>, no reason to
//! enable it for local development unless you want to build your own copy of the documentation
//! (<https://github.com/rust-lang/cargo/issues/8905>). Disabled by default.
//!
//! - `batteries`: helpers implemented with public `bpaf` API. Enabled by default.
//!
//! - `autocomplete`: enables support for shell autocompletion. Disabled by default.

#[cfg(feature = "extradocs")]
pub mod _combinatoric_tutorial;
#[cfg(feature = "extradocs")]
pub mod _derive_tutorial;
#[cfg(feature = "extradocs")]
mod _flow;
mod arg;
mod args;
#[cfg(feature = "autocomplete")]
mod complete_gen;
#[cfg(feature = "autocomplete")]
mod complete_run;
mod info;
mod item;
mod meta;
mod meta_help;
mod meta_usage;
mod meta_youmean;
mod params;
mod structs;
#[cfg(test)]
mod tests;

pub mod batteries;

#[doc(hidden)]
pub use crate::info::Error;
use crate::item::Item;
use std::marker::PhantomData;
#[doc(hidden)]
pub use structs::{ParseBox, ParseCon};

pub mod parsers {
    //! This module exposes parsers that can be configured further with builder pattern
    //!
    //! In most cases you won't be using those names directly, they are only listed here to provide
    //! access to documentation for member functions
    pub use crate::params::{NamedArg, ParseArgument, ParseCommand, ParsePositional};
    pub use crate::structs::{ParseMany, ParseOptional, ParseSome};
}

use structs::{
    ParseAdjacent, ParseFail, ParseFallback, ParseFallbackWith, ParseFromStr, ParseGroupHelp,
    ParseGuard, ParseHide, ParseMany, ParseMap, ParseOptional, ParseOrElse, ParsePure, ParseSome,
    ParseWith,
};

#[cfg(feature = "autocomplete")]
use structs::{ParseComp, ParseCompStyle};

#[doc(inline)]
pub use crate::args::Args;
pub use crate::info::OptionParser;
pub use crate::meta::Meta;

#[doc(inline)]
pub use crate::params::{any, command, env, long, positional, short};

#[cfg(doc)]
pub(self) use crate::parsers::NamedArg;

#[doc(inline)]
#[cfg(feature = "bpaf_derive")]
pub use bpaf_derive::Bpaf;
mod from_os_str;

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
/// // parallel composition, tries all parsers, picks succeeding left most one:
/// construct!([a, b, c]);
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
/// ```rust
/// # use bpaf::*;
/// struct Res (u32, u32);
/// enum Ul { T { a: u32, b: u32 } }
///
/// // You can share parameters across multiple construct invocations
/// // if defined as functions
/// fn a() -> impl Parser<u32> {
///     short('a').argument("N").from_str::<u32>()
/// }
///
/// // You can construct structs or enums with unnamed fields
/// fn res() -> impl Parser<Res> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(Res ( a(), b ))
/// }
///
/// // You can construct structs or enums with named fields
/// fn ult() -> impl Parser<Ul> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(Ul::T { a(), b })
/// }
///
/// // You can also construct simple tuples
/// fn tuple() -> impl Parser<(u32, u32)> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(a(), b)
/// }
///
/// // You can create boxed version of parsers so the type will be the same for as long
/// // as return type is the same - can be useful for all sort of dynamic parsers
/// fn boxed() -> impl Parser<u32> {
///     let a = short('a').argument('n').from_str::<u32>();
///     construct!(a)
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
///     short('b').argument("NUM").from_str::<u32>()
/// }
///
/// fn a_or_b() -> impl Parser<u32> {
///     let a = short('a').argument("NUM").from_str::<u32>();
///     // equivalent way of writing this would be `a.or_else(b())`
///     construct!([a, b()])
/// }
/// ```
///
/// # Derive usage
///
/// `bpaf_derive` would combine fields of struct or enum constructors sequentially and enum
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
/// // here user needs to pass either both -a AND -b or both -c and -d
/// #[derive(Debug, Clone, Bpaf)]
/// enum Ult {
///     AB { a: u32, b: u32 },
///     CD { c: u32, d: u32 }
/// }
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
    ($first:ident $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first $($tokens)*) }};

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

    (@prepare [alt] [$first:ident $($fields:ident)*]) => {
        #[allow(deprecated)]
        { use $crate::Parser; $first $(.or_else($fields))* }
    };

    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::__cons_prepare!($ty [$($fields)*])
    };

    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ { $($fields),* } };
    (@make [pos   [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ ( $($fields),* ) };
    (@make [pos] [$($fields:ident)*]) => { ( $($fields),* ) };
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "autocomplete"))]
/// to avoid extra parsing when autocomplete feature is off
macro_rules! __cons_prepare {
    ([pos] [$field:ident]) => { $crate::ParseBox { inner: Box::new($field) } };

    ($ty:tt [$($fields:ident)+]) => {{
        use $crate::Parser;
        let meta = $crate::Meta::And(vec![ $($fields.meta()),+ ]);
        let inner = move |args: &mut $crate::Args| {
            $(let $fields = $fields.eval(args)?;)+
            args.current = None;
            ::std::result::Result::Ok::<_, $crate::Error>
                ($crate::construct!(@make $ty [$($fields)+]))
        };
        $crate::ParseCon { inner, meta }
    }};
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "autocomplete")]
/// for completion bpaf needs to observe all the failures in a branch
macro_rules! __cons_prepare {
    ([pos] [$field:ident]) => { $crate::ParseBox { inner: Box::new($field) } };

    ($ty:tt [$($fields:ident)+]) => {{
        use $crate::Parser;
        let meta = $crate::Meta::And(vec![ $($fields.meta()),+ ]);
        let inner = move |args: &mut $crate::Args| {
            $(let $fields = if args.is_comp() {
                $fields.eval(args)
            } else {
                Ok($fields.eval(args)?)
            };)+
            $(let $fields = $fields?;)+

            args.current = None;
            ::std::result::Result::Ok::<_, $crate::Error>
                ($crate::construct!(@make $ty [$($fields)+]))
        };
        $crate::ParseCon { inner, meta }
    }};
}

/// Simple or composed argument parser
///
/// # Overview
///
/// It's best to think of an object implementing [`Parser`] trait as a container with a value
/// inside that are composable with other `Parser` containers using [`construct!`] and the only
/// way to extract this value is by transforming it to [`OptionParser`] with
/// [`to_options`](Parser::to_options) and running it with [`run`](OptionParser::run). At which
/// point you either get your value out or `bpaf` would generate a message describing a problem
/// (missing argument, validation failure, user requested help, etc) and the program would
/// exit.
///
/// Values inside can be of any type for as long as they implement `Debug`, `Clone` and
/// there's no lifetimes other than static.
///
/// When consuming the values you usually start with `Parser<String>` or `Parser<OsString>` which
/// you then transform into something that your program would actually use. it's better to perform
/// as much parsing and validation inside the `Parser` as possible so the program itself gets
/// strictly typed and correct value while user gets immediate feedback on what's wrong with the
/// arguments they pass.
///
/// For example suppose your program needs user to specify a dimensions of a rectangle, with sides
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
///         .argument("PX")
///         .from_str::<u32>()
///         .guard(|&x| 1 <= x && x <= 10, invalid_size);
///     let height = long("height")
///         .help("Height of the rectangle")
///         .argument("PX")
///         .from_str::<u32>()
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
/// annotation. `bpaf_derive` would try to figure out what chain to use for as long as there's no
/// options changing the type: you can use [`fallback`](Parser::fallback_with),
/// [`fallback_with`](Parser::fallback_with), [`guard`](Parser::guard), [`hide`](Parser::hide`) and
/// [`group_help`](Parser::group_help) but not the rest of them.
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     // no annotation at all - `bpaf_derive` inserts implicit `argument` and `from_str`
///     number_1: u32,
///
///     // fallback isn't changing the type so `bpaf_derive` still handles it
///     #[bpaf(fallback(42))]
///     number_2: u32,
///
///     // `bpaf_derive` inserts implicit `argument`, `optional` and `from_str`
///     number_3: Option<u32>,
///
///     // fails to compile: you need to specify a consumer, `argument` or `argument_os`
///     // #[bpaf(optional)]
///     // number_4: Option<u32>
///
///     // fails to compile: you also need to specify how to go from String to u32
///     // #[bpaf(argument("N"), optional)]
///     // number_5: Option<u32>,
///
///     // explicit consumer and a full postprocessing chain
///     #[bpaf(argument("N"), from_str(u32), optional)]
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
    fn eval(&self, args: &mut Args) -> Result<T, Error>;

    /// Included information about the parser
    ///
    /// Mostly internal implementation details, you can try using it to test your parsers
    // it's possible to move this function from the trait to the structs but having it
    // in the trait ensures the composition always works
    #[doc(hidden)]
    fn meta(&self) -> Meta;

    // change shape
    // {{{ many
    /// Consume zero or more items from a command line and collect them into [`Vec`]
    ///
    /// `many` only collects elements that only consume something from the argument list.
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// fn numbers() -> impl Parser<Vec<u32>> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .many()
    /// }
    /// ```
    ///
    /// # Derive usage:
    /// `bpaf` would insert implicit `many` when resulting type is a vector
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"))]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    /// But it's also possible to specify it explicitly, both cases renerate the same code.
    /// Note, since using `many` resets the postprocessing chain - you also need to specify
    /// [`from_str`](Parser::from_str)
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), many)]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    ///
    ///
    /// # Example
    /// ```console
    /// $ app -n 1 -n 2 -n 3
    /// // [1, 2, 3]
    /// ```
    ///
    /// # See also
    /// [`some`](Parser::some) also collects results to a vector but requires at least one
    /// element to succeed
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

    // {{{ some
    /// Consume one or more items from a command line
    ///
    /// Takes a string used as an error message if there's no specified parameters
    ///
    /// `some` only collects elements that only consume something from the argument list.
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// let numbers
    ///     = short('n')
    ///     .argument("NUM")
    ///     .from_str::<u32>()
    ///     .some("Need at least one number");
    /// # drop(numbers);
    /// ```
    ///
    /// # Derive usage
    /// Since using `some` resets the postprocessing chain - you also need to specify
    /// [`from_str`](Parser::from_str) or similar, depending on your type
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), some("Need at least one number"))]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    ///
    ///
    /// # Example
    /// ```console
    /// $ app
    /// // fails with "Need at least one number"
    /// $ app -n 1 -n 2 -n 3
    /// // [1, 2, 3]
    /// ```
    ///
    /// # See also
    /// [`many`](Parser::many) also collects results to a vector but succeeds with
    /// no matching values
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
    /// Turn a required argument into optional one
    ///
    /// `optional` converts any failure caused by missing items into is `None` and passes
    /// the remaining parsing failures untouched.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<Option<u32>> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .optional()
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// By default `bpaf_derive` would automatically use optional for fields of type `Option<T>`,
    /// for as long as it's not prevented from doing so by present postprocessing options
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"))]
    ///    number: Option<u32>
    /// }
    /// ```
    ///
    /// But it's also possible to specify it explicitly, in which case you need to specify
    /// a full postprocessing chain which starts from [`from_str`](Parser::from_str) in this
    /// example.
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"), from_str(u32), optional)]
    ///    number: Option<u32>
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app
    /// // None
    /// $ app -n 42
    /// // Some(42)
    /// ```
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

    // parse
    // {{{ parse
    /// Apply a failing transformation to a contained value
    ///
    /// This is a most general of transforming parsers and you can express remaining ones
    /// terms of it: [`map`](Parser::map), [`from_str`](Parser::from_str) and
    /// [`guard`](Parser::guard).
    ///
    /// Examples given here are a bit artificail, to parse a value from string you should use
    /// [`from_str`](Parser::from_str).
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// # use std::str::FromStr;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .parse(|s| u32::from_str(&s))
    /// }
    /// ```
    /// # Derive usage:
    /// `parse` takes a single parameter: function name to call. Function type should match
    /// parameter `F` used by `parse` in combinatoric API.
    /// ```rust
    /// # use bpaf::*;
    /// # use std::str::FromStr;
    /// # use std::num::ParseIntError;
    /// fn read_number(s: String) -> Result<u32, ParseIntError> {
    ///     u32::from_str(&s)
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), parse(read_number))]
    ///     number: u32
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 12
    /// // 12
    /// # app -n pi
    /// // fails with "Couldn't parse "pi": invalid numeric literal"
    /// ```
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
    /// A common case of [`parse`](Parser::parse) method, exists mostly for convenience.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .map(|v| v * 2)
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// fn double(num: u32) -> u32 {
    ///     num * 2
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), map(double))]
    ///     number: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 21
    /// // 42
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
    // }}}

    // {{{ from_str
    /// Parse stored [`String`] using [`FromStr`](std::str::FromStr) instance
    ///
    /// A common case of [`parse`](Parser::parse) method, exists mostly for convenience.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn speed() -> impl Parser<f64> {
    ///     short('s')
    ///         .argument("SPEED")
    ///         .from_str::<f64>()
    /// }
    /// ```
    ///
    /// # Derive usage
    /// By default `bpaf_derive` would use [`from_str`](Parser::from_str) for any time it's not
    /// familiar with so you don't need to specify anything
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("SPEED"))]
    ///     speed: f64
    /// }
    /// ```
    ///
    /// But it's also possible to specify it explicitly
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("SPEED"), from_str(f64))]
    ///     speed: f64
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -s pi
    /// // fails with "Couldn't parse "pi": invalid float literal"
    /// $ app -s 3.1415
    /// // Version: 3.1415
    /// ```
    ///
    /// # See also
    /// Other parsing and restricting methods include [`parse`](Parser::parse) and
    /// [`guard`](Parser). For transformations that can't fail you can use [`map`](Parser::map).
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
    // }}}

    // {{{ guard
    /// Validate or fail with a message
    ///
    /// If value doesn't satisfy the constraint - parser fails with the specified error message.
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .guard(|n| *n <= 10, "Values greater than 10 are only available in the DLC pack!")
    /// }
    /// ```
    ///
    /// # Derive usage
    /// Unlike combinator counterpart, derive variant of `guard` takes a function name instead
    /// of a closure, mostly to keep thing clean. Second argument can be either a string literal
    /// or a constant name for a static [`str`].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn dlc_check(number: &u32) -> bool {
    ///     *number <= 10
    /// }
    ///
    /// const DLC_NEEDED: &str = "Values greater than 10 are only available in the DLC pack!";
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), guard(dlc_check, DLC_NEEDED))]
    ///     number: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 100
    /// // fails with "Values greater than 10 are only available in the DLC pack!"
    /// $ app -n 5
    /// // 5
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
    // }}}

    // combine
    // {{{ fallback
    /// Use this value as default if value isn't present on a command line
    ///
    /// Parser would still fail if value is present but failure comes from some transformation
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .fallback(42)
    /// }
    /// ```
    ///
    /// # Derive usage
    /// Expression in parens should have the right type, this example uses `u32` literal,
    /// but it can also be your own type if that's what you are parsing, it can also be a function
    /// call.
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"), from_str(u32), fallback(42))]
    ///    number: u32
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 100
    /// // 10
    /// $ app
    /// // 42
    /// $ app -n pi
    /// // fails with "Couldn't parse "pi": invalid numeric literal"
    /// ```
    ///
    /// # See also
    /// [`fallback_with`](Parser::fallback_with) would allow to try to fallback to a value that
    /// comes from a failing computation such as reading a file.
    /// TODO: document top level fallback for derive
    #[must_use]
    fn fallback(self, value: T) -> ParseFallback<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        ParseFallback { inner: self, value }
    }
    // }}}

    // {{{ fallback_with
    /// Use value produced by this function as default if value isn't present
    ///
    /// Would still fail if value is present but failure comes from some earlier transformation
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn username() -> impl Parser<String> {
    ///     long("user")
    ///         .argument("USER")
    ///         .fallback_with::<_, Box<dyn std::error::Error>>(||{
    ///             let output = std::process::Command::new("whoami")
    ///                 .stdout(std::process::Stdio::piped())
    ///                 .spawn()?
    ///                 .wait_with_output()?
    ///                 .stdout;
    ///             Ok(std::str::from_utf8(&output)?.to_owned())
    ///         })
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// fn get_current_user() -> Result<String, Box<dyn std::error::Error>> {
    ///     let output = std::process::Command::new("whoami")
    ///         .stdout(std::process::Stdio::piped())
    ///         .spawn()?
    ///         .wait_with_output()?
    ///         .stdout;
    ///     Ok(std::str::from_utf8(&output)?.to_owned())
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(long, argument("USER"), fallback_with(get_current_user))]
    ///     user: String,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --user bobert
    /// // "bobert"
    /// $ app
    /// // "pacak"
    /// ```
    ///
    /// # See also
    /// [`fallback`](Parser::fallback) implements similar logic expect that failures
    /// aren't expected.
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
    ///     short('a').argument("NUM").from_str::<u32>()
    /// }
    ///
    /// fn b() -> impl Parser<u32> {
    ///     short('b').argument("NUM").from_str::<u32>()
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
    /// `bpaf_derive` translates enum into alternative combinations, different shapes of variants
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
    // }}}

    // misc
    // {{{ hide
    /// Ignore this parser during any sort of help generation
    ///
    /// Best used for optional parsers or parsers with a defined fallback, usually for implementing
    /// backward compatibility or hidden aliases
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// /// bpaf would accept both `-W` and `-H` flags, but the help message
    /// /// would contain only `-H`
    /// fn rectangle() -> impl Parser<(u32, u32)> {
    ///     let width = short('W')
    ///         .argument("PX")
    ///         .from_str::<u32>()
    ///         .fallback(10)
    ///         .hide();
    ///     let height = short('H')
    ///         .argument("PX")
    ///         .from_str::<u32>()
    ///         .fallback(10)
    ///         .hide();
    ///     construct!(width, height)
    /// }
    /// ```
    /// # Example
    /// ```console
    /// $ app -W 12 -H 15
    /// // (12, 15)
    /// $ app -H 333
    /// // (10, 333)
    /// $ app --help
    /// // contains -H but not -W
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Rectangle {
    ///     #[bpaf(short('W'), argument("PX"), from_str(u32), fallback(10), hide)]
    ///     width: u32,
    ///     #[bpaf(short('H'), argument("PX"), from_str(u32))]
    ///     height: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -W 12 -H 15
    /// // Rectangle { width: 12, height: 15 }
    /// $ app -H 333
    /// // Rectangle { width: 10, height: 333 }
    /// $ app --help
    /// // contains -H but not -W
    /// ```
    fn hide(self) -> ParseHide<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHide { inner: self }
    }
    // }}}

    // {{{ group_help
    /// Attach help message to a complex parser
    ///
    /// `bpaf` inserts the group help message before the block with all the fields
    /// from the inner parser and an empty line after the block.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn rectangle() -> impl Parser<(u32, u32)> {
    ///     let width = short('w')
    ///         .argument("PX")
    ///         .from_str::<u32>();
    ///     let height = short('h')
    ///         .argument("PX")
    ///         .from_str::<u32>();
    ///     construct!(width, height)
    ///         .group_help("Takes a rectangle")
    /// }
    /// ```
    /// # Example
    /// ```console
    /// $ app --help
    /// <skip>
    ///             Takes a rectangle
    ///    -w <PX>  Width of the rectangle
    ///    -h <PX>  Height of the rectangle
    ///
    /// <skip>
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Rectangle {
    ///     width: u32,
    ///     height: u32,
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(external, group_help("Takes a rectangle"))]
    ///     rectangle: Rectangle
    /// }
    /// ```
    fn group_help(self, message: &'static str) -> ParseGroupHelp<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseGroupHelp {
            inner: self,
            message,
        }
    }
    // }}}

    // {{{ comp
    /// Dynamic shell completion
    ///
    /// Allows to generate autocompletion information for shell. Completer places generated input
    /// in place of metavar placeholders, so running `completer` on something that doesn't have a
    /// [`positional`] or an [`argument`](NamedArg::argument) (or their `_os` variants) doesn't make
    /// much sense.
    ///
    /// Takes a function as a parameter that tries to complete partial input to a full one with
    /// optional description. `bpaf` would substitute current positional item or an argument an empty
    /// string if a value isn't available yet so it's best to run `complete` where parsing can't fail:
    /// right after [`argument`](NamedArg::argument) or [`positional`], but this isn't enforced.
    ///
    /// `bpaf` doesn't support generating [`OsString`](std::ffi::OsString) completions: `bpaf` must
    /// print completions to console and for non-string values it's not possible (accurately).
    ///
    /// **Using this function requires enabling `"autocomplete"` feature, not enabled by default**.
    ///
    /// # Example
    /// ```console
    /// $ app --name L<TAB>
    /// $ app --name Lupusregina _
    /// ```
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    ///     let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    ///     names
    ///         .iter()
    ///         .filter(|name| name.starts_with(input))
    ///         .map(|name| (*name, None))
    ///         .collect::<Vec<_>>()
    /// }
    ///
    /// fn name() -> impl Parser<String> {
    ///     short('n')
    ///         .long("name")
    ///         .help("Specify character's name")
    ///         .argument("Name")
    ///         .complete(completer)
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    ///     let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    ///     names
    ///         .iter()
    ///         .filter(|name| name.starts_with(input))
    ///         .map(|name| (*name, None))
    ///         .collect::<Vec<_>>()
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(argument("NAME"), complete(completer))]
    ///     name: String,
    /// }
    /// ```
    #[cfg(feature = "autocomplete")]
    fn complete<M, F>(self, op: F) -> ParseComp<Self, F>
    where
        M: Into<String>,
        F: Fn(&T) -> Vec<(M, Option<M>)>,
        Self: Sized + Parser<T>,
    {
        ParseComp { inner: self, op }
    }
    // }}}

    // {{{ complete_style
    /// Add extra annotations to completion information
    ///
    /// Not all information is gets supported by all the shells
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn opts() -> impl Parser<(bool, bool)> {
    ///     let a = short('a').switch();
    ///     let b = short('b').switch();
    ///     let c = short('c').switch();
    ///     let d = short('d').switch();
    ///     let ab = construct!(a, b).complete_style(CompleteDecor::VisibleGroup("a and b"));
    ///     let cd = construct!(c, d).complete_style(CompleteDecor::VisibleGroup("c and d"));
    ///     construct!([ab, cd])
    /// }
    #[cfg(feature = "autocomplete")]
    fn complete_style(self, style: CompleteDecor) -> ParseCompStyle<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseCompStyle { inner: self, style }
    }
    // }}}

    // {{{ adjacent
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
    /// `bpaf` can run adjacently restricted parsers multiple times to refine the guesses. It is
    /// best not to have complex inter-fields verification since they might trip up the detection
    /// logic: instead of destricting let's say sum of two fields to be 5 or greater *inside* the
    /// `adjacent` parser, you can restrict it *outside*, once `adjacent` done the parsing.
    ///
    /// `adjacent` is defined on a trait for better discoverability, it doesn't make much sense to
    /// use it on something other than [`command`](OptionParser::command) or [`construct!`] encasing
    /// several fields.
    ///
    /// There's also similar method [`adjacent`](ParseArgument) that allows to restrict argument
    /// parser to work only for arguments where both key and a value are in the same shell word:
    /// `-f=bar` or `-fbar`, but not `-f bar`.
    #[must_use]
    fn adjacent(self) -> ParseAdjacent<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseAdjacent { inner: self }
    }
    // }}}

    // consume
    // {{{ to_options
    /// Transform `Parser` into [`OptionParser`] to attach metadata and run
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parser() -> impl Parser<u32> {
    ///     short('i')
    ///         .argument("ARG")
    ///         .from_str::<u32>()
    /// }
    ///
    /// fn option_parser() -> OptionParser<u32> {
    ///     parser()
    ///         .to_options()
    ///         .version("3.1415")
    ///         .descr("This is a description")
    /// }
    /// ```
    ///
    /// See [`OptionParser`] for more methods available after conversion.
    ///
    /// # Derive usage
    /// Add a top level `options` annotation to generate [`OptionParser`] instead of default
    /// [`Parser`].
    ///
    /// In addition to `options` annotation you can also specify either `version` or
    /// `version(value)` annotation. Former uses version from `cargo`, later uses the
    /// specified value which should be an expression of type `&'static str`, see
    /// [`version`](OptionParser::version).
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version("3.1415"))]
    /// /// This is a description
    /// struct Options {
    ///    verbose: bool,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --version
    /// // Version: 3.1415
    /// $ app --help
    /// <skip>
    /// This is a description
    /// <skip>
    /// ```
    fn to_options(self) -> OptionParser<T>
    where
        Self: Sized + Parser<T> + 'static,
    {
        OptionParser {
            info: info::Info::default(),
            inner_type: PhantomData,
            inner: Box::new(self),
        }
    }
    // }}}
}

#[non_exhaustive]
/// Various complete options decorations
///
/// Somewhat work in progress, only makes a difference in zsh
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn pair() -> impl Parser<(bool, bool)> {
///     let a = short('a').switch();
///     let b = short('b').switch();
///     construct!(a, b)
///         .complete_style(CompleteDecor::VisibleGroup("a and b"))
/// }
/// ```
///
/// # Derive usage
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(complete_style(CompleteDecor::VisibleGroup("a and b")))]
/// struct Options {
///     a: bool,
///     b: bool,
/// }
/// ```
///
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "autocomplete")]
pub enum CompleteDecor {
    /// Group items according to this group
    HiddenGroup(&'static str),
    /// Group items according to this group but also show the group name
    VisibleGroup(&'static str),
}

/// Wrap a value into a `Parser`
///
/// This parser produces `T` without consuming anything from the command line, can be useful
/// with [`construct!`]. As with any parsers `T` should be `Clone` and `Debug`.
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn pair() -> impl Parser<(bool, u32)> {
///     let a = long("flag-a").switch();
///     let b = pure(42u32);
///     construct!(a, b)
/// }
/// ```
#[must_use]
pub fn pure<T>(val: T) -> ParsePure<T> {
    ParsePure(val)
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

/// Unsuccessful command line parsing outcome, use it for unit tests
///
/// Useful for unit testing for user parsers, consume it with
/// [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`]
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Print this to stdout and exit with success code
    Stdout(String),
    /// Print this to stderr and exit with failure code
    Stderr(String),
}

impl ParseFailure {
    /// Returns the contained `stderr` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stdout`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Self::Stderr(err) => err,
            Self::Stdout(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    /// Returns the contained `stdout` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stderr`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Self::Stdout(err) => err,
            Self::Stderr(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }
}

/// Strip a command name if present at the front when used as a `cargo` command
///
/// When implementing a cargo subcommand parser needs to be able to skip the first argument which
/// is always the same as the executable name without `cargo-` prefix. For example if executable name is
/// `cargo-cmd` so first argument would be `cmd`. `cargo_helper` helps to support both invocations:
/// with name present when used via cargo and without it when used locally.
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn options() -> OptionParser<(u32, u32)> {
///     let width = short('w').argument("PX").from_str::<u32>();
///     let height = short('h').argument("PX").from_str::<u32>();
///     let parser = construct!(width, height);
///     cargo_helper("cmd", parser).to_options()
/// }
/// ```
///
/// # Derive usage
///
/// If you pass a cargo command name as a parameter to `options` annotation `bpaf_derive` would generate `cargo_helper`.
/// ```no_run
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options("cmd"))]
/// struct Options {
///     #[bpaf(short, argument("PX"))]
///     width: u32,
///     #[bpaf(short, argument("PX"))]
///     height: u32,
/// }
///
/// fn main() {
///    println!("{:?}", options().run());
/// }
///
/// ```
///
/// # Example
///
/// ```console
/// $ cargo cmd -w 3 -h 5
/// (3, 5)
/// $ cargo run --bin cargo-cmd -- -w 3 -h 5
/// (3, 5)
/// ```
#[must_use]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    T: 'static,
    P: Parser<T>,
{
    let eat_command =
        positional("").parse(move |s| if cmd == s { Ok(()) } else { Err(String::new()) });
    let ignore_non_command = pure(());
    let skip = construct!([eat_command, ignore_non_command]).hide();
    construct!(skip, parser).map(|x| x.1)
}
