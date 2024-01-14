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
//! [2](page_02)
//! [3](page_03)
//! [4](page_04)
//! [5](page_05)
//! [6](page_06)
//! [7](page_07)
//! [8](page_08)
//! [9](page_09)
//! [10](page_10)
//! [&rarr;](page_02)
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
/// Error: expected `--argument=ARG`, pass `--help` for usage information
/// ```
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
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_03)
///
///  </td>
/// </tr></table>
pub mod page_02 {}


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
/// [&larr;](page_02)
/// [1](super::derive_api)
/// [2](page_02)
/// **3**
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_04)
///
///  </td>
/// </tr></table>
pub mod page_03 {}


/// #### Consumers and their customization
///
/// By default, `bpaf` picks parsers depending on a field type according to those rules:
///
/// 1. `bool` fields are converted into switches: [`SimpleParser::switch`](SimpleParser::switch), when value is present
///    it parses as `true`, when it is absent - `false`
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
///    [`SimpleParser::req_flag`](SimpleParser::req_flag) and thus users must always specify them for the parser to succeed
///
/// ````rust
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
/// Error: expected `--agree`, pass `--help` for usage information
/// ```
///
///
/// 3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but
///    smartly, so non-utf8 `PathBuf`/`OsString` are working as expected.
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// numeric argument
///     width: usize,
///     /// IPv4 address
///     addr: std::net::Ipv4Addr,
///     /// Path
///     path: std::path::PathBuf,
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
/// $ app --width 42 --addr 127.0.0.1 --path /etc/passwd
/// Options { width: 42, addr: 127.0.0.1, path: "/etc/passwd" }
/// ```
///
///
/// 4. For values wrapped in `Option` or `Vec` bpaf derives the inner parser and then applies
///    applies logic from [`Parser::optional`](Parser::optional) and [`Parser::many`](Parser::many) respectively. You can also
///    use `optional` and `many` annotation explicitly.
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// optional numeric argument
///     width: Option<usize>,
///     /// many IPv4 addresses
///     addr: Vec<std::net::Ipv4Addr>,
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
/// $ app --addr 127.0.0.1 --addr 10.0.1.254
/// Options { width: None, addr: [127.0.0.1, 10.0.1.254] }
/// ```
///
///
/// 5. Fields in tuple structures are converted into positional parsers
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options (
///     /// First positional
///     String,
///     /// second positional
///     usize
/// );
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
/// Usage: app ARG ARG
///
/// Available positional items:
///     ARG         First positional
///     ARG         second positional
///
/// Available options:
///     -h, --help  Prints help information
/// ```
///
///
///
/// ```text
/// $ app Bob 42
/// Options("Bob", 42)
/// ```
///
///
/// 6. You can change it with explicit annotations like `switch`, `flag`, `req_flag`, `argument` or
///    `positional`. `external` annotation allows you to nest results from a whole different
///    parser. `external` is somewhat special since it disables any logic that applies extra
///    transformations based on the type. For example if you have an optional `external` field -
///    you have to specify that it is `optional` manually.
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// A custom switch
///     #[bpaf(switch)]
///     switch: bool,
///
///     /// Explicit required flag
///     #[bpaf(req_flag(42))]
///     agree: u8,
///
///     /// Custom boolean switch with inverted values
///     #[bpaf(flag(false, true))]
///     inverted: bool,
///
///     /// Custom argument
///     #[bpaf(argument("DIST"))]
///     distance: f64,
///
///     /// external parser
///     #[bpaf(external, optional)]
///     rectangle: Option<Rectangle>,
///
///     /// Custom positional number
///     #[bpaf(positional("NUM"))]
///     argument: usize,
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// pub struct Rectangle {
///     /// Width of the rectangle
///     width: usize,
///     /// Height of the rectangle
///     height: usize,
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
/// Usage: app [--switch] --agree [--inverted] --distance=DIST [--width=ARG --height=ARG] NUM
///
/// Available positional items:
///     NUM                  Custom positional number
///
/// Available options:
///         --switch         A custom switch
///         --agree          Explicit required flag
///         --inverted       Custom boolean switch with inverted values
///         --distance=DIST  Custom argument
///         --width=ARG      Width of the rectangle
///         --height=ARG     Height of the rectangle
///     -h, --help           Prints help information
/// ```
///
///
///
/// ```text
/// $ app --switch --agree --inverted --distance 23 --width 20 --height 30 42
/// Options { switch: true, agree: 42, inverted: false, distance: 23.0, rectangle: Some(Rectangle { width: 20, height: 30 }), argument: 42 }
/// ```
///
///
/// With arguments that consume a value you can specify its type using turbofish syntax
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
/// [&larr;](page_03)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// **4**
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_05)
///
///  </td>
/// </tr></table>
pub mod page_04 {}


