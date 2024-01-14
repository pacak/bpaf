//! #### Combinatoric API
//!
//! Parse arguments without using proc macros
//!
//! When making a parser in the Combinatoric style API you usually go through those steps
//!
//! 1. Design data type your application will receive
//! 1. Design command line options user will have to pass
//! 1. Create a set of simple parsers
//! 1. Combine and transform simple parsers to create the final data type
//! 1. Transform the resulting [`Parser`](Parser) into [`OptionParser`](OptionParser) to add extra header/footer and run it
//!
//! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
//!  <td style='text-align: center;'>
//!
//! **1**
//! [2](page_02)
//! [3](page_03)
//! [4](page_04)
//! [5](page_05)
//! [6](page_06)
//! [7](page_07)
//! [8](page_08)
//! [9](page_09)
//! [10](page_10)
//! [11](page_11)
//! [12](page_12)
//! [&rarr;](page_02)
//!
//!  </td>
//! </tr></table>
#[allow(unused_imports)] use crate::{*, parsers::*};


/// #### Switch parser
///
/// Let's start with the simplest possible one - a simple switch that gets parsed into a `bool`.
///
/// First of all - the switch needs a name - you can start with [`short`](short) or [`long`](long) and add more
/// names if you want: `long("simple")` or `short('s').long("simple")`. This gives something with
/// the type [`SimpleParser`](SimpleParser):
///
/// ````rust
/// # use bpaf::*;
/// use bpaf::{SimpleParser, parsers::Named};
/// fn simple_switch() -> SimpleParser<Named> {
///     short('s').long("simple")
/// }
/// ````
///
/// With [`SimpleParser::help`](SimpleParser::help) you can attach a help message that will be used in `--help` output.
///
/// From `SimpleParser` you make a switch parser by calling [`SimpleParser::switch`](SimpleParser::switch). Usually, you do it
/// right away without assigning `SimpleParser` to a variable.
///
/// ````rust
/// # use bpaf::*;
/// fn simple_switch() -> impl Parser<bool> {
///     short('s').long("simple").help("A simple switch").switch()
/// }
///
/// fn main() {
///     println!("{:?}", simple_switch().run());
/// }
/// # pub fn options() -> OptionParser<bool> { simple_switch().to_options() }
/// ````
///
/// The switch parser we just made implements trait [`Parser`](Parser). You can run it right right away
/// with [`Parser::run`](Parser::run) or convert to [`OptionParser`](OptionParser) with [`Parser::to_options`](Parser::to_options) and run it with
/// [`OptionParser::run`](OptionParser::run). Later allows attaching extra help information.
///
///
///
/// ```text
/// $ app --simple
/// true
/// ```
///
///
/// When switch is not present on a command line - parser produces `false`.
///
///
///
/// ```text
/// $ app
/// false
/// ```
///
///
/// You also get a help message.
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-s]
///
/// Available options:
///     -s, --simple  A simple switch
///     -h, --help    Prints help information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](super::combinatoric_api)
/// [1](super::combinatoric_api)
/// **2**
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_03)
///
///  </td>
/// </tr></table>
pub mod page_02 {}


/// #### Argument parser
///
/// Next in complexity would be a parser to consume a named argument, such as `-p my_crate`. Same
/// as with the switch parser it starts from a `SimpleParser<Named>` but the next method is
/// [`SimpleParser::argument`](SimpleParser::argument). This method takes a metavariable name - a short description that
/// will be used in the `--help` output. `rustc` also needs to know the parameter type you are
/// trying to parse, there are several ways to do it:
///
/// ````rust
/// # use bpaf::*;
/// fn simple_argument_1() -> impl Parser<String> {
///     // rustc figures out the type from returned value
///     long("name").help("Crate name").argument("String")
/// }
///
/// fn simple_argument_2() -> impl Parser<String> {
///     // type is specified explicitly with turbofish
///     long("name").help("Crate name").argument::<String>("NAME")
/// }
///
/// fn main() {
///     println!("{:?}", simple_argument_2().run());
/// }
/// # pub fn options() -> OptionParser<String> { simple_argument_2().to_options() }
/// ````
///
///
///
/// ```text
/// $ app --name my_crate
/// "my_crate"
/// ```
///
///
///
/// ```text
/// $ app --help
/// Usage: app --name=NAME
///
/// Available options:
///         --name=NAME  Crate name
///     -h, --help       Prints help information
/// ```
///
///
/// You can use any type for as long as it implements [`FromStr`](std::str::FromStr). To parse
/// items that don't implement it you can first parse a `String` or `OsString` and then use
/// [`Parser::parse`](Parser::parse), see the next chapter on how to do that.
///
/// Unlike [`SimpleParser::switch`](SimpleParser::switch), by default parser for argument requires it to be present on a
/// command line to succeed. There's several ways to add a value to fallback to, for example
/// [`Parser::fallback`](Parser::fallback).
///
///
///
/// ```text
/// $ app
/// Error: expected `--name=NAME`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_02)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// **3**
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_04)
///
///  </td>
/// </tr></table>
pub mod page_03 {}


