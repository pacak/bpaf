#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::redundant_else)] // not useful

//! Lightweight and flexible command line argument parser with derive and combinatoric style API

//! # Derive and combinatoric API
//!
//! `bpaf` supports both combinatoric and derive APIs and it's possible to mix and match both APIs
//! at once. Both APIs provide access to mostly the same features, some things are more convenient
//! to do with derive (usually less typing), some - with combinatoric (usually maximum flexibility
//! and reducing boilerplate structs). In most cases using just one would suffice. Whenever
//! possible APIs share the same keywords and overall structure. Documentation is shared and
//! contains examples for both combinatoric and derive style.
//!
//! `bpaf` supports dynamic shell completion for `bash`, `zsh`, `fish` and `elvish`.

//! # Quick links
//!
//! - [Derive tutorial](crate::_derive_tutorial)
//! - [Combinatoric tutorial](crate::_combinatoric_tutorial)
//! - [Some very unusual cases](crate::_unusual)
//! - [Applicative functors? What is it all about](crate::_applicative)
//! - [Batteries included](crate::batteries)
//! - [Q&A](https://github.com/pacak/bpaf/discussions/categories/q-a)

//! # Quick start - combinatoric and derive APIs
//!
//! <details>
//! <summary style="display: list-item;">Derive style API, click to expand</summary>
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//!    ```toml
//!    [dependencies]
//!    bpaf = { version = "0.7", features = ["derive"] }
//!    ```
//!
//! 2. Define a structure containing command line attributes and run generated function
//!    ```no_run
//!    use bpaf::Bpaf;
//!
//!    #[derive(Clone, Debug, Bpaf)]
//!    #[bpaf(options, version)]
//!    /// Accept speed and distance, print them
//!    struct SpeedAndDistance {
//!        /// Speed in KPH
//!        speed: f64,
//!        /// Distance in miles
//!        distance: f64,
//!    }
//!
//!    fn main() {
//!        // #[derive(Bpaf)] generates `speed_and_distance` function
//!        let opts = speed_and_distance().run();
//!        println!("Options: {:?}", opts);
//!    }
//!    ```
//!
//! 3. Try to run the app
//!    ```console
//!    % very_basic --help
//!    Accept speed and distance, print them
//!
//!    Usage: --speed ARG --distance ARG
//!
//!    Available options:
//!            --speed <ARG>     Speed in KPH
//!            --distance <ARG>  Distance in miles
//!        -h, --help            Prints help information
//!        -V, --version         Prints version information
//!
//!    % very_basic --speed 100
//!    Expected --distance ARG, pass --help for usage information
//!
//!    % very_basic --speed 100 --distance 500
//!    Options: SpeedAndDistance { speed: 100.0, distance: 500.0 }
//!
//!    % very_basic --version
//!    Version: 0.5.0 (taken from Cargo.toml by default)
//!    ```
//! 4. You can check the [derive tutorial](crate::_derive_tutorial) for more detailed information.
//!
//! </details>
//!
//! <details>
//! <summary style="display: list-item;">Combinatoric style API, click to expand</summary>
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//!    ```toml
//!    [dependencies]
//!    bpaf = "0.7"
//!    ```
//!
//! 2. Declare parsers for components, combine them and run it
//!    ```no_run
//!    use bpaf::{construct, long, Parser};
//!    #[derive(Clone, Debug)]
//!    struct SpeedAndDistance {
//!        /// Dpeed in KPH
//!        speed: f64,
//!        /// Distance in miles
//!        distance: f64,
//!    }
//!
//!    fn main() {
//!        // primitive parsers
//!        let speed = long("speed")
//!            .help("Speed in KPG")
//!            .argument::<f64>("SPEED");
//!
//!        let distance = long("distance")
//!            .help("Distance in miles")
//!            .argument::<f64>("DIST");
//!
//!        // parser containing information about both speed and distance
//!        let parser = construct!(SpeedAndDistance { speed, distance });
//!
//!        // option parser with metainformation attached
//!        let speed_and_distance
//!            = parser
//!            .to_options()
//!            .descr("Accept speed and distance, print them");
//!
//!        let opts = speed_and_distance.run();
//!        println!("Options: {:?}", opts);
//!    }
//!    ```
//!
//! 3. Try to run the app
//!
//!    ```console
//!    % very_basic --help
//!    Accept speed and distance, print them
//!
//!    Usage: --speed ARG --distance ARG
//!
//!    Available options:
//!            --speed <ARG>     Speed in KPH
//!            --distance <ARG>  Distance in miles
//!        -h, --help            Prints help information
//!        -V, --version         Prints version information
//!
//!    % very_basic --speed 100
//!    Expected --distance ARG, pass --help for usage information
//!
//!    % very_basic --speed 100 --distance 500
//!    Options: SpeedAndDistance { speed: 100.0, distance: 500.0 }
//!
//!    % very_basic --version
//!    Version: 0.5.0 (taken from Cargo.toml by default)
//!    ```
//!
//! 4. You can check the [combinatoric tutorial](crate::_combinatoric_tutorial) for more detailed information.
//!
//!
//! </details>
//!
//! # Design goals: flexibility, reusability, correctness
//!
//! Library allows to consume command line arguments by building up parsers for individual
//! arguments and combining those primitive parsers using mostly regular Rust code plus one macro.
//! For example it's possible to take a parser that requires a single floating point number and
//! transform it to a parser that takes several of them or takes it optionally so different
//! subcommands or binaries can share a lot of the code:
//!
//! ```rust
//! # use bpaf::*;
//! // a regular function that doesn't depend on any context, you can export it
//! // and share across subcommands and binaries
//! fn speed() -> impl Parser<f64> {
//!     long("speed")
//!         .help("Speed in KPH")
//!         .argument::<f64>("SPEED")
//! }
//!
//! // this parser accepts multiple `--speed` flags from a command line when used,
//! // collecting results into a vector
//! fn multiple_args() -> impl Parser<Vec<f64>> {
//!     speed().many()
//! }
//!
//! // this parser checks if `--speed` is present and uses value of 42.0 if it's not
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
//!         .argument::<f64>("SPEED")
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
//! Library follows **parse, don’t validate** approach to validation when possible. Usually you parse
//! your values just once and get the results as a Rust struct/enum with strict types rather than a
//! stringly typed hashmap with stringly typed values in both combinatoric and derive APIs.

