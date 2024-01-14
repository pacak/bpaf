//! #### How to parse less frequent combinations
//!
//! While the design of `bpaf` tries to cover the most common use cases, mostly posix conventions,
//! it can also handle some more unusual requirements. It might come at the cost of having to write
//! more code, sometimes more confusing error messages or worse performance, but it will get the
//! job done.
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


/// #### Implementing `find(1)`: capture everything between `-exec` and `;`
///
/// Full example for `find(1)` is available in examples folder at bpaf's github.
///
/// It is possibe to capture everything between two tokens by using a combination of [`literal`](literal),
/// [`SimpleParser::anywhere`](SimpleParser::anywhere) and [`SimpleParser::adjacent`](SimpleParser::adjacent).
///
/// Since subsequence starts from an unusual item - `-exec` parser starts with `literal` and
/// `anywhere` that looks for exact match among unparsed items ignoring special meaning for dashes.
/// This is gets saved as the `tag` parser. To parse similar combination but starting with `--exec`
/// it is easier to use something like `long("exec").help("xxx").req_flag(())`. `tag` parser
/// produces `()` since we don't really care about the value it returns, only about the fact if it
/// succeeds.
///
/// Next building block is `item` parser. It consumes anything except for `;` so it uses `any`. To
/// fully support non-utf8 file names it parsers `OsString` and collects as many items as possible
/// into a vector `Vec<OsString>`.
///
/// Last building block takes care about trailing `;` so parser uses `literal` again.
///
/// Once building primitives are constructed parser combines them with [`construct!`](construct!) and
/// `adjacent`, extracts parsed items and makes the whole combination
/// [`optional`](Parser::optional) to handle cases where `-exec` is not present, same as `find(1)`
/// does it.
///
/// To make final option parser - parser for `-exec` should go first or at least before any other
/// parsers that might try to capture items it needs.
///
/// ````rust
/// # use std::ffi::OsString;
/// # use bpaf::*;
/// // parsers -exec xxx yyy zzz ;
/// fn exec() -> impl Parser<Option<Vec<OsString>>> {
///     let tag = literal("-exec", ())
///         .help("for every file find finds execute a separate shell command")
///         .anywhere();
///
///     let item = any::<OsString, _, _>("ITEM", |s| (s != ";").then_some(s))
///         .help("command with its arguments, find will replace {} with a file name")
///         .many();
///
///     let endtag = literal(";", ())
///         .help("anything after literal \";\" will be considered a regular option again");
///
///     construct!(tag, item, endtag)
///         .adjacent()
///         .map(|triple| triple.1)
///         .optional()
/// }
///
/// #[derive(Debug, Clone)]
/// # pub
/// struct Options {
///     exec: Option<Vec<OsString>>,
///     flag: bool,
/// }
///
/// # pub
/// fn options() -> OptionParser<Options> {
///     let flag = short('f').long("flag").help("Custom flag").switch();
///     construct!(Options { exec(), flag }).to_options()
/// }
///
/// fn main() {
///     println!("{:#?}", options().run());
/// }
/// ````
///
/// Resulting parser gets a `--help` message with individual items of the `-exec` parser coming in
/// a separate block
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-exec [ITEM]... ;] [-f]
///
/// Available options:
///   -exec [ITEM]... ;
///     -exec       for every file find finds execute a separate shell command
///     ITEM        command with its arguments, find will replace {} with a file name
///     ;           anything after literal ";" will be considered a regular option again
///
///     -f, --flag  Custom flag
///     -h, --help  Prints help information
/// ```
///
///
/// As expected everything between `-exec` and `;` is captured inside `exec` field and usual items
/// are parsed both before and after the `-exec` group.
///
///
///
/// ```text
/// $ app --flag -exec --hello {} ;
/// Options { exec: Some(["--hello", "{}"]), flag: true }
/// ```
///
///
///
/// ```text
/// $ app -exec --hello {} ; --flag
/// Options { exec: Some(["--hello", "{}"]), flag: true }
/// ```
///
///
/// And since `-exec` parser runs first - it captures anything that goes inside
///
///
///
/// ```text
/// $ app -exec --flag --hello {} ;
/// Options { exec: Some(["--flag", "--hello", "{}"]), flag: false }
/// ```
///
///
///
/// ```text
/// $ app --flag
/// Options { exec: None, flag: true }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](super::cookbook)
/// [1](super::cookbook)
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