/// #### Positional item parser
///
/// Next last simple option type is a parser for positional items. Since there's no name you use
/// the [`positional`](positional) function directly. Similar to [`SimpleParser::argument`](SimpleParser::argument) this function takes
/// a metavariable name and a type parameter in some form. You can also attach the help message
/// thanks to [`SimpleParser::help`](SimpleParser::help)
///
/// ````rust
/// # use bpaf::*;
/// fn pos() -> impl Parser<String> {
///     positional("URL").help("Url to open")
/// }
///
/// fn main() {
///     println!("{:?}", pos().run());
/// }
/// # pub fn options() -> OptionParser<String> { pos().to_options() }
/// ````
///
/// Same as with argument by default there's no fallback so with no arguments parser fails
///
///
///
/// ```text
/// $ app
/// Error: expected `URL`, pass `--help` for usage information
/// ```
///
///
/// Other than that any name that does not start with a dash or explicitly converted to positional
/// parameter with `--` gets parsed:
///
///
///
/// ```text
/// $ app https://lemmyrs.org
/// "https://lemmyrs.org"
/// ```
///
///
///
/// ```text
/// $ app "strange url"
/// "strange url"
/// ```
///
///
///
/// ```text
/// $ app -- --can-start-with-dash-too
/// "--can-start-with-dash-too"
/// ```
///
///
/// And as usual there's help message
///
///
///
/// ```text
/// $ app --help
/// Usage: app URL
///
/// Available positional items:
///     URL         Url to open
///
/// Available options:
///     -h, --help  Prints help information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_03)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// **4**
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_05)
///
///  </td>
/// </tr></table>
pub mod page_04 {}