//! # Design goals: restrictions
//!
//! The main restricting library sets is that you can't use parsed values (but not the fact that
//! parser succeeded or failed) to decide how to parse subsequent values. In other words parsers
//! don't have the monadic strength, only the applicative one - for more detailed explanation see
//! [Applicative functors? What is it all about](crate::_applicative).
//!
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
//! This set of restrictions allows `bpaf` to extract information about the structure of the computations
//! to generate help, dynamic completion and overall results in less confusing enduser experience
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
//!    bpaf = { version = "0.7", features = ["autocomplete"] }
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
//! You can find a more examples here: <https://github.com/pacak/bpaf/tree/master/examples>
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
//! - `derive`: adds a dependency on `bpaf_derive` crate and reexport `Bpaf` derive macro. You
//!   need to enable it to use derive API. Disabled by default.
//!
//! - `extradocs`: used internally to include tutorials to <https://docs.rs/bpaf>, no reason to
//! enable it for local development unless you want to build your own copy of the documentation
//! (<https://github.com/rust-lang/cargo/issues/8905>). Disabled by default.
//!
//! - `batteries`: helpers implemented with public `bpaf` API. Disabled by default.
//!
//! - `autocomplete`: enables support for shell autocompletion. Disabled by default.
//!
//! - `bright-color`, `dull-color`: use more colors when printing `--help` and such. Enabling
//!   either color feature adds some extra dependencies and might raise MRSV. If you are planning
//!   to use this feature in a published app - it's best to expose them as feature flags:
//!   ```toml
//!   [features]
//!   bright-color = ["bpaf/bright-color"]
//!   dull-color = ["bpaf/dull-color"]
//!   ```
//!   Disabled by default.
//!
//! - `manpage`: generate man page from help declaration, see [`OptionParser::as_manpage`]. Disabled by default.
//!
//!

#[macro_use]
#[cfg(feature = "color")]
mod color;

#[macro_use]
#[cfg(not(feature = "color"))]
mod no_color;

#[cfg(feature = "color")]
#[doc(hidden)]
pub use color::set_override;

#[cfg(not(feature = "color"))]
#[doc(hidden)]
pub use no_color::set_override;

