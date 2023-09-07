//! #### Derive API tutorial
//!
//! Create a parser by defining a structure
//!
//! When making a parser using Derive API you should go through approximately following steps:
//!
//! 1. Design data type your application will receive
//! 1. Design command line options user will have to pass
//! 1. Add `#[derive(Bpaf, Debug, Clone)]` on top of your type or types
//! 1. Add `#[bpaf(xxx)]` annotations on types and fields
//! 1. And `#[bpaf(options)]` to the top type
//! 1. Run the resulting parser
//!
//! Let’s go through some of them in more detail.
//!
//! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
//!  <td style='text-align: center;'>
//!
//! **1**
//! [2](page_1)
//! [3](page_2)
//! [4](page_3)
//! [5](page_4)
//! [6](page_5)
//! [7](page_6)
//! [8](page_7)
//! [9](page_8)
//! [10](page_9)
//! [&rarr;](page_1)
//!
//!  </td>
//! </tr></table>
#[allow(unused_imports)] use crate::{*, parsers::*};


/// #### Getting started with derive macro
///
/// Let's take a look at a simple example
///
/// ````rust
/// use bpaf::*;
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A custom switch
///     switch: bool,
///
///     /// A custom argument
///     argument: usize,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// `bpaf` generates a help message
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--switch] --argument=ARG
///
/// Available options:
///         --switch        A custom switch
///         --argument=ARG  A custom argument
///     -h, --help          Prints help information
/// ```
///
///
/// And parsers for two items: numeric argument is required, boolean switch is optional and fall back value
/// is `false`.
///
///
///
/// ```text
/// $ app --switch
/// expected `--argument=ARG`, pass `--help` for usage information```
///
///
///
/// ```text
/// $ app --switch --argument 42
/// Options { switch: true, argument: 42 }
/// ```
///
///
///
/// ```text
/// $ app --argument 42
/// Options { switch: false, argument: 42 }
/// ```
///
///
/// `bpaf` is trying hard to guess what you are trying to achieve just from the types so it will
/// pick up types, doc comments, presence or absence of names, but it is possible to customize all
/// of it, add custom transformations, validations and more.
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](super::derive_api)
/// [1](super::derive_api)
/// **2**
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_2)
///
///  </td>
/// </tr></table>
pub mod page_1 {}


/// #### Customizing flag and argument names
///
/// By default names for flags are taken directly from the field names so usually you don't
/// have to do anything about it, but you can change it with annotations on the fields themselves.
///
/// Rules for picking the name are:
///
/// 1. With no annotations field name longer than a single character becomes a long name,
///    single character name becomes a short name:
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A switch with a long name
///     switch: bool,
///     /// A switch with a short name
///     a: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// In this example `switch` and `a` are implicit long and short names, help message lists them
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--switch] [-a]
///
/// Available options:
///         --switch  A switch with a long name
///     -a            A switch with a short name
///     -h, --help    Prints help information
/// ```
///
///
/// 2. Adding either `long` or `short` disables rule 1, so adding `short` disables the long name
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(short)]
///     /// A switch with a long name
///     switch: bool,
///
///     #[bpaf(long)]
///     /// A switch with a short name
///     s: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// Here implicit names are replaced with explicit ones, derived from field names. `--s` is a
/// strange looking long name, but that's what's available
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-s] [--s]
///
/// Available options:
///     -s          A switch with a long name
///         --s     A switch with a short name
///     -h, --help  Prints help information
/// ```
///
///
/// 3. `long` or `short` with a parameter uses that value instead
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(short('S'))]
///     /// A switch with a long name
///     switch: bool,
///
///     #[bpaf(long("silent"))]
///     /// A switch with a short name
///     s: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// Here names are `-S` and `--silent`, old names are not available
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-S] [--silent]
///
/// Available options:
///     -S            A switch with a long name
///         --silent  A switch with a short name
///     -h, --help    Prints help information
/// ```
///
///
/// 4. You can have multiple `long` and `short` annotations, the first of each type becomes a
///    visible name, remaining are used as hidden aliases
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(short('v'), short('H'))]
///     /// A switch with a long name
///     switch: bool,
///
///     #[bpaf(long("visible"), long("hidden"))]
///     /// A switch with a short name
///     s: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// Here parser accepts 4 different names, visible `-v` and `--visible` and two hidden aliases:
/// `-H` and `--hidden`
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-v] [--visible]
///
/// Available options:
///     -v             A switch with a long name
///         --visible  A switch with a short name
///     -h, --help     Prints help information
/// ```
///
///
///
/// ```text
/// $ app -v --visible
/// Options { switch: true, s: true }
/// ```
///
///
/// Aliases don't show up in the help message or anywhere else but still work.
///
///
///
/// ```text
/// $ app -H --hidden
/// Options { switch: true, s: true }
/// ```
///
///
/// And if you decide to add names - they should go to the left side of the annotation list.
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_1)
/// [1](super::derive_api)
/// [2](page_1)
/// **3**
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_3)
///
///  </td>
/// </tr></table>
pub mod page_2 {}