/// #### Transforming parsers
///
/// Once you have your simple parsers implemented you might want to improve them further - add
/// fallback values, or change them to consume multiple items, etc. Every primitive (or composite)
/// parser implements [`Parser`](Parser) so most of the transformations are coming from this trait.
///
/// Say you have a parser that takes a crate name as a required argument:
///
/// ````rust
/// use bpaf::*;
/// fn krate() -> impl Parser<String> {
///     long("crate").help("Crate name to process").argument("CRATE")
/// }
/// ````
///
/// You can turn it into, for example, an optional argument - something that returns
/// `Some("my_crate")` if specified or `None` if it wasn't. Or to let the user to pass a multiple
/// of them and collect them all into a `Vec`
///
/// ````rust
/// # use bpaf::*;
/// fn maybe_krate() -> impl Parser<Option<String>> {
///     long("crate")
///         .help("Crate name to process")
///         .argument("CRATE")
///         .optional()
/// }
///
/// fn many_krates() -> impl Parser<Vec<String>> {
///     long("crate")
///         .help("Crate name to process")
///         .argument("CRATE")
///         .many()
/// }
///
/// fn main() {
///     println!("{:?}", many_krates().run());
/// }
/// # pub fn options() -> OptionParser<Vec<String>> { many_krates().to_options() }
/// ````
///
///
///
/// ```text
/// $ app --crate bpaf --crate luhn3
/// ["bpaf", "luhn3"]
/// ```
///
///
/// Transforming a parser with a method from the `Parser` trait usually gives you a new parser back and
/// you can chain as many transformations as you need.
///
/// Transformations available in the `Parser` trait are things like adding fallback values, making
/// the parser optional, making it so it consumes many but at least one value, changing how it is
/// being shown in `--help` output, adding additional validation and parsing on top and so on.
///
/// The order of those chained transformations matters and for some operations using the right order
/// makes code cleaner. For example, suppose you are trying to write a parser that takes an even
/// number and this parser should be optional. There are two ways to write it:
///
/// Validation first:
///
/// ````rust
/// # use bpaf::*;
/// fn even() -> impl Parser<Option<usize>> {
///     long("even")
///         .argument("N")
///         .guard(|&n| n % 2 == 0, "number must be even")
///         .optional()
/// }
/// # pub fn options() -> OptionParser<Option<usize>> { even().to_options() }
/// ````
///
/// Optional first:
///
/// ````rust
/// # use bpaf::*;
/// fn even() -> impl Parser<Option<usize>> {
///     long("even")
///         .argument("N")
///         .optional()
///         .guard(|&n| n.map_or(true, |n| n % 2 == 0), "number must be even")
/// }
/// ````
///
/// In later case validation function must deal with a possibility where a number is absent, for this
/// specific example it makes code less readable.
///
/// Result is identical in both cases:
///
///
///
/// ```text
/// $ app --even 2
/// Some(2)
/// ```
///
///
///
/// ```text
/// $ app --even 3
/// Error: `3`: number must be even
/// ```
///
///
///
/// ```text
/// $ app
/// None
/// ```
///
///
/// One of the important types of transformations you can apply is a set of failing
/// transformations. Suppose your application operates with numbers and uses `newtype` pattern to
/// keep track of what numbers are odd or even. A parser that consumes an even number can use
/// [`Parser::parse`](Parser::parse) and may look like this:
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Copy)]
/// pub struct Even(usize);
///
/// fn mk_even(n: usize) -> Result<Even, &'static str> {
///     if n % 2 == 0 {
///         Ok(Even(n))
///     } else {
///         Err("Not an even number")
///     }
/// }
///
/// fn even() -> impl Parser<Even> {
///     long("even")
///         .argument::<usize>("N")
///         .parse(mk_even)
/// }
///
/// fn main() {
///     println!("{:?}", even().run());
/// }
/// # pub fn options() -> OptionParser<Even> { even().to_options() }
/// ````
///
/// User gets the same/similar output, but the application gets a value in a `newtype` wrapper.
///
///
///
/// ```text
/// $ app --even 2
/// Even(2)
/// ```
///
///
///
/// ```text
/// $ app --even 3
/// Error: couldn't parse `3`: Not an even number
/// ```
///
///
///
/// ```text
/// $ app
/// Error: expected `--even=N`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_04)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// **5**
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_06)
///
///  </td>
/// </tr></table>
pub mod page_05 {}