/// #### Transforming parsed values
///
/// Often specifying consumer is enough to parse a value, but in some cases you might want to apply
/// additional transformations or validations. for example some numeric parameter must be not only
/// valid `u8`, but also in range 1..100 inclusive or an IP address should belong to a certain
/// range. On the right side of the consumer you can list zero or more transformations from the
/// [`Parser`](Parser) trait. Annotation share the same names and follow the same composition rules as in
/// Combinatoric API.
///
/// ````rust
/// use bpaf::*;
/// fn small(size: &usize) -> bool {
///     *size < 10
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     // double the width
///     #[bpaf(argument::<usize>("PX"), map(|w| w*2))]
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
/// And parsed values are differnt from what user passes
///
///
///
/// ```text
/// $ app --width 10 --height 3
/// Options { width: 20, height: 3 }
/// ```
///
///
/// Additionally height cannot exceed 10
///
///
///
/// ```text
/// $ app --width 555 --height 42
/// Error: `42`: must be less than 10
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_04)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// **5**
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_06)
///
///  </td>
/// </tr></table>
pub mod page_05 {}


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
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     /// User name
///     user: String,
///     #[bpaf(pure(100))]
///     starting_money: usize,
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
/// Usage: app --user=ARG
///
/// Available options:
///         --user=ARG  User name
///     -h, --help      Prints help information
/// ```
///
///
/// `starting_money` is filled from [`pure`](pure) and there's no way for user to override it
///
///
///
/// ```text
/// $ app --user Bob
/// Options { user: "Bob", starting_money: 100 }
/// ```
///
///
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub enum Options {
///     ByPath {
///         path: std::path::PathBuf
///     },
///     ByName {
///         name: String,
///     },
///     #[bpaf(skip)]
///     Resolved {
///         id: usize,
///     }
/// }
///
/// fn main() {
///     println!("{:?}", options().run());
/// }
/// ````
///
/// `bpaf` ignores `Options::Resolved` constructor
///
///
///
/// ```text
/// $ app --help
/// Usage: app (--path=ARG | --name=ARG)
///
/// Available options:
///         --path=ARG
///         --name=ARG
///     -h, --help      Prints help information
/// ```
///
///
///
/// ```text
/// $ app --name hackerman
/// ByName { name: "hackerman" }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_05)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// **6**
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_07)
///
///  </td>
/// </tr></table>
pub mod page_06 {}


/// #### What gets generated
///
/// Usually calling derive macro on a type generates a trait implementation for this type. With
/// bpaf it's slightly different. It instead generates a function with a name that depends on the
/// name of the type and gives either a composable parser (`Parser`) or option parser
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
/// fn main() {
///     println!("{:?}", my_options().run());
/// }
/// # pub fn options() -> OptionParser<Options> { my_options() }
/// ````
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--switch]
///
/// Available options:
///         --switch  A simple switch
///     -h, --help    Prints help information
/// ```
///
///
/// By default function shares the same visibility as the structure, but you can make it module
/// private with `private` annotation:
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options, generate(my_options), private)]
/// pub struct Options {
///     /// A simple switch
///     switch: bool
/// }
///
/// fn main() {
///     println!("{:?}", my_options().run());
/// }
/// # pub fn options() -> OptionParser<Options> { my_options() }
/// ````
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--switch]
///
/// Available options:
///         --switch  A simple switch
///     -h, --help    Prints help information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_06)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// **7**
/// [8](page_08)
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_08)
///
///  </td>
/// </tr></table>
pub mod page_07 {}


