//! # Batteries included - helpful parsers that use only public API
//!
//! `bpaf` comes with a few extra functions that use only public API in their implementation. You
//! might find them useful either for your code or as an inspiration source
//!
//! **To use anything in this module you need to enable `batteries` cargo feature.**
//!
//! Examples contain combinatoric usage, for derive usage you should create a parser function and
//! use `external` annotation.

use crate::{construct, parsers::NamedArg, positional, short, Parser};

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

    construct!(verbose, quiet).map(move |(v, q)| (v - q + offset).clamp(min, max))
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

/// Pick last passed value between two different flags
///
/// Usually `bpaf` only allows to parse a single instance for every invocation unless
/// you specify [`many`](Parser::many) or [`some`](Parser::some). `toggle_flag` would consume
/// multiple instances of two different flags and returns last specified value.
///
/// This function relies on a fact that selection between two different parsers prefers left most
/// value. This helps to preserve relative order of parsrs.
/// You can use similar approach to combine multiple flags accounting for their relative order.
///
/// Parser returns `Optional<T>` value, you can add a fallback with [`map`](Parser::map) or turn
/// missing value info failure with a custom error message with [`parse`](Parser::parse).
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
    a: NamedArg,
    val_a: T,
    b: NamedArg,
    val_b: T,
) -> impl Parser<Option<T>> {
    let a = a.req_flag(val_a);
    let b = b.req_flag(val_b);
    construct!([a, b]).many().map(|xs| xs.into_iter().last())
}

/// Strip a command name if present at the front when used as a `cargo` command
///
/// When implementing a cargo subcommand parser needs to be able to skip the first argument which
/// is always the same as the executable name without `cargo-` prefix. For example if executable name is
/// `cargo-cmd` so first argument would be `cmd`. `cargo_helper` helps to support both invocations:
/// with name present when used via cargo and without it when used locally.
///
/// You can read the code of this function as this approximate sequence of statements:
/// 1. Want to parse a word
/// 2. Word must match a given string literal
/// 3. It's okay if it's missing
/// 4. It's also okay when it's not matching expectations, don't consume it in this case
/// 5. And don't show anything to the user in `--help` or completion
/// 6. Parse this word and then everything else as a tuple, return that second item.
///
#[doc = include_str!("docs/cargo_helper.md")]
///
#[must_use]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    P: Parser<T>,
{
    let skip = positional::<String>("cmd")
        .guard(move |s| s == cmd, "")
        .optional()
        .catch()
        .hide();
    construct!(skip, parser).map(|x| x.1)
}
