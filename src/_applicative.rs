//! # Applicative functors? What is it about?
//!
//! You don't need to read/understand this chapter in order to use the library but it might
//! help to understand what makes it tick.

//! ## Functors
//!
//! Let's start by talking about what a `Functor` is. In Haskell it is defined as
//!
//! ```haskell
//! class Functor f where
//!     fmap :: (a -> b) -> f a -> f b
//! ```
//!
//! looks scary, but in Rust terms it's a trait that takes a *value in a context* or a container such as
//! `Option<A>` and a function `fn(A) -> B` and gives you `Option<B>` back.
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
//! `Vec`, `Result` and other types that implement `map` are Functors as well, but functors are not
//! limited just to containers - you don't have to have a value inside to be able to manipulate it.
//! Consider `Reader` that allows you to perform pure transformations on a *value in a context* `T`
//! without having any value until it's ready to execute:
//!
//! ```rust
//! # use std::marker::PhantomData;
//! struct Reader<T>(PhantomData<T>, Box<dyn Fn(T) -> T>);
//!
//! impl<T> Reader<T> {
//!     /// Initialize an new value in a context
//!     fn new() -> Self {
//!         Self(PhantomData, Box::new(|x| x))
//!     }
//!
//!     /// Modify a value in a context
//!     fn map<F>(self, f: F) -> Self
//!     where
//!         F: Fn(T) -> T + 'static,
//!         T: 'static,
//!     {
//!         Self(
//!             PhantomData,
//!             Box::new(move |x| {
//!                 let inner = &self.1;
//!                 f(inner(x))
//!             }),
//!         )
//!     }
//!
//!     /// Apply the changes by giving it the initial value
//!     fn run(self, input: T) -> T {
//!         let f = &self.1;
//!         f(input)
//!     }
//! }
//!
//! let val = Reader::<u32>::new();
//! let val = val.map(|x| x + 1);
//! let res = val.run(10);
//! assert_eq!(res, 11);
//! ```

//! ## Applicative Functors
//!
//! `map` in `Functors` is limited to a single value, `Applicative Functors` extend it to multiple values, closest Rust
//! analogy would be doing computations on `Option` or `Result` using only `?` and `Some`/`Ok` but
//! without making any decisions on input values after they been extracted from `Option`.
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
//!

//! ## `Parser` trait
//!
//! Similarly to `Reader` struct described above individual parsers don't actually contain a value
//! but only a value in context - they contain a computation describing the initial consumer plus
//! all the transformations and validations layered on top. [`construct!`] macro takes several
//! parser objects and composes them similarly to `add_numbers` function above.

#[cfg(doc)]
use crate::construct;