/// #### Combining multiple simple parsers
///
/// A single-item option parser can only get you so far. Fortunately, you can combine multiple
/// parsers with [`construct!`](construct!) macro.
///
/// For sequential composition (all the fields must be present) you write your code as if you are
/// constructing a structure, enum variant or a tuple and wrap it with `construct!`. Both a
/// constructor and parsers must be present in the scope. If instead of a parser you have a
/// function that creates one - just add `()` after the name:
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Copy)]
/// # pub
/// struct Options {
///     alpha: usize,
///     beta: usize
/// }
///
/// fn alpha() -> impl Parser<usize> {
///     long("alpha").argument("ALPHA")
/// }
///
/// fn both() -> impl Parser<Options> {
///     let beta = long("beta").argument("BETA");
///     // call `alpha` function, and use result to make parser
///     // for field `alpha`,
///     // use parser `beta` from the scope for field `beta`
///     construct!(Options { alpha(), beta })
/// }
///
/// fn main() {
///     println!("{:?}", both().run());
/// }
/// # pub fn options() -> OptionParser<Options> { both().to_options() }
/// ````
///
///
///
/// ```text
/// $ app --alpha 10 --beta 20
/// Options { alpha: 10, beta: 20 }
/// ```
///
///
/// For named parsers order doesn't matter
///
///
///
/// ```text
/// $ app --beta 20 --alpha 10
/// Options { alpha: 10, beta: 20 }
/// ```
///
///
///
/// ```text
/// $ app --help
/// Usage: app --alpha=ALPHA --beta=BETA
///
/// Available options:
///         --alpha=ALPHA
///         --beta=BETA
///     -h, --help         Prints help information
/// ```
///
///
/// If you are using positional parsers - they must go to the right-most side and will run in
/// the order you specify them. For named parsers order affects only the `--help` message.
///
/// The second type of composition `construct!` offers is a parallel composition. You pass multiple
/// parsers that produce the same result type in `[]` and `bpaf` selects one that fits best with
/// the data user gave.
///
/// ````rust
/// # use bpaf::*;
/// fn distance() -> impl Parser<f64> {
///     let km = long("km").help("Distance in km").argument::<f64>("KM");
///     let miles = long("mi").help("Distance in miles").argument::<f64>("MI").map(|d| d * 1.621);
///     construct!([km, miles])
/// }
///
/// fn main() {
///     println!("{:?}", distance().run());
/// }
/// # pub fn options() -> OptionParser<f64> { distance().to_options() }
/// ````
///
/// Parser `distance` accepts either `--km` or `--mi`, but not both at once and produces a single
/// `f64` converted to km.
///
///
///
/// ```text
/// $ app --km 42
/// 42.0
/// ```
///
///
///
/// ```text
/// $ app --mi 42
/// 68.082
/// ```
///
///
///
/// ```text
/// $ app --km 42 --mi 42
/// Error: `--mi` cannot be used at the same time as `--km`
/// ```
///
///
/// Help indicates that either value is accepted
///
///
///
/// ```text
/// $ app --help
/// Usage: app (--km=KM | --mi=MI)
///
/// Available options:
///         --km=KM  Distance in km
///         --mi=MI  Distance in miles
///     -h, --help   Prints help information
/// ```
///
///
/// If parsers inside parallel composition parse the same items from the command line - the longest
/// possible match should go first since `bpaf` picks an earlier parser if everything else is
/// equal, otherwise it does not matter. In this example `construct!([miles, km])` produces the
/// same results as `construct!([km, miles])` and only `--help` message is going to be different.
///
/// Parsers created with [`construct!`](construct!) still implement the [`Parser`](Parser) trait so you can apply more
/// transformation on top. For example same as you can make a simple parser optional - you can make
/// a composite parser optional. Parser transformed this way will succeed if both `--alpha` and
/// `--beta` are present or neither of them:
///
/// ````rust
/// # use bpaf::*;
/// # #[derive(Debug, Clone)] pub
/// struct Options {
///     alpha: usize,
///     beta: usize
/// }
///
/// fn parser() -> impl Parser<Option<Options>> {
///     let alpha = long("alpha").argument("ALPHA");
///     let beta = long("beta").argument("BETA");
///     construct!(Options { alpha, beta }).optional()
/// }
///
/// fn main() {
///     println!("{:?}", parser().run() );
/// }
/// # pub fn options() -> OptionParser<Option<Options>> { parser().to_options() }
/// ````
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--alpha=ALPHA --beta=BETA]
///
/// Available options:
///         --alpha=ALPHA
///         --beta=BETA
///     -h, --help         Prints help information
/// ```
///
///
/// Here `optional` parser returns `Some` value if inner parser succeeds
///
///
///
/// ```text
/// $ app --alpha 10 --beta 15
/// Some(Options { alpha: 10, beta: 15 })
/// ```
///
///
/// Or `None` if neither value is present
///
///
///
/// ```text
/// $ app
/// None
/// ```
///
///
/// For parsers that are partially successfull user will get an error
///
///
///
/// ```text
/// $ app --alpha 10
/// Error: expected `--beta=BETA`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_05)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// **6**
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_07)
///
///  </td>
/// </tr></table>
pub mod page_06 {}