/// #### Implementing `find(1)`: parsing permissions `-mode -rx` or `-mode /r`
///
/// Full example for `find(1)` is available in examples folder at bpaf's github.
///
/// `find(1)` program accepts more variations, This parser deals with parsing a subset of
/// permission string: flag `-perm` followed by a set of permission symbols prefixed with `-` or
/// `/`. To achieve that parser uses a combination of [`literal`](literal), [`SimpleParser::anywhere`](SimpleParser::anywhere) and
/// [`SimpleParser::adjacent`](SimpleParser::adjacent).
///
/// Flag starts with an unusual name - `-mode` so parser starts with `literal` and `anywhere`.
///
/// Next building block is the `mode` parser. Since mode can start with `-` parser uses [`any`](any) to
/// consume that. `any` consumes the whole string unconditionally using `Some` as a matching
/// function because this parser runs only after `-mode` is located.
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Default)]
/// # pub
/// struct Perms {
///     read: bool,
///     write: bool,
///     exec: bool,
/// }
///
/// #[derive(Debug, Clone)]
/// # pub
/// enum Perm {
///     All(Perms),
///     Any(Perms),
///     Exact(Perms),
/// }
///
/// #[derive(Debug, Clone)]
/// # pub
/// struct Options {
///     perm: Option<Perm>,
///     flag: bool,
/// }
///
/// /// parses symbolic permissions `-perm -mode`, `-perm /mode` and `-perm mode`
/// fn perm() -> impl Parser<Option<Perm>> {
///     fn parse_mode(input: &str) -> Result<Perms, String> {
///         let mut perms = Perms::default();
///         for c in input.chars() {
///             match c {
///                 'r' => perms.read = true,
///                 'w' => perms.write = true,
///                 'x' => perms.exec = true,
///                 _ => return Err(format!("{} is not a valid permission string", input)),
///             }
///         }
///         Ok(perms)
///     }
///
///     let tag = literal("-mode", ()).anywhere();
///
///     // `any` here is used to parse an arbitrary string that can also start with dash (-)
///     // regular positional parser won't work here
///     let mode = any("MODE", Some)
///         .help("(perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw")
///         .parse::<_, _, String>(|s: String| {
///             if let Some(m) = s.strip_prefix('-') {
///                 Ok(Perm::All(parse_mode(m)?))
///             } else if let Some(m) = s.strip_prefix('/') {
///                 Ok(Perm::Any(parse_mode(m)?))
///             } else {
///                 Ok(Perm::Exact(parse_mode(&s)?))
///             }
///         });
///
///     construct!(tag, mode)
///         .adjacent()
///         .map(|pair| pair.1)
///         .optional()
/// }
///
/// # pub
/// fn options() -> OptionParser<Options> {
///     let flag = short('f').long("flag").help("Custom flag").switch();
///     construct!(Options { perm(), flag }).to_options()
/// }
///
/// fn main() {
///     println!("{:#?}", options().run());
/// }
/// ````
///
/// Generated help message contains the details of the `-mode` flag in a separate block
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-mode MODE] [-f]
///
/// Available options:
///   -mode MODE
///     MODE        (perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw
///
///     -f, --flag  Custom flag
///     -h, --help  Prints help information
/// ```
///
///
/// And it can be used alongside other parsers.
///
///
///
/// ```text
/// $ app --flag -mode /rwx
/// Options { perm: Some(Any(Perms { read: true, write: true, exec: true })), flag: true }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_02)
/// [1](super::cookbook)
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