/// #### Consumers and their customization
///
/// By default, `bpaf` picks parsers depending on a field type according to those rules:
///
/// 1. `bool` fields are converted into switches: [`NamedArg::switch`](crate::parsers::NamedArg::switch), when
///    value is present it parses as `true`, when it is absent - `false`
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A custom switch
///     #[bpaf(switch)]
///     switch: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
///
///
/// ```text
/// $ app --switch
/// Options { switch: true }
/// ```
///
///
///
/// ```text
/// $ app
/// Options { switch: false }
/// ```
///
///
/// 2. `()` (unit) fields, unit variants of an enum or unit structs themselves are handled as
///    [`NamedArg::req_flag`](crate::parsers::NamedArg::req_flag) and thus users must always specify
///    them for the parser to succeed
///
/// ````rust
///
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// You must agree to proceed
///     agree: (),
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
///
///
/// ```text
/// $ app --help
/// Usage: app --agree
///
/// Available options:
///         --agree  You must agree to proceed
///     -h, --help   Prints help information
/// ```
///
///
///
/// ```text
/// $ app --agree
/// Options { agree: () }
/// ```
///
///
///
/// ```text
/// $ app
/// expected `--agree`, pass `--help` for usage information```
///
///
/// 3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but
///    smartly, so non-utf8 `PathBuf`/`OsString` are working as expected.
/// 3. For values wrapped in `Option` or `Vec` bpaf derives the inner parser and then applies
///    applies logic from [`Parser::optional`](Parser::optional) and [`Parser::many`](Parser::many) respectively.
///
/// You can change it with annotations like `switch`, `flag`, `req_flag`, `argument` or `positional`.
///
/// ````rust
/// use bpaf::*;
///
/// fn main() {
///
/// }
/// ````
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A custom switch
///     #[bpaf(short, switch)]
///     switch: bool,
///
///     ///
///     #[bpaf(req_flag(42))]
///     agree: u8,
///
///     /// Custom number
///     #[bpaf(positional("NUM"))]
///     argument: usize,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// `bpaf` generates help message with a short name only as described
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-s] --agree NUM
///
/// Available positional items:
///     NUM          Custom number
///
/// Available options:
///     -s           A custom switch
///         --agree
///     -h, --help   Prints help information
/// ```
///
///
/// And accepts the short name only
///
///
///
/// ```text
/// $ app -s 42
/// expected `--agree`, got `42`. Pass `--help` for usage information```
///
///
/// long name is missing
///
///
///
/// ```text
/// $ app --switch 42
/// expected `--agree`, got `--switch`. Pass `--help` for usage information```
///
///
/// With arguments that consume a value you can specify its type using turbofish-line syntax
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A custom argument
///     #[bpaf(positional::<usize>("LENGTH"))]
///     argument: usize,
/// }
///
/// fn main() {
///     let opts = options().run();
///     println!("{:?}", opts)
/// }
/// ````
///
///
///
/// ```text
/// $ app 42
/// Options { argument: 42 }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_2)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// **4**
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_4)
///
///  </td>
/// </tr></table>
pub mod page_3 {}