#[cfg(feature = "extradocs")]
pub mod _applicative;
#[cfg(feature = "extradocs")]
pub mod _combinatoric_tutorial;
#[cfg(feature = "extradocs")]
pub mod _derive_tutorial;
#[cfg(feature = "extradocs")]
pub mod _unusual;
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
mod help;
mod info;
mod item;
#[cfg(feature = "manpage")]
mod manpage;
mod meta;
mod meta_help;
mod meta_usage;
mod meta_youmean;
pub mod params;
mod structs;
#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use crate::info::Error;
use crate::item::Item;
use std::marker::PhantomData;
#[doc(hidden)]
pub use structs::{ParseBox, ParseCon};

#[cfg(feature = "autocomplete")]
pub use crate::complete_shell::ShellComp;

#[cfg(feature = "manpage")]
pub use manpage::Section;

pub mod parsers {
    //! This module exposes parsers that accept further configuration with builder pattern
    //!
    //! In most cases you won't be using those names directly, they're only listed here to provide
    //! access to documentation
    #[cfg(feature = "autocomplete")]
    pub use crate::complete_shell::ParseCompShell;
    pub use crate::params::{NamedArg, ParseArgument, ParseCommand, ParsePositional};
    pub use crate::structs::{ParseBox, ParseMany, ParseOptional, ParseSome};
}

use structs::{
    ParseAdjacent, ParseAnywhere, ParseFail, ParseFallback, ParseFallbackWith, ParseGroupHelp,
    ParseGuard, ParseHide, ParseHideUsage, ParseMany, ParseMap, ParseOptional, ParseOrElse,
    ParsePure, ParsePureWith, ParseSome, ParseWith,
};

#[cfg(feature = "autocomplete")]
use structs::{ParseComp, ParseCompStyle};

#[doc(inline)]
pub use crate::args::Args;
pub use crate::from_os_str::FromUtf8;
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
/// fn boxed() -> impl Parser<u32> {
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
        $crate::__cons_prepare!($ty [ $($fields)* ])
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
    ([named [$($con:tt)+]] []) => { $crate::pure($($con)+ { })};
    ([pos   [$($con:tt)+]] []) => { $crate::pure($($con)+ ( ))};

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
    ([named [$($con:tt)+]] []) => { $crate::pure($($con)+ { })};
    ([pos   [$($con:tt)+]] []) => { $crate::pure($($con)+ ( ))};

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