/// #### `flag` - general version of `switch`
///
/// `bpaf` contains a few more primitive parsers: [`SimpleParser::flag`](SimpleParser::flag) and [`SimpleParser::req_flag`](SimpleParser::req_flag).
/// First one is a more general case of [`SimpleParser::switch`](SimpleParser::switch) that lets you to make a parser for a
/// flag, but instead of producing `true` or `false` it can produce one of two values of the same
/// type.
///
/// ````rust
/// # use bpaf::*;
/// fn simple_switch() -> impl Parser<u8> {
///     short('s').long("simple").help("A simple flag ").flag(1, 0)
/// }
///
/// fn main() {
///     println!("{:?}", simple_switch().run());
/// }
/// # pub fn options() -> OptionParser<u8> { simple_switch().to_options() }
/// ````
///
///
///
/// ```text
/// $ app --simple
/// 1
/// ```
///
///
///
/// ```text
/// $ app
/// 0
/// ```
///
///
/// You can use [`SimpleParser::flag`](SimpleParser::flag) to crate an inverted switch like `--no-logging` that would
/// return `false` when switch is present and `true` otherwise or make it produce type with more
/// meaning such as `Logging::Enabled` / `Logging::Disabled`.
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_06)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// **7**
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_08)
///
///  </td>
/// </tr></table>
pub mod page_07 {}


/// #### Types of failures
///
/// Let's consider a parser that takes an optional numeric argument and makes sure it's below 10 if
/// present.
///
/// ````rust
/// # use bpaf::*;
/// fn numeric() -> impl Parser<Option<usize>> {
///     short('n')
///         .argument::<usize>("N")
///         .guard(|n| *n <= 10, "N must be at or below 10")
///         .optional()
/// }
///
/// fn main() {
///     println!("{:?}", numeric().run());
/// }
/// # pub fn options() -> OptionParser<Option<usize>> { numeric().to_options() }
/// ````
///
/// Option is present and is valid
///
///
///
/// ```text
/// $ app -n 1
/// Some(1)
/// ```
///
///
/// Option is missing
///
///
///
/// ```text
/// $ app
/// None
/// ```
///
///
/// Option is present, it is a number but it's larger than the validation function allows
///
///
///
/// ```text
/// $ app -n 11
/// Error: `11`: N must be at or below 10
/// ```
///
///
/// Option is present, but the value is not a number
///
///
///
/// ```text
/// $ app -n five
/// Error: couldn't parse `five`: invalid digit found in string
/// ```
///
///
/// `short('n').argument("N")` part of the parser succeeds in the first and the third cases since
/// the parameter is present and it is a valid number, in the second care it fails with "value not
/// found", and in the fourth case it fails with "value is not valid".
///
/// Result produced by `argument` gets handled by `guard`. Failures in the second and the fourth
/// cases are passed as is, successes are checked with "less than 11" and turned into failures if
/// check fails - in the third case.
///
/// Result of `guard` gets into `optional` which converts present values into `Some` values, "value
/// not found" types of errors into `None` and keeps the rest of the failures as is.
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_07)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// **8**
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_09)
///
///  </td>
/// </tr></table>
pub mod page_08 {}


/// #### `req_flag` - half of the `flag`
///
/// [`SimpleParser::flag`](SimpleParser::flag) handles missing value by using second of the provided values,
/// [`SimpleParser::req_flag`](SimpleParser::req_flag) instead fails with "value is missing" error - this makes it useful
/// when making combinations from multiple parsers.
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Copy)]
/// # pub
/// enum Vote {
///     Yes,
///     No,
///     Undecided
/// }
///
/// fn parser() -> impl Parser<Vote> {
///     let yes = long("yes").help("vote yes").req_flag(Vote::Yes);
///     let no = long("no").help("vote no").req_flag(Vote::No);
///
///     // parsers expect `--yes` and `--no` respectively.
///     // their combination takes either of those
///     // and fallback handles the case when both values are absent
///     construct!([yes, no]).fallback(Vote::Undecided)
/// }
///
/// fn main() {
///     println!("{:?}", parser().run());
/// }
/// # pub fn options() -> OptionParser<Vote> { parser().to_options() }
/// ````
///
/// Help message reflects that `--yes` and `--no` options are optional and mutually exclusive
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--yes | --no]
///
/// Available options:
///         --yes   vote yes
///         --no    vote no
///     -h, --help  Prints help information
/// ```
///
///
///
/// ```text
/// $ app --yes
/// Yes
/// ```
///
///
/// [`Parser::fallback`](Parser::fallback) handles the case when both values are missing
///
///
///
/// ```text
/// $ app
/// Undecided
/// ```
///
///
/// And `bpaf` itself handles the case where both values are present - in this scenario both
/// parsers can succeed, but in the alternative combination only one parser gets to consume its
/// arguments. Since combined parser runs only once (there's no [`Parser::many`](Parser::many) or
/// [`Parser::some`](Parser::some)) present) - only one value is consumed. One of the requirements for parsing to
/// succeed - all the items from the command line must be consumed by something.
///
///
///
/// ```text
/// $ app --yes --no
/// Error: `--no` cannot be used at the same time as `--yes`
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_08)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// **9**
/// [10](page_10)
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_10)
///
///  </td>
/// </tr></table>
pub mod page_09 {}


