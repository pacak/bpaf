//! # Applicative functors? What is it about?
//!
//! You don't need to read/understand this chapter in order to use the library but it might
//! help to understand what makes it tick.

//! ## Functors
//!
//! Let's start by talking about what a `Functor` is. Wikipedia defines it as a "design pattern
//! that allows for a generic type to apply a function inside without changing the structure of
//! the generic type". Sounds scary, but in Rust terms it's a trait that takes a value or values
//! in a container (or more general *value in a context* ) such as `Option<A>` and a function
//! `fn(A) -> B` and gives you `Option<B>` back.
//!
//! Closest analogy in a real code you can write in Rust right now would be modifying an `Option`
//! using only `Option::map`:
//! ```rust
//! fn plus_one(input: Option<u32>) -> Option<u32> {
//!     input.map(|i| i + 1)
//! }
//!
//! let present = Some(10);
//! let absent = None;
//!
//! assert_eq!(plus_one(present), Some(11));
//! assert_eq!(plus_one(absent), None);
//! ```
//!
//! `Vec`, `Result` and other types that implement `map` are `Functors` as well, but `Functor`
//! is not limited just to containers - you don't have to have a value inside to be able to
//! manipulate it. In fact a regular rust function is also a `Functor` if you squint hard enough.
//! Consider `Reader` that allows you to perform transformations on a *value in a context* `T`
//! without having any value until it the execution time:
//!
//! ```rust
//! struct Reader<T>(Box<dyn Fn(T) -> T>);
//! impl<T: 'static> Reader<T> {
//!     /// Initialize an new value in a context
//!     fn new() -> Self {
//!         Self(Box::new(|x| x))
//!     }
//!
//!     /// Modify a value in a context
//!     fn map<F:  Fn(T) -> T + 'static>(self, f: F) -> Self {
//!         Self(Box::new(move |x| f((self.0)(x))))
//!     }
//!
//!     /// Apply the changes by giving it the initial value
//!     fn run(self, input: T) -> T {
//!         (self.0)(input)
//!     }
//! }
//!
//! let val = Reader::<u32>::new();
//! let val = val.map(|x| x + 1);
//! let res = val.run(10);
//! assert_eq!(res, 11);
//! ```
//!
//! Not all the collections are `Functors` - by `Functor` laws mapping the *value in context*
//! shouldn't change the shape so any collections where shape depends on a value, such as `HashSet`
//! or `BTreeSet` are out.

//! ## Applicative Functors
//!
//! `map` in `Functor` is limited to a single *value in a context*, `Applicative Functor` extends it
//! to operations combining multiple values, closest Rust analogy would be doing computations on
//! `Option` or `Result` using only `?`, having `Some`/`Ok` around the whole expression and not using `return`.
//! ```rust
//! fn add_numbers(input_a: Option<u32>, input_b: Option<u32>) -> Option<u32> {
//!     Some(input_a? + input_b?)
//! }
//!
//! let present_1 = Some(10);
//! let present_2 = Some(20);
//! let absent = None;
//!
//! assert_eq!(add_numbers(present_1, present_2), Some(30));
//! assert_eq!(add_numbers(present_1, absent), None);
//! assert_eq!(add_numbers(absent, absent), None);
//! ```
//!
//! Similarly to `Functors`, `Applicative Functors` are not limited to containers and can
//! represent *a value in an arbitrary context*.
//!
//! `Try` trait (`?`) for `Option` and `Result` short circuits when it finds a missing value,
//! but `Applicative Functors` in general don't have to - in fact to implement dynamic completion
//! `bpaf` needs to check items past the first failure point to collect all the possible
//! completions.

//! ## Alternative Functors
//!
//! So far `Applicative Functors` allow us to create structs containing multiple fields out of
//! individual parsers for each field. `Alternative` extends `Applicative` with two extra
//! things: one for combining two *values in a context* into one and and an idenity element
//! for this operation. In Rust a closest analogy would be `Option::or` and `Option::None`:
//!
//! ```rust
//! fn pick_number(a: Option<u32>, b: Option<u32>) -> Option<u32> {
//!     a.or(b)
//! }
//!
//! let present_1 = Some(10);
//! let present_2 = Some(20);
//! let empty = None;
//! assert_eq!(pick_number(present_1, present_2), present_1);
//! assert_eq!(pick_number(present_1, empty), present_1);
//! assert_eq!(pick_number(empty, present_1), present_1);
//! assert_eq!(pick_number(empty, empty), empty);
//! ```

//! ## `Parser` trait and `construct!` macro
//!
//! [`Parser`] trait defines a context for values and gives access to `Functor` laws and [`construct!`]
//! macro allows to compose several values according to `Applicative` and `Alternative` laws.

//! ## So why use `Applicative Functors` then?
//!
//! As a user I want to be able to express requirements using full power of Rust algebraic
//! datatypes: `struct` for product types and `enum` for sum types. To give an example -
//! `cargo-show-asm` asks user to specify what to output - Intel or AT&T asm, LLVM or Rust's MIR
//! and opts to represent it as one of four flags: `--intel`, `--att`, `--llvm` and `--mir`. While
//! each flag can be though of a boolean value - present/absent - consuming it as an `enum` with four
//! possible values is much more convenient compared to a struct-like thing that can have any
//! combination of the flags inside:
//!
//! ```no_check
//! /// Format selection as enum - program needs to deal with just one format
//! enum Format {
//!     Intel,
//!     Att,
//!     Llvm,
//!     Mir
//! }
//!
//! /// Format selection as struct - can represent any possible combination of formats
//! struct Formats {
//!     intel: bool,
//!     att: bool,
//!     llvm: bool,
//!     mir: bool,
//! }
//! ```
//!
//! `Applicative` interface gives just enough power to compose simple parsers as an arbitrary tree
//! ready for consumption.
//!
//! As a library author I need to be able to extract information from the tree constructed by user
//! to generate `--help` information and do command line completion. As long as the tree uses only
//! `Applicative` powers - it is possible to evaluate it without giving it any input.
//! Adding `Monadic` powers (deciding what to parse next depending on the previous input) would
//! make this impossible.
//!
//! So `Applicative Functors` sits right in the middle between what users want to express and
//! library can consume.
//!
//! To recap - all sorts of Functors listed here only define laws to how individual parts are
//! composed, how values in context can be transformed and how pure values can be turned into a
//! functor, but not how the values are parsed or how they can be extracted.

//! ## Putting the values into a context
//!
//! Similarly to how `Reader` defined above `bpaf`'s `Parsers` don't actually have values inside
//! until they are executed. Instead starting points ([`flag`](NamedArg::flag), [`positional`],
//! [`argument`](NamedArg::argument), etc) define what exactly needs to be consumed, various mapping
//! functions define transformations, [`construct!`] composes them and defines the relative order
//! values should be consumed. Not everything present inside [`Parser`] can be repesented in terms
//! of plain applicative functors - specifically [`parse`](Parser::parse) is not and it is best
//! though of as a function that takes one applicative and gives a different applicative back.
//! The actual values will show up inside once `bpaf` starts running the [`OptionParser`] with
//! [`run`](OptionParser::run).

//! ## Taking the results out
//!
//! The rest of the execution is relatively simple: getting console arguments from OS, doing the
//! initial split into short/long flags and standalone words, disambiguating groups of short
//! options from short options with attached values and applying all the transformations like
//! `Reader::run` above would do.

#[cfg(doc)]
use crate::*;
