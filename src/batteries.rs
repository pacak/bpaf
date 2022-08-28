//! # Batteries included
//!
//! `bpaf` comes with a few extra functions that use only public API in their implementation. You
//! might find them useful either for your code or as an inspiration source
//!
//! **To use anything in this module you need to enable `batteries` cargo feature.**
//!
//! Examples contain combinatoric usage, for derive usage you should create a parser function and
//! use `external` annotation.

use crate::{construct, short, Named, Parser};

/// `--verbose` and `--quiet` flags with results encoded as number
///
/// Parameters specify the offset and minimal/maximal values. Parser accepts many `-v | --verbose` and
/// `-q | --quiet` to increase and decrease verbosity respectively
///
/// # Usage
///
/// ```rust
/// # use bpaf::*;
/// use bpaf::batteries::*;
/// fn verbose() -> impl Parser<usize> {
///     verbose_and_quiet_by_number(2, 0, 5).map(|v| v as usize)
/// }
/// ```
#[must_use]
pub fn verbose_and_quiet_by_number(offset: isize, min: isize, max: isize) -> impl Parser<isize> {
    #![allow(clippy::cast_possible_wrap)]
    let verbose = short('v')
        .long("verbose")
        .help("Increase output verbosity, can be used several times")
        .req_flag(())
        .many()
        .map(|v| v.len() as isize);

    let quiet = short('q')
        .long("quiet")
        .help("Decrease output verbosity, can be used several times")
        .req_flag(())
        .many()
        .map(|v| v.len() as isize);

    construct!(verbose, quiet).map(move |(v, q)| (v - q + offset).max(min).min(max))
}

/// `--verbose` and `--quiet` flags with results choosen from a slice of values
///
/// Parameters specify an array of possible values and a default index
///
/// # Usage
/// ```rust
/// # use bpaf::*;
/// use bpaf::batteries::*;
///
/// #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// enum Level {
///    Error,
///    Warning,
///    Info,
///    Debug,
///    Trace,
/// }
///
/// fn verbose() -> impl Parser<Level> {
///     use Level::*;
///     verbose_by_slice(2, [Error, Warning, Info, Debug, Trace])
/// }
/// # let parser = verbose().to_options();
/// # let res = parser.run_inner(Args::from(&[])).unwrap();
/// # assert_eq!(Level::Info, res);
/// # let res = parser.run_inner(Args::from(&["-q"])).unwrap();
/// # assert_eq!(Level::Warning, res);
/// # let res = parser.run_inner(Args::from(&["-qqq"])).unwrap();
/// # assert_eq!(Level::Error, res);
/// # let res = parser.run_inner(Args::from(&["-qqqq"])).unwrap();
/// # assert_eq!(Level::Error, res);
/// # let res = parser.run_inner(Args::from(&["-vvvvq"])).unwrap();
/// # assert_eq!(Level::Trace, res);
/// ```
#[must_use]
pub fn verbose_by_slice<T: Copy + 'static, const N: usize>(
    offset: usize,
    items: [T; N],
) -> impl Parser<T> {
    #![allow(clippy::cast_possible_wrap)]
    #![allow(clippy::cast_sign_loss)]
    verbose_and_quiet_by_number(offset as isize, 0, items.len() as isize - 1)
        .map(move |i| items[i as usize])
}

/// Flag that can be toggled multiple times
///
/// Usually `bpaf` only allows to parse a single instance for every invocation unless
/// [`many`](Parser::many) or [`some`](Parser::some) is used. `toggle_flag` consumes
/// multiple instances of two different flags and returns last specified value.
///
/// This function relies on a fact that selection between two different parsers prefers left most
/// value. This helps to preserve relative order of parsrs.
/// You can use similar approach to combine multiple flags accounting for their relative order.
///
/// # Example
/// ```console
/// $ app --banana --no-banana --banana --banana
/// Some(Banana)
/// $ app
/// None
/// ```
///
/// # Usage
/// ```rust
/// # use bpaf::*;
/// use bpaf::batteries::toggle_flag;
///
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// enum Select {
///     Banana,
///     NoBanana,
/// }
///
/// fn pick() -> impl Parser<Option<Select>> {
///     toggle_flag(long("banana"), Select::Banana, long("no-banana"), Select::NoBanana)
/// }
/// ```
pub fn toggle_flag<T: Copy + 'static>(
    a: Named,
    val_a: T,
    b: Named,
    val_b: T,
) -> impl Parser<Option<T>> {
    let a = a.req_flag(val_a);
    let b = b.req_flag(val_b);
    construct!([a, b]).many().map(|xs| xs.into_iter().last())
}