/// #### Implementing `dd(1)`: parsing named parameters in `key=val` form
///
/// Example is available in examples folder at bpaf's github.
///
/// This example parses syntax similar to `dd(1)` command. The main idea is to implement something
/// to make it simple to make parsers for `PREFIX=SUFFIX`, where prefix is fixed for each parser -
/// for example `if=` or `of=` and suffix is parsed with usual `FromStr` trait.
///
/// The function `tag` serves this purpose. It performs the following steps:
///
/// * consume any item that starts with a prefix at any argument position with [`any`](any) and
///   [`SimpleParser::anywhere`](SimpleParser::anywhere)
/// * attaches help message and custom metadata to make `--help` friendlier
/// * parses suffix with [`Parser::parse`](Parser::parse)
///
/// The rest of the parser simply uses tag to parse a few of dd arguments
///
/// ````rust
/// use bpaf::{*, doc::Style};
/// use std::str::FromStr;
///
/// #[derive(Debug, Clone)]
/// pub struct Options {
///     magic: bool,
///     in_file: String,
///     out_file: String,
///     block_size: usize,
/// }
///
/// /// Parses a string that starts with `name`, returns the suffix parsed in a usual way
/// fn tag<T>(name: &'static str, meta: &str, help: &'static str) -> impl Parser<T>
/// where
///     T: FromStr,
///     <T as FromStr>::Err: std::fmt::Display,
/// {
///     // it is possible to parse OsString here and strip the prefix with
///     // `os_str_bytes` or a similar crate
///     any("", move |s: String| Some(s.strip_prefix(name)?.to_owned()))
///         // this composes a metavar from two parts - literal and metavariable
///         // help message displays them in different colors
///         .metavar(&[(name, Style::Literal), (meta, Style::Metavar)][..])
///         // if you don't want to use colors you can replace previous line with this:
///         // .metavar(format!("{name}{meta}"))
///         .help(help)
///         .anywhere()
///         .parse(|s| s.parse())
/// }
///
/// fn in_file() -> impl Parser<String> {
///     tag::<String>("if=", "FILE", "read from FILE")
///         .fallback(String::from("-"))
///         .display_fallback()
/// }
///
/// fn out_file() -> impl Parser<String> {
///     tag::<String>("of=", "FILE", "write to FILE")
///         .fallback(String::from("-"))
///         .display_fallback()
/// }
///
/// fn block_size() -> impl Parser<usize> {
///     // it is possible to parse notation used by dd itself as well,
///     // using usuze only for simplicity
///     tag::<usize>("bs=", "SIZE", "read/write SIZE blocks at once")
///         .fallback(512)
///         .display_fallback()
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let magic = short('m')
///         .long("magic")
///         .help("a usual switch still works")
///         .switch();
///     construct!(Options {
///         magic,
///         in_file(),
///         out_file(),
///         block_size(),
///     })
///     .to_options()
/// }
///
/// fn main() {
///     println!("{:#?}", options().run());
/// }
/// ````
///
/// Generated help lists all the fields
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-m] [if=FILE] [of=FILE] [bs=SIZE]
///
/// Available options:
///     -m, --magic  a usual switch still works
///     if=FILE      read from FILE
///                  [default: -]
///     of=FILE      write to FILE
///                  [default: -]
///     bs=SIZE      read/write SIZE blocks at once
///                  [default: 512]
///     -h, --help   Prints help information
/// ```
///
///
/// Parser can handle expected input
///
///
///
/// ```text
/// $ app if=/dev/zero of=/tmp/blob bs=1024
/// Options { magic: false, in_file: "/dev/zero", out_file: "/tmp/blob", block_size: 1024 }
/// ```
///
///
/// And produces a reasonable error message for unsupported input
///
///
///
/// ```text
/// $ app if=/dev/zero of=/tmp/blob bs=1024 unsupported=false
/// Error: `unsupported=false` is not expected in this context
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_03)
/// [1](super::cookbook)
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