/// #### `any` - parse a single arbitrary item from a command line
///
/// **[`any`](any) is designed to consume items that don’t fit into the usual [`flag`](SimpleParser::flag)
/// / [`switch`](SimpleParser::switch) / [`argument`](SimpleParser::argument) / [`positional`](positional)
/// / [`command`](OptionParser::command) classification, in most cases you don’t need to use it**.
///
/// To understand how `any` works you first need to learn about positional and named arguments.
/// Named argument starts with a name or consists of a name only and can be consumed in any order,
/// while positional doesn't have a name and are consumed sequentially:
///
/// If the app defines two named parsers with long names `alpha` and `beta` those two user inputs
/// are identical
///
/// ````text
/// --alpha --beta
/// ````
///
/// ````text
/// --beta --alpha
/// ````
///
/// But with positional items `alpha` and `beta` results are going to be different. Earlier
/// positional parser in the first example will capture value `alpha`, later positional parser will
/// capture `beta`. For the second example earlier parser will capture `beta`.
///
/// ````text
/// alpha beta
/// ````
///
/// ````text
/// beta alpha
/// ````
///
/// It is possible to mix named parsers with positional ones, as long as check for positional is
/// done after. Positional and named parsers won't know that parameters for their conterparts are
/// present. In this example named parsers will only see presence of `--alpha` and `--beta`, while
/// positional parser will encounter only `alpha` and `beta`.
///
/// ````text
/// --alpha --beta alpha beta
/// ````
///
/// Parser created with `any` gets shown everything and it is up to parser to decide if the value it
/// gets is a match or not. By default `any` parser behaves as positional and only looks at the
/// first unconsumed item, but can be modified with [`SimpleParser::anywhere`](SimpleParser::anywhere) to look at all the
/// unconsumed items and producing the first value it accepts. `check` parameter to `any` should
/// take `String` or `OsString` as input and decide if parser should match on this value.
///
/// Let's make a parser to accept windows style flags (`/name:value`). Parser should take a name -
/// `"help"` to parse `/help` and produce value T, parsed from `value`.
///
/// ````rust
/// # use bpaf::*;
/// # use std::str::FromStr;
/// // this makes a generic version for all the windows like items
/// fn win<T>(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<T>
///     where T: FromStr, <T as FromStr>::Err: std::fmt::Display,
/// {
///     any::<String, _, _>(meta, move |s: String|
///         {
///             // check function will be called for all the unconsumed items on the command line.
///             // strip_prefix functions sequentially consume `/`, name and `:`, producing the
///             // leftovers, for `/size:1024` it will be `1024`
///             Some(
///              s.strip_prefix("/")?
///              .strip_prefix(name)?
///              .strip_prefix(":")?
///              // this packs leftovers into a String
///              .to_owned())
///          })
///         .help(help)
///         // apply it to each unconsumed item
///         .anywhere()
///         // and try to parse string into T
///         .parse(|s| s.parse())
/// }
///
/// fn size() -> impl Parser<usize> {
///     // and finally make it into a parser that accepts the size
///     win("/size:MB", "size", "File size")
/// }
///
/// fn main() {
///     println!("{:?}", size().run());
/// }
/// # pub fn options() -> OptionParser<usize> { size().to_options() }
/// ````
///
/// Parser works as expected
///
///
///
/// ```text
/// $ app /size:1024
/// 1024
/// ```
///
///
/// Produces somewhat reasonable error message
///
///
///
/// ```text
/// $ app /size:fourty-two
/// Error: couldn't parse `/size:fourty-two`: invalid digit found in string
/// ```
///
///
/// And even generates the help message (which can be further improved with custom metavar)
///
///
///
/// ```text
/// $ app --help
/// Usage: app /size:MB
///
/// Available options:
///     /size:MB    File size
///     -h, --help  Prints help information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_09)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// **10**
/// [11](page_11)
/// [12](page_12)
/// [&rarr;](page_11)
///
///  </td>
/// </tr></table>
pub mod page_10 {}


