//! #### Types of arguments
//!
//! This chapter serves as an introduction to available command line options and tries to set the
//! terminology. If you are familiar with command line argument parsers in general - feel free to
//! skip it.
//!
//! If you ever used any software from a command line (say `cargo`) you used command line options.
//! Let's recap how you might run tests for a crate in your rust project:
//!
//! ````text
//! $ cargo test -p my_project --verbose
//! ````
//!
//! `cargo` here is an executable name, everything to the right of it separated by spaces are the
//! options.
//!
//! Nowadays programs share mostly similar conventions about what a command line argument is, it
//! wasn't the case before though. Let's cover the basic types.
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
//! [&rarr;](page_1)
//!
//!  </td>
//! </tr></table>
#[allow(unused_imports)] use crate::{*, parsers::*};


/// #### Options, switches or flags
///
/// Options or flags usually starts with a dash, a single dash for short options and a double dash for
/// long one. Several short options can usually be squashed together with a single dash in front of
/// them to save on typing: `-vvv` can be parsed the same as `-v -v -v`. Options don't have any
/// other information apart from being there or not. Relative position usually does not matter and
/// `--alpha --beta` should parse the same as `--beta --alpha`.
///
/// <div class="code-wrap">
/// <pre>
/// $ cargo <span style="font-weight: bold">--help</span>
/// $ ls <span style="font-weight: bold">-la</span>
/// $ ls <span style="font-weight: bold">--time --reverse</span>
/// </pre>
/// </div>
///
/// To parse one
///
/// For more detailed info see [`NamedArg::switch`](NamedArg::switch) and
/// [`NamedArg::flag`](NamedArg::flag)
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](super::types_of_arguments)
/// [1](super::types_of_arguments)
/// **2**
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [&rarr;](page_2)
///
///  </td>
/// </tr></table>
pub mod page_1 {}


/// #### Option arguments or arguments
///
/// Option arguments are similar to regular options but they come with an extra value attached.
/// Value can be separated by a space, `=` or directly adjacent to a short name. Same as with
/// options - their relative position usually doesn't matter.
///
/// <div class="code-wrap">
/// <pre>
/// $ cargo build <span style="font-weight: bold">--package bpaf</span>
/// $ cargo test <span style="font-weight: bold">-j2</span>
/// $ cargo check <span style="font-weight: bold">--bin=megapotato</span>
/// </pre>
/// </div>
///
/// In the generated help message or documentation they come with a placeholder metavariable,
/// usually a short, all-caps word describing what the value means: `NAME`, `AGE`, `SPEC`, and `CODE`
/// are all valid examples.
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/argument.md"))\]
///
/// For more detailed info see [`NamedArg::argument`](NamedArg::argument)
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_1)
/// [1](super::types_of_arguments)
/// [2](page_1)
/// **3**
/// [4](page_3)
/// [5](page_4)
/// [6](page_5)
/// [&rarr;](page_3)
///
///  </td>
/// </tr></table>
pub mod page_2 {}


/// #### Operands or positional items
///
/// Operands are usually items that are present on a command line and not prefixed by a short or
/// long name. They are usually used to represent the most important part of the operation:
/// `cat Cargo.toml` - display THIS file, `rm -rf target` - remove THIS folder and so on.
///
/// <div class="code-wrap">
/// <pre>
/// $ cat <span style="font-weight: bold">/etc/passwd</span>
/// $ rm -rf <span style="font-weight: bold">target</span>
/// $ man <span style="font-weight: bold">gcc</span>
/// </pre>
/// </div>
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))\]
///
/// For more detailed info see [`positional`](crate::positional) and
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_2)
/// [1](super::types_of_arguments)
/// [2](page_1)
/// [3](page_2)
/// **4**
/// [5](page_4)
/// [6](page_5)
/// [&rarr;](page_4)
///
///  </td>
/// </tr></table>
pub mod page_3 {}


/// #### Commands or subcommands
///
/// Commands are similar to positional items, but instead of representing an item they start
/// a whole new parser, usually with its help and other arguments. Commands allow a single
/// application to perform multiple different functions. The command parser will be able to parse all
/// the command line options to the right of the command name
///
/// <div class="code-wrap">
/// <pre>
/// $ cargo <span style="font-weight: bold">build --release</span>
/// $ cargo <span style="font-weight: bold">clippy</span>
/// $ cargo <span style="font-weight: bold">asm --intel --everything</span>
/// </pre>
/// </div>
///
/// \#![cfg_attr(not(doctest), doc = include_str!("docs2/command.md"))\]
///
/// For more detailed info see [`OptionParser::command`](OptionParser::command)
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_3)
/// [1](super::types_of_arguments)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// **5**
/// [6](page_5)
/// [&rarr;](page_5)
///
///  </td>
/// </tr></table>
pub mod page_4 {}


/// #### Exotic schemas
///
/// While modern software tends to use just the options listed above you can still encounter
/// programs created before those options became the norm and they use something completely different,
/// let me give a few examples, see [the parsing cookbook](crate::_documentation::_2_howto)
/// about actually parsing them
///
/// `su` takes an option that consists of a single dash `-`
///
/// <div class="code-wrap"><pre>
/// $ su <span style="font-weight: bold">-</span>
/// </pre></div>
///
/// `find` considers everything between `--exec` and `;` to be a single item.
/// this example calls `ls -l` on every file `find` finds.
///
/// <div class="code-wrap"><pre>
/// $ find /etc --exec ls -l '{}' \;
/// </pre></div>
///
/// `Xorg` and related tools use flag-like items that start with a single `+` to enable a
/// feature and with `-` to disable it.
///
/// <div class="code-wrap"><pre>
/// $ xorg -backing +xinerama
/// </pre></div>
///
/// `dd` takes several key-value pairs, this would create a 100M file
///
/// <div class="code-wrap"><pre>
/// $ dd if=/dev/zero of=dummy.bin bs=1M count=100
/// </pre></div>
///
/// Most of the command line arguments in Turbo C++ 3.0 start with `/`. For example, option
/// `/x` tells it to use all available extended memory, while `/x[=n]` limits it to n kilobytes
///
/// <div class="code-wrap"><pre>
/// C:\PROJECT>TC /x=200
/// </pre></div>
///
/// <table width='100%' cellspacing='0' style='border: hidden;'><tr>
///  <td style='text-align: center;'>
///
/// [&larr;](page_4)
/// [1](super::types_of_arguments)
/// [2](page_1)
/// [3](page_2)
/// [4](page_3)
/// [5](page_4)
/// **6**
///
///  </td>
/// </tr></table>
pub mod page_5 {}