/// #### Implementing `Xorg(1)`: parsing `+xinerama` and `-xinerama` into a `bool`
///
/// A full example is available in examples folder at bpaf's github.
///
/// This example implements a parser for a named argument that starts with either `+` or `-` and
/// gets parsed into a `bool`. As with anything unusual parser utilizes [`any`](any) function to check
/// if input starts with `+` or `-`, and if so - checks if it matches predefined name, then producing
/// `true` or `false` depending on the first character. This logic is placed in `toggle` function.
///
/// Since custom items that start with a `-` can be interpreted as a set of short flags - it's a
/// good idea to place parsers created by `toggle` before regular parsers.
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone)]
/// pub struct Options {
///     turbo: bool,
///     backing: bool,
///     xinerama: bool,
/// }
///
/// // matches literal name prefixed with `-` or `+`.
/// // If name is not specified - parser fails with "not found" type of error.
/// fn toggle(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<bool> {
///     any(meta, move |s: String| {
///         if let Some(suf) = s.strip_prefix('+') {
///             (suf == name).then_some(true)
///         } else if let Some(suf) = s.strip_prefix('-') {
///             (suf == name).then_some(false)
///         } else {
///             None
///         }
///     })
///     .help(help)
///     .anywhere()
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let backing = toggle("(+|-)backing", "backing", "Set backing status")
///         .fallback(false)
///         .display_fallback();
///     let xinerama = toggle("(+|-)xinerama", "xinerama", "Set Xinerama status")
///         .fallback(true)
///         .display_fallback();
///     let turbo = short('t')
///         .long("turbo")
///         .help("Engage the turbo mode")
///         .switch();
///     construct!(Options {
///         backing,
///         xinerama,
///         turbo,
///     })
///     .to_options()
/// }
///
/// fn main() {
///     println!("{:#?}", options().run());
/// }
/// ````
///
/// Help message lists all the custom items
///
///
///
/// ```text
/// $ app --help
/// Usage: app [(+|-)backing] [(+|-)xinerama] [-t]
///
/// Available options:
///     (+|-)backing   Set backing status
///                    [default: false]
///     (+|-)xinerama  Set Xinerama status
///                    [default: true]
///     -t, --turbo    Engage the turbo mode
///     -h, --help     Prints help information
/// ```
///
///
/// You can use custom parsers alongside with regular parsers
///
///
///
/// ```text
/// $ app -xinerama --turbo +backing
/// Options { turbo: true, backing: true, xinerama: false }
/// ```
///
///
/// And default values for toggle parsers are set according to fallback values.
///
///
///
/// ```text
/// $ app
/// Options { turbo: false, backing: false, xinerama: true }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_04)
/// [1](super::cookbook)
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


/// #### Implementing `Xorg(1)`: parsing `+ext name` and `-ext name` into a `(String, bool)`
///
/// A full example is available in examples folder at bpaf's github.
///
/// This example parses a literal `+ext` or `-ext` followed by an arbitrary extension name a pair
/// containing extension name and status. As with anything unusal parser utilizes [`any`](any) with
/// [`SimpleParser::anywhere`](SimpleParser::anywhere) to match initial `+ext` and `-ext`, alternative approach is going to
/// be using a combination of two [`literal`](literal) functions. Once the tag is parsed - string that
/// follows it is parsed with [`adjacent`](crate::SimpleParser::adjacent) restriction.
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone)]
/// #[allow(dead_code)]
/// pub struct Options {
///     turbo: bool,
///     extensions: Vec<(String, bool)>,
/// }
///
/// // matches literal +ext and -ext followed by an extension name
/// fn extension() -> impl Parser<(String, bool)> {
///     let state = any("(+|-)ext", |s: String| match s.as_str() {
///         "-ext" => Some(false),
///         "+ext" => Some(true),
///         _ => None,
///     })
///     .anywhere();
///
///     let name = positional::<String>("EXT")
///         .help("Extension to enable or disable, see documentation for the full list");
///     construct!(state, name).adjacent().map(|(a, b)| (b, a))
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let turbo = short('t')
///         .long("turbo")
///         .help("Engage the turbo mode")
///         .switch();
///     let extensions = extension().many();
///     construct!(Options {
///         extensions,
///         turbo,
///     })
///     .to_options()
/// }
///
/// fn main() {
///     println!("{:#?}", options().run());
/// }
/// ````
///
///
///
/// ```text
/// $ app --help
/// Usage: app [(+|-)ext EXT]... [-t]
///
/// Available options:
///   (+|-)ext EXT
///     EXT          Extension to enable or disable, see documentation for the full list
///
///     -t, --turbo  Engage the turbo mode
///     -h, --help   Prints help information
/// ```
///
///
///
/// ```text
/// $ app +ext banana -t -ext apple
/// Options { turbo: true, extensions: [("banana", true), ("apple", false)] }
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_05)
/// [1](super::cookbook)
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