#[cfg(doc)]
use std::str::FromStr;

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
/// When consuming the values you can jump straight to a value that implements
/// [`FromStr`] trait then transform into something that your program would actually use. Alternatively
/// you can consume either `String` or `OsString` and parse that by hand. It's better to perform
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
/// annotation. `bpaf_derive` would try to figure out what chain to use for as long as there's no
/// options changing the type: you can use [`fallback`](Parser::fallback_with),
/// [`fallback_with`](Parser::fallback_with), [`guard`](Parser::guard), [`hide`](Parser::hide`) and
/// [`group_help`](Parser::group_help) but not the rest of them.
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     // no annotation at all - `bpaf_derive` inserts implicit `argument` and gets the right type
///     number_1: u32,
///
///     // fallback isn't changing the type so `bpaf_derive` still handles it
///     #[bpaf(fallback(42))]
///     number_2: u32,
///
///     // `bpaf_derive` inserts implicit `argument`, `optional` and the right type
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
    /// `many` preserves any parsing falures and propagates them outwards, with extra
    /// [`catch`](ParseMany::catch) statement you can instead stop at the first value
    /// that failed to parse and ignore it and all the subsequent ones.
    ///
    /// `many` will collect at most one result that does not consume anything from the argument
    /// list allowing using it in combination of any parsers with a fallback. After the first one
    /// it will keep collecting the results as long as they consume something.
    ///
    /// For derive usage `bpaf_derive` would insert implicit `many` when resulting type is a
    /// vector.
    ///
    #[doc = include_str!("docs/many.md")]
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
    /// `some` preserves any parsing falures and propagates them outwards, with extra
    /// [`catch`](ParseSome::catch) statement you can instead stop at the first value
    /// that failed to parse and ignore it and all the subsequent ones.
    ///
    /// `some` will collect at most one result that does not consume anything from the argument
    /// list allowing using it in combination of any parsers with a fallback. After the first one
    /// it will keep collecting the results as long as they consume something.
    ///
    #[doc = include_str!("docs/some.md")]
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
    /// `optional` converts any missing items into is `None` and passes the remaining parsing
    /// failures untouched. With extra [`catch`](ParseOptional::catch) statement you can handle
    /// those failures too.
    ///
    /// # Derive usage
    ///
    /// By default `bpaf_derive` would automatically use optional for fields of type `Option<T>`,
    /// for as long as it's not prevented from doing so by present postprocessing options.
    /// But it's also possible to specify it explicitly.
    ///
    #[doc = include_str!("docs/optional.md")]
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

    // parse
    // {{{ parse
    /// Apply a failing transformation to a contained value
    ///
    /// Transformation preserves present/absent state of the value: to parse an optional value you
    /// can either first try to `parse` it and then mark as [`optional`](Parser::optional) or first
    /// deal with the optionality and then parse a value wrapped in [`Option`]. In most cases
    /// former approach is more concise.
    ///
    /// Similarly it is possible to parse multiple items with [`many`](Parser::many) or
    /// [`some`](Parser::some) by either parsing a single item first and then turning it into a [`Vec`]
    /// or collecting them into a [`Vec`] first and then parsing the whole vector. Former approach
    /// is more concise.
    ///
    /// This is a most general of transforming parsers and you can express
    /// [`map`](Parser::map) and [`guard`](Parser::guard) in terms of it.
    ///
    /// Examples are a bit artificail, to parse a value from string you can specify
    /// the type directly in `argument`'s turbofish and then apply `map`.
    ///
    /// # Derive usage:
    /// `parse` takes a single parameter: function name to call. Function type should match
    /// parameter `F` used by `parse` in combinatoric API.
    ///
    #[doc = include_str!("docs/parse.md")]
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
    /// # Derive usage:
    /// `map` takes a single parameter: function name to call. Function type should match
    /// parameter `F` used by `map` in combinatoric API.
    ///
    #[doc = include_str!("docs/map.md")]
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
    /// If value doesn't satisfy the constraint - parser fails with the specified error message.
    ///
    /// # Derive usage
    /// Derive variant of `guard` takes a function name instead of a closure, mostly to keep things
    /// clean. Second argument can be either a string literal or a constant name for a static [`str`].
    ///
    #[doc = include_str!("docs/guard.md")]
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
    /// Use this value as default if value isn't present on a command line
    ///
    /// Parser would still fail if value is present but failure comes from some transformation
    ///
    #[doc = include_str!("docs/fallback.md")]
    ///
    /// # See also
    /// [`fallback_with`](Parser::fallback_with) would allow to try to fallback to a value that
    /// comes from a failing computation such as reading a file.
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
    #[doc = include_str!("docs/fallback_with.md")]
    ///
    /// # See also
    /// [`fallback`](Parser::fallback) implements similar logic expect that failures aren't expected.
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
    #[doc = include_str!("docs/hide.md")]
    ///
    fn hide(self) -> ParseHide<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHide { inner: self }
    }
    // }}}

    /// Ignore this parser when generating usage line
    ///
    /// Parsers hidden from usage will still show up in available arguments list. Best used on
    /// optional things that augment main application functionality but not define it. You might
    /// use custom usage to indicate that some options are hidden
    ///
    #[doc = include_str!("docs/hide_usage.md")]
    #[must_use]
    fn hide_usage(self) -> ParseHideUsage<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHideUsage { inner: self }
    }

    // {{{ group_help
    /// Attach help message to a complex parser
    ///
    /// `bpaf` inserts the group help message before the block with all the fields
    /// from the inner parser and an empty line after the block.
    ///
    #[doc = include_str!("docs/group_help.md")]
    ///
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
    /// [`positional`] or an [`argument`](NamedArg::argument) doesn't make much sense.
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
    ///         .argument::<String>("Name")
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

    // {{{
    /// Static shell completion
    ///
    /// Allows to ask existing shell completion to provide some information such as file or
    /// directory names or pass though existing shell completion scripts, see
    /// [`ShellComp`](complete_shell::ShellComp) for accessible functionality
    ///
    /// Places function call in place of metavar placeholder, so running `complete_shell` on
    /// something that doesn't have a [`positional`] or [`argument`](NamedArg::argument) doesn't
    /// make much sense.
    ///
    /// **Using this function requires enabling `"autocomplete"` feature, not enabled by default**.
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
    /// `bpaf` can run adjacently restricted parsers multiple times to refine the guesses. It's
    /// best not to have complex inter-fields verification since they might trip up the detection
    /// logic: instead of destricting, for example "sum of two fields to be 5 or greater" *inside* the
    /// `adjacent` parser, you can restrict it *outside*, once `adjacent` done the parsing.
    ///
    /// `adjacent` is available on a trait for better discoverability, it doesn't make much sense to
    /// use it on something other than [`command`](OptionParser::command) or [`construct!`] encasing
    /// several fields.
    ///
    /// There's also similar method [`adjacent`](crate::parsers::ParseArgument) that allows to restrict argument
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

    /// Parse anywhere
    ///
    /// Most generic escape hatch available, in combination with [`any`] allows to parse anything
    /// anywhere, works by repeatedly trying to run the inner parser on each subsequent context.
    /// Can be expensive performance wise especially if parser contains complex logic.
    ///
    #[doc = include_str!("docs/anywhere.md")]
    ///
    /// When using parsers annotated with `anywhere` it's a good idea to place them before other
    /// parsers so combinations they are looking for are not consumed by simplier parsers.
    ///
    #[doc = include_str!("docs/anywhere_1.md")]
    #[must_use]
    fn anywhere(self) -> ParseAnywhere<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseAnywhere {
            inner: self,
            catch: false,
        }
    }

    // consume
    // {{{ to_options
    /// Transform `Parser` into [`OptionParser`] to attach metadata and run
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parser() -> impl Parser<u32> {
    ///     short('i')
    ///         .argument::<u32>("ARG")
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

    #[doc(hidden)]
    #[deprecated = "You should finalize the parser first: see Parser::to_options"]
    fn run(self) -> T
    where
        Self: Sized + Parser<T> + 'static,
    {
        self.to_options().run()
    }

    #[doc(hidden)]
    /// Create a boxed representation for a parser
    ///
    ///
    fn boxed(self) -> ParseBox<T>
    where
        Self: Sized + Parser<T> + 'static,
    {
        ParseBox {
            inner: Box::new(self),
        }
    }
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
/// Both `pure` and [`pure_with`] are designed to put values into structures, to generate fallback
/// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
///
/// See also [`pure_with`] for a pure computation that can fail.
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

/// Wrap a calculated value into a `Parser`
///
/// This parser represents a possibly failing equivalent to [`pure`].
/// It produces `T` by invoking the provided callback without consuming anything from the command
/// line, can be useful with [`construct!`]. As with any parsers `T` should be `Clone` and `Debug`.
///
/// Both [`pure`] and `pure_with` are designed to put values into structures, to generate fallback
/// you should be using [`fallback`](Parser::fallback) and [`fallback_with`](Parser::fallback_with).
///
/// See also [`pure`] for a pure computation that can't fail.

/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn pair() -> impl Parser<bool> {
///     let a = long("flag-a").switch();
///     let b = pure_with::<_, _, String>(|| {
///         // search for history file and try to fish out the last used value ...
///         // if this computation fails - user will see it
///         Ok(false)
///     });
///     construct!([a, b])
/// }
/// ```
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

    /// Run an action appropriate to the failure and produce the exit code
    ///
    /// Prints a message to `stdout` or `stderr` and returns the exit code
    #[allow(clippy::must_use_candidate)]
    pub fn exit_code(self) -> i32 {
        match self {
            ParseFailure::Stdout(msg) => {
                print!("{}", msg); // completions are sad otherwise
                0
            }
            ParseFailure::Stderr(msg) => {
                eprintln!("{}", msg);
                1
            }
        }
    }
}

/// Strip a command name if present at the front when used as a `cargo` command
///
/// See batteries::cargo_helper
#[must_use]
#[doc(hidden)]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    T: 'static,
    P: Parser<T>,
{
    let skip = positional::<String>("cmd")
        .guard(move |s| s == cmd, "")
        .optional()
        .catch()
        .hide();
    construct!(skip, parser).map(|x| x.1)
}