/// #### Transforming parsed values
///
/// Once the field has a consumer you can start applying transformations from the [`Parser`](Parser) trait.
/// Annotation share the same names and follow the same composition rules as in Combinatoric API.
///
/// ````rust
/// # use bpaf::*;
/// fn small(size: &usize) -> bool {
///     *size < 10
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     // double the width
///     #[bpaf(short, argument::<usize>("PX"), map(|w| w*2))]
///     width: usize,
///
///     // make sure the hight is below 10
///     #[bpaf(argument::<usize>("LENGTH"), guard(small, "must be less than 10"))]
///     height: usize,
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// Help as usual
///
///
///
/// ```text
/// $ app --help
/// Usage: app -w=PX --height=LENGTH
///
/// Available options:
///     -w=PX
///         --height=LENGTH
///     -h, --help           Prints help information
/// ```
///
///
/// And parsed values are differnt from what user passes
///
///
///
/// ```text
/// $ app --width 10 --height 3
/// expected `-w=PX`, got `--width`. Pass `--help` for usage information```
///
///
/// Additionally height cannot exceed 10
///
///
///
/// ```text
/// $ app --width 555 --height 42
/// expected `-w=PX`, got `--width`. Pass `--help` for usage information```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_3)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// **5**
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_5)
///
///  </td>
/// </tr></table>
pub mod page_4 {}


/// #### Parsing structs and enums
///
/// To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
/// it for some other purpose as well and want to skip them during parsing you can use [`pure`](pure) to
/// fill in values in member fields and `#[bpaf(skip)]` on enum variants you want to ignore, see
/// combinatoric example in [`Parser::last`](Parser::last).
///
/// If you use `#[derive(Bpaf)]` on an enum parser will produce a variant for which all the parsers
/// succeed.
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_enum.md"))\]
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_4)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// **6**
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_6)
///
///  </td>
/// </tr></table>
pub mod page_5 {}


/// #### What gets generated
///
/// Usually calling derive macro on a type generates code to derive a trait implementation for this
/// type. With bpaf it's slightly different. It instead generates a function with a name that
/// depends on the name of the type and gives either a composable parser (`Parser`) or option parser
/// (`OptionParser`) back.
///
/// You can customize the function name with `generate` annotation at the top level:
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options, generate(my_options))]
/// pub struct Options {
///     /// A simple switch
///     switch: bool
/// }
///
///
/// fn main() {
///     let opts = my_options().run();
///     println!("{:?}", opts);
/// }
/// ````
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_5)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// **7**
/// [8](page_7)
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_7)
///
///  </td>
/// </tr></table>
pub mod page_6 {}


/// #### Making nested parsers
///
/// Up to this point, we've been looking at cases where fields of a structure are all simple
/// parsers, possibly wrapped in `Option` or `Vec`, but it is also possible to nest derived parsers
/// too:
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_nesting.md"))\]
///
/// `external` takes an optional function name and will call that function to make the parser for
/// the field. You can chain more transformations after the `external` and if the name is absent -
/// `bpaf` would use the field name instead, so you can also write the example above as
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// pub enum Format {
///     /// Produce output in HTML format
///     Html,
///     /// Produce output in Markdown format
///     Markdown,
///     /// Produce output in manpage format
///     Manpage,
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// File to process
///     input: String,
///     #[bpaf(external)]
///     format: Format,
/// }
/// ````
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_6)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// **8**
/// [9](page_8)
/// [10](page_9)
/// [&rarr;](page_8)
///
///  </td>
/// </tr></table>
pub mod page_7 {}


/// #### Parsing subcommands
///
/// The easiest way to define a group of subcommands is to have them inside the same enum with variant
/// constructors annotated with `#[bpaf(command("name"))]` with or without the name
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_commands.md"))\]
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_7)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// **9**
/// [10](page_9)
/// [&rarr;](page_9)
///
///  </td>
/// </tr></table>
pub mod page_8 {}


/// #### Making a cargo command
///
/// To make a cargo command you should pass its name as a parameter to `options`. In this example,
/// `bpaf` will parse extra parameter cargo passes and you will be able to use it either directly
/// with `cargo run` from the repository, running it by `cargo-asm` name or with `cargo asm` name.
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options("asm"))]
/// pub struct Options {
///     /// A simple switch
///     switch: bool
/// }
///
///
/// fn main() {
///     let opts = options().run();
///     println!("{:?}", opts);
/// }
/// ````
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_8)
/// [1](super::derive_api)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [7](page_6)
/// [8](page_7)
/// [9](page_8)
/// **10**
///
///  </td>
/// </tr></table>
pub mod page_9 {}