/// #### Command chaining: parsing `setup.py sdist bdist`
///
/// By default subcommand parser must consume all the items until the end of the line, but with
/// [`SimpleParser::adjacent`](SimpleParser::adjacent) restriction it can to parse only as much as it needs. Values must be
/// adjacent to the command name from the right side. When parser succeeds leftovers will be passed
/// to subsequent parsers.
///
/// Here `arg1` and `arg2` are adjacent to the `command` on the right side, while `prefix` is not.
///
/// ````console
/// prefix command arg1 arg2
/// ````
///
/// ##### Combinatoric example
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone)]
/// pub struct Options {
///     premium: bool,
///     commands: Vec<Cmd>,
/// }
///
/// #[derive(Debug, Clone)]
/// // shape of the variants doesn't really matter, let's use all of them :)
/// enum Cmd {
///     Eat(String),
///     Drink { coffee: bool },
///     Sleep { time: usize },
/// }
///
/// fn cmd() -> impl Parser<Cmd> {
///     // eat DISH
///     let eat = positional::<String>("FOOD")
///         .to_options()
///         .descr("Performs eating action")
///         .command("eat")
///         .adjacent()
///         .map(Cmd::Eat);
///
///     // drink [--coffee]
///     let coffee = long("coffee")
///         .help("Are you going to drink coffee?")
///         .switch();
///     let drink = construct!(Cmd::Drink { coffee })
///         .to_options()
///         .descr("Performs drinking action")
///         .command("drink")
///         .adjacent();
///
///     // sleep --time DURATION
///     let time = long("time").argument::<usize>("HOURS");
///     let sleep = construct!(Cmd::Sleep { time })
///         .to_options()
///         .descr("Performs taking a nap action")
///         .command("sleep")
///         .adjacent();
///
///     construct!([eat, drink, sleep])
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let premium = short('p')
///         .long("premium")
///         .help("Opt in for premium serivces")
///         .switch();
///     let commands = cmd().many();
///     // you can still combine with regular parsers, here - premium
///     construct!(Options { premium, commands }).to_options()
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// ##### Derive example
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(short, long)]
///     /// Opt in for premium serivces
///     pub premium: bool,
///     #[bpaf(external(cmd), many)]
///     pub commands: Vec<Cmd>,
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// pub enum Cmd {
///     #[bpaf(command, adjacent)]
///     /// Performs eating action
///     Eat(#[bpaf(positional("FOOD"))] String),
///     #[bpaf(command, adjacent)]
///     /// Performs drinking action
///     Drink {
///         /// Are you going to drink coffee?
///         coffee: bool,
///     },
///     #[bpaf(command, adjacent)]
///     /// Performs taking a nap action
///     Sleep {
///         #[bpaf(argument("HOURS"))]
///         time: usize,
///     },
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// Both examples implement a parser that supports one of three possible commands:
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-p] [COMMAND ...]...
///
/// Available options:
///     -p, --premium  Opt in for premium serivces
///     -h, --help     Prints help information
///
/// Available commands:
///     eat            Performs eating action
///     drink          Performs drinking action
///     sleep          Performs taking a nap action
/// ```
///
///
/// As usual every command comes with its own help
///
///
///
/// ```text
/// $ app drink --help
/// Performs drinking action
///
/// Usage: app drink [--coffee]
///
/// Available options:
///         --coffee  Are you going to drink coffee?
///     -h, --help    Prints help information
/// ```
///
///
/// Normally you can use one command at a time, but making commands adjacent lets parser to succeed
/// after consuming an adjacent block only and leaving leftovers for the rest of the parser,
/// consuming them as a `Vec<Cmd>` with many allows to chain multiple items sequentially
///
///
///
/// ```text
/// $ app eat Fastfood drink --coffee sleep --time=5
/// Options { premium: false, commands: [Eat("Fastfood"), Drink { coffee: true }, Sleep { time: 5 }] }
/// ```
///
///
/// The way this works is by running parsers for each command. In the first iteration eat succeeds,
/// it consumes eat fastfood portion and appends its value to the resulting vector. Then second
/// iteration runs on leftovers, in this case it will be `drink --coffee sleep --time=5`. Here `drink`
/// succeeds and consumes `drink --coffee` portion, then sleep parser runs, etc.
///
/// You can mix chained commands with regular arguments that belong to the top level parser
///
///
///
/// ```text
/// $ app sleep --time 10 --premium eat 'Bak Kut Teh' drink
/// Options { premium: true, commands: [Sleep { time: 10 }, Eat("Bak Kut Teh"), Drink { coffee: false }] }
/// ```
///
///
/// But not inside the command itself since values consumed by the command are not going to be adjacent:
///
///
///
/// ```text
/// $ app sleep --time 10 eat --premium 'Bak Kut Teh' drink
/// Error: expected `FOOD`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_06)
/// [1](super::cookbook)
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