/// #### Subcommand parsers
///
/// To make a parser for a subcommand you make an `OptionParser` for that subcommand first as if it
/// was the only thing your application would parse then turn it into a regular [`Parser`](Parser)
/// you can further compose with [`OptionParser::command`](OptionParser::command).
///
/// This gives `SimpleParser<Command>` back, you can add aliases or tweak the help message if you want to.
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone)]
/// pub enum Options {
///     /// Run a binary
///     Run {
///         /// Name of a binary to run
///         bin: String,
///
///         /// Arguments to pass to a binary
///         args: Vec<String>,
///     },
///     /// Compile a binary
///     Build {
///         /// Name of a binary to build
///         bin: String,
///
///         /// Compile the binary in release mode
///         release: bool,
///     },
/// }
///
/// // combine mode gives more flexibility to share the same code across multiple parsers
/// fn run() -> impl Parser<Options> {
///     let bin = long("bin").help("Name of a binary to run").argument("BIN");
///     let args = positional("ARG")
///         .strict()
///         .help("Arguments to pass to a binary")
///         .many();
///
///     construct!(Options::Run { bin, args })
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let run = run().to_options().descr("Run a binary").command("run");
///
///     let bin = long("bin")
///         .help("Name of a binary to build ")
///         .argument("BIN");
///     let release = long("release")
///         .help("Compile the binary in release mode")
///         .switch();
///     let build = construct!(Options::Build { bin, release })
///         .to_options()
///         .descr("Compile a binary")
///         .command("build");
///
///     construct!([run, build]).to_options()
/// }
///
/// pub fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// Help contains both commands, bpaf takes short command description from the inner command
/// description
///
///
///
/// ```text
/// $ app --help
/// Usage: app COMMAND ...
///
/// Available options:
///     -h, --help  Prints help information
///
/// Available commands:
///     run         Run a binary
///     build       Compile a binary
/// ```
///
///
/// Same as before each command gets its own help message
///
///
///
/// ```text
/// $ app run --help
/// Run a binary
///
/// Usage: app run --bin=BIN -- [ARG]...
///
/// Available positional items:
///     ARG            Arguments to pass to a binary
///
/// Available options:
///         --bin=BIN  Name of a binary to run
///     -h, --help     Prints help information
/// ```
///
///
/// And can be executed separately
///
///
///
/// ```text
/// $ app run --bin basic
/// Run { bin: "basic", args: [] }
/// ```
///
///
///
/// ```text
/// $ app build --bin demo --release
/// Build { bin: "demo", release: true }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_10)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// **11**
/// [12](page_12)
/// [&rarr;](page_12)
///
///  </td>
/// </tr></table>
pub mod page_11 {}


/// #### Improving the user experience
///
/// Once you have the final parser done there are still a few ways you can improve user experience.
/// [`OptionParser`](OptionParser) comes equipped with a few methods that let you set version number,
/// description, help message header and footer and so on.
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_to_options.md"))\]
///
/// There are a few other things you can do:
///
/// * group some of the primitive parsers into logical blocks for `--help` message with
///   [`Parser::group_help`](Parser::group_help)
/// * add tests to make sure important combinations are handled the way they are supposed to
///   after any future refactors with [`OptionParser::run_inner`](OptionParser::run_inner)
/// * add a test to make sure that bpaf internal invariants are satisfied with
///   [`OptionParser::check_invariants`](OptionParser::check_invariants)
/// * generate user documentation in manpage and markdown formats with
///   [`OptionParser::render_manpage`](OptionParser::render_manpage) and [`OptionParser::render_markdown`](OptionParser::render_markdown)
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_11)
/// [1](super::combinatoric_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [11](page_11)
/// **12**
///
///  </td>
/// </tr></table>
pub mod page_12 {}