/// #### Making nested parsers
///
/// Up to this point, we've been mostly looking at cases where fields of a structure are all simple
/// parsers, possibly wrapped in `Option` or `Vec`, but it is also possible to nest derived parsers
/// too:
///
/// ````rust
/// use bpaf::*;
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
///     #[bpaf(external(format))]
///     format: Format,
/// }
/// ````
///
/// Help message lists all possible options
///
///
///
/// ```text
/// $ app --help
/// Usage: app --input=ARG (--html | --markdown | --manpage)
///
/// Available options:
///         --input=ARG  File to process
///         --html       Produce output in HTML format
///         --markdown   Produce output in Markdown format
///         --manpage    Produce output in manpage format
///     -h, --help       Prints help information
/// ```
///
///
/// Parser accepts one and only one value from enum in this example
///
///
///
/// ```text
/// $ app --input Cargo.toml --html
/// Options { input: "Cargo.toml", format: Html }
/// ```
///
///
///
/// ```text
/// $ app --input hello
/// Error: expected `--html`, `--markdown`, or more, pass `--help` for usage information
/// ```
///
///
///
/// ```text
/// $ app --input hello --html --markdown
/// Error: `--markdown` cannot be used at the same time as `--html`
/// ```
///
///
/// `external` takes an optional function name and will call that function to make the parser for
/// the field. You can chain more transformations after the `external` and if the name is absent -
/// `bpaf` would use the field name instead.
///
/// Because of the limitations of the macro system having `external` parser disables automatic
/// detection for `Option` or `Vec` containers so you have to specify it explicitly:
///
/// ````rust
/// use bpaf::*;
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
///     #[bpaf(external(format), many)]
///     format: Vec<Format>,
/// }
/// ````
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_07)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// **8**
/// [9](page_09)
/// [10](page_10)
/// [&rarr;](page_09)
///
///  </td>
/// </tr></table>
pub mod page_08 {}


/// #### Parsing subcommands
///
/// The easiest way to define a group of subcommands is to have them inside the same enum with variant
/// constructors annotated with `#[bpaf(command("name"))]` with or without the name
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub enum Options {
///     #[bpaf(command("run"))]
///     /// Run a binary
///     Run {
///         /// Name of a binary crate
///         name: String,
///     },
///     /// Run a self test
///     #[bpaf(command)]
///     Test,
/// }
/// ````
///
/// Help message lists subcommand
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
///     test        Run a self test
/// ```
///
///
/// Commands have their own arguments and their own help
///
///
///
/// ```text
/// $ app run --help
/// Run a binary
///
/// Usage: app run --name=ARG
///
/// Available options:
///         --name=ARG  Name of a binary crate
///     -h, --help      Prints help information
/// ```
///
///
///
/// ```text
/// $ app run --name Bob
/// Run { name: "Bob" }
/// ```
///
///
///
/// ```text
/// $ app test
/// Test
/// ```
///
///
/// And even if `--name` is valid in scope of `run` command - it's not valid for `test`
///
///
///
/// ```text
/// $ app test --name bob
/// Error: `--name` is not expected in this context
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_08)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// **9**
/// [10](page_10)
/// [&rarr;](page_10)
///
///  </td>
/// </tr></table>
pub mod page_09 {}


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
/// [&larr;](page_09)
/// [1](super::derive_api)
/// [2](page_02)
/// [3](page_03)
/// [4](page_04)
/// [5](page_05)
/// [6](page_06)
/// [7](page_07)
/// [8](page_08)
/// [9](page_09)
/// **10**
///
///  </td>
/// </tr></table>
pub mod page_10 {}