/// #### Multi value arguments: `--point X Y Z`
///
/// By default arguments take at most one value, you can create multi value options by using
/// [`SimpleParser::adjacent`](SimpleParser::adjacent) modifier.
///
/// ````rust
/// use bpaf::*;
/// #[derive(Debug, Clone)]
/// pub struct Options {
///     point: Vec<Point>,
///     rotate: bool,
/// }
///
/// #[derive(Debug, Clone)]
/// struct Point {
///     point: (),
///     x: usize,
///     y: usize,
///     z: f64,
/// }
///
/// fn point() -> impl Parser<Point> {
///     let point = short('p')
///         .long("point")
///         .help("Point coordinates")
///         .req_flag(());
///     let x = positional::<usize>("X").help("X coordinate of a point");
///     let y = positional::<usize>("Y").help("Y coordinate of a point");
///     let z = positional::<f64>("Z").help("Height of a point above the plane");
///     construct!(Point { point, x, y, z }).adjacent()
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let rotate = short('r')
///         .long("rotate")
///         .help("Face the camera towards the first point")
///         .switch();
///     let point = point().many();
///     construct!(Options { point, rotate }).to_options()
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(external, many)]
///     point: Vec<Point>,
///     #[bpaf(short, long)]
///     /// Face the camera towards the first point
///     rotate: bool,
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(adjacent)]
/// struct Point {
///     #[bpaf(short, long)]
///     /// Point coordinates
///     point: (),
///     #[bpaf(positional("X"))]
///     /// X coordinate of a point
///     x: usize,
///     #[bpaf(positional("Y"))]
///     /// Y coordinate of a point
///     y: usize,
///     #[bpaf(positional("Z"))]
///     /// Height of a point above the plane
///     z: f64,
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// Fields can have different types, including Option or Vec, in this example they are two usize and one f64.
///
///
///
/// ```text
/// $ app --help
/// Usage: app [-p X Y Z]... [-r]
///
/// Available options:
///   -p X Y Z
///     -p, --point   Point coordinates
///     X             X coordinate of a point
///     Y             Y coordinate of a point
///     Z             Height of a point above the plane
///
///     -r, --rotate  Face the camera towards the first point
///     -h, --help    Prints help information
/// ```
///
///
/// Flag `--point` takes 3 positional arguments: two integers for X and Y coordinates and one
/// floating point for height, order is important, switch `--rotate` can go on either side of it
///
///
///
/// ```text
/// $ app --rotate --point 10 20 3.1415
/// Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }], rotate: true }
/// ```
///
///
/// Parser accepts multiple points as long as they don't interleave
///
///
///
/// ```text
/// $ app --point 10 20 3.1415 --point 1 2 0.0
/// Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }, Point { point: (), x: 1, y: 2, z: 0.0 }], rotate: false }
/// ```
///
///
/// `--rotate` can’t go in the middle of the point definition as the parser expects the second item
///
///
///
/// ```text
/// $ app --point 10 20 --rotate 3.1415
/// Error: expected `Z`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_07)
/// [1](super::cookbook)
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


/// #### Structure groups: parsing `--sensor --sensor-device NAME --sensor-value VAL`
///
/// With [`SimpleParser::adjacent`](SimpleParser::adjacent) you can restrict several parsers to be adjacent together,
/// starting from a simple flag parser allowing you to parse multiple structures where order inside
/// the structure doesn't matter, but presense of all the field matters.
///
/// ````console
/// $ prometheus_sensors_exporter \
///     \
///     `# 2 physical sensors located on physycial different i2c bus or address` \
///     --sensor \
///         --sensor-device=tmp102 \
///         --sensor-name="temperature_tmp102_outdoor" \
///         --sensor-i2c-bus=0 \
///         --sensor-i2c-address=0x48 \
///     --sensor \
///         --sensor-device=tmp102 \
///         --sensor-name="temperature_tmp102_indoor" \
///         --sensor-i2c-bus=1 \
///         --sensor-i2c-address=0x49 \
///     ...
/// ````
///
/// ##### Combinatoric example
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone)]
/// pub struct Options {
///     rect: Vec<Rect>,
///     mirror: bool,
/// }
///
/// #[derive(Debug, Clone)]
/// struct Rect {
///     rect: (),
///     width: usize,
///     height: usize,
///     painted: bool,
/// }
///
/// fn rect() -> impl Parser<Rect> {
///     let rect = long("rect").help("Define a new rectangle").req_flag(());
///     let width = short('w')
///         .long("width")
///         .help("Rectangle width in pixels")
///         .argument::<usize>("PX");
///     let height = short('h')
///         .long("height")
///         .help("Rectangle height in pixels")
///         .argument::<usize>("PX");
///     let painted = short('p')
///         .long("painted")
///         .help("Should rectangle be filled?")
///         .switch();
///     construct!(Rect {
///         rect,
///         width,
///         height,
///         painted,
///     })
///     .adjacent()
/// }
///
/// pub fn options() -> OptionParser<Options> {
///     let mirror = long("mirror").help("Mirror the image").switch();
///     let rect = rect().many();
///     construct!(Options { rect, mirror }).to_options()
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// ##### Derive example
///
/// ````rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)]
/// pub struct Options {
///     #[bpaf(external, many)]
///     rect: Vec<Rect>,
///     /// Mirror the image
///     mirror: bool,
/// }
///
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(adjacent)]
/// struct Rect {
///     /// Define a new rectangle
///     rect: (),
///     #[bpaf(short, long, argument("PX"))]
///     /// Rectangle width in pixels
///     width: usize,
///     #[bpaf(short, long, argument("PX"))]
///     /// Rectangle height in pixels
///     height: usize,
///     #[bpaf(short, long)]
///     /// Should rectangle be filled?
///     painted: bool,
/// }
///
/// fn main() {
///     println!("{:?}", options().run())
/// }
/// ````
///
/// This example parses multipe rectangles from a command line defined by dimensions and the fact
/// if its filled or not, to make things more interesting - every group of coordinates must be
/// prefixed with `--rect`
///
///
///
/// ```text
/// $ app --help
/// Usage: app [--rect -w=PX -h=PX [-p]]... [--mirror]
///
/// Available options:
///   --rect -w=PX -h=PX [-p]
///         --rect       Define a new rectangle
///     -w, --width=PX   Rectangle width in pixels
///     -h, --height=PX  Rectangle height in pixels
///     -p, --painted    Should rectangle be filled?
///
///         --mirror     Mirror the image
///     -h, --help       Prints help information
/// ```
///
///
/// Order of items within the rectangle is not significant and you can have several of them,
/// because fields are still regular arguments - order doesn’t matter for as long as they belong to
/// some rectangle
///
///
///
/// ```text
/// $ app --rect --width 10 --height 10 --rect --height=10 --width=10
/// Options { rect: [Rect { rect: (), width: 10, height: 10, painted: false }, Rect { rect: (), width: 10, height: 10, painted: false }], mirror: false }
/// ```
///
///
/// You can have optional values that belong to the group inside and outer flags in the middle
///
///
///
/// ```text
/// $ app --rect --width 10 --painted --height 10 --mirror --rect --height 10 --width 10
/// Options { rect: [Rect { rect: (), width: 10, height: 10, painted: true }, Rect { rect: (), width: 10, height: 10, painted: false }], mirror: true }
/// ```
///
///
/// But with adjacent they cannot interleave
///
///
///
/// ```text
/// $ app --rect --rect --width 10 --painted --height 10 --height 10 --width 10
/// Error: expected `--width=PX`, pass `--help` for usage information
/// ```
///
///
/// Or have items that don’t belong to the group inside them
///
///
///
/// ```text
/// $ app --rect --width 10 --mirror --painted --height 10 --rect --height 10 --width 10
/// Error: expected `--height=PX`, pass `--help` for usage information
/// ```
///
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_08)
/// [1](super::cookbook)
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


/// #### Skipping optional positional items with validation
///
/// Combinations like [`Parser::optional`](Parser::optional) and [`SimpleParser::catch`](SimpleParser::catch) allow to try to parse something and
/// then handle the error as if pase attempt never existed
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_09)
/// [1](super::cookbook)
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
