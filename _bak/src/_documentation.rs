//! #### Project documentation
//! 
//! See [official website](https://pacak.github.io/bpaf/bpaf/_documentation/index.html) for more up to date version.
//!
//! - [Introduction and design goals](_0_intro) - A quick intro. What, why and how
//! - [Tutorials](_1_tutorials) - practical, learning oriented guides
//! - [HOWTO - practical, oriented to solving problems guides](_2_howto)
//! - [Parsing cookbook](_3_cookbook) - How to parse less frequent combinations
//! - [Theory explanation](_4_explanation) - Theoretical information about abstractions used by the library, oriented for understanding
//!
    pub mod _0_intro {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Tutorials &rarr;](super::_1_tutorials)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### Introduction and design goals
        //! A quick intro. What, why and how
        //! 
        //! `bpaf` is a lightweight and flexible command line parser that uses both combinatoric and derive
        //! style API
        //! 
        //! Combinatoric API usually means a bit more typing but no dependency on proc macros and more help
        //! from the IDE, derive API uses proc macro to save on typing but your IDE will be less likely to
        //! help you. Picking one API style does not lock you out from using the other style, you can mix
        //! and match both in a single parser
        //! 
        //! # Examples of both styles
        //! 
        #![cfg_attr(not(doctest), doc = include_str!("docs2/intro.md"))]
        //! 
        //! # Design goals
        //! 
        //! ## Parse, don't validate
        //! 
        //! `bpaf` tries hard to let you move as many invariants about the user input you are
        //! trying to parse into rust types: for mutually exclusive options you can get `enum` with
        //! exclusive items going into separate branches, and you can collect results into types like
        //! [`BTreeSet`](std::collections::BTreeSet), or whatever custom type you might have with
        //! custom parsing. Ideas for
        //! [making invalid states unrepresentable](https://geeklaunch.io/blog/make-invalid-states-unrepresentable/)
        //! and [using parsing over validation](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
        //! are not new.
        //! 
        //! That said you can also validate your inputs if this fits your situation better. If you want to
        //! ensure that the sum of every numeric field must be divisible by both 3 and 5, but only when it's
        //! Thursday - you can do that too.
        //! 
        //! ## Flexibility
        //! 
        //! While aiming to be a general-purpose command line parser `bpaf` offers a few backdoors that
        //! allow you to parse pretty much anything you want: chained commands, custom blocks of options,
        //! DOS-style options (`/ofile.pas`), `dd` style options (`if=file of=out`), etc. A similar idea applies
        //! to what the parser can produce - if your app operates with boxed string slices internally - `bpaf`
        //! will give you `Box<str>` instead of `String` if you ask it to.
        //! 
        //! The only restriction is that you cannot use information from items parsed earlier (but not
        //! the fact that something was parsed successfully or not) to decide to how to parse further
        //! options, and even then you can side step this restriction by passing some shared state as a
        //! parameter to the parsers.
        //! 
        //! ## Reusability
        //! 
        //! Parsers in `bpaf` are not monolithic and you can share their parts across multiple binaries,
        //! workspace members or even independent projects. Say you have multiple binaries in a workspace
        //! that perform different operations on some input. You can declare a parser for the input
        //! specifically, along with all the validations, help messages or shell dynamic completion
        //! functions you need and use it across all the binaries alongside the arguments specific to
        //! those binaries.
        //! 
        //! ## Composition, transformation
        //! 
        //! Parsers in `bpaf` are not finalized either, say you have a parser that describes a single input
        //! for your program, it can take multiple arguments or perform extra validations, etc. You can
        //! always compose this parser with any other parser to produce tuples of both results for example.
        //! Or to make it so parser runs multiple times and collects results into a `Vec`.
        //! 
        //! ## Performance
        //! 
        //! While performance is an explicit non-goal - `bpaf` does nothing that would pessimize it either,
        //! so performance is on par or better compared to other fully featured parsers.
        //! 
        //! ## Correctness
        //! 
        //! `bpaf` would parse only items it can represent and will reject anything it cannot represent
        //! in the output. Say your parser accepts both `--intel` and `--att` flags, but encodes the result
        //! into `enum Style { Intel, Att }`, `bpaf` will accept those flags separately, but not if they
        //! are used both at once. If the parser later collects multiple styles into a `Vec<Style>` then it
        //! will accept any combinationof those flags.
        //! 
        //! ## User friendly
        //! 
        //! `bpaf` tries to provide user-friendly error messages, and suggestions for typos but also scripts
        //! for shell completion, `man` pages and markdown documentation for the web.
        //!
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Tutorials &rarr;](super::_1_tutorials)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
    use crate::*;
    }
    pub mod _1_tutorials {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Introduction and design goals](super::_0_intro)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [HOWTO - practical, oriented to solving problems guides &rarr;](super::_2_howto)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### Tutorials
        //! practical, learning oriented guides
        //!
        //! - [Types of arguments](_0_types_of_arguments) - common types of line options and conventions
        //! - [Combinatoric API](_1_combinatoric_api) - Parse arguments without using proc macros
        //! - [Derive API tutorial](_2_derive_api) - Create a parser by defining a structure
        //! - [Designing a good datatype](_3_picking_type) - bpaf allows you to reduce the size of legal values to valid ones
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Introduction and design goals](super::_0_intro)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [HOWTO - practical, oriented to solving problems guides &rarr;](super::_2_howto)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        pub mod _0_types_of_arguments {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Combinatoric API &rarr;](super::_1_combinatoric_api)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Types of arguments
            //! common types of line options and conventions
            //! 
            //! This chapter serves as an introduction to available command line options and tries to set the
            //! terminology. If you are familiar with command line argument parsers in general - feel free to
            //! skip it.
            //! 
            //! If you ever used any software from a command line (say `cargo`) you used command line options.
            //! Let's recap how you might run tests for a crate in your rust project:
            //! 
            //! <div class="code-wrap">
            //! <pre>
            //! $ cargo test -p my_project --verbose
            //! </pre>
            //! </div>
            //! 
            //! `cargo` here is an executable name, everything to the right of it separated by spaces are the
            //! options.
            //! 
            //! Nowadays programs share mostly similar conventions about what a command line argument is, it
            //! wasn't the case before though. Let's cover the basic types.
            //!
            //! - [Options, switches or flags](_0_switch)
            //! - [Option arguments or arguments](_1_argument)
            //! - [Operands or positional items](_2_positional)
            //! - [Commands or subcommands](_3_command)
            //! - [Exotic schemas](_4_exotic)
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Combinatoric API &rarr;](super::_1_combinatoric_api)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            pub mod _0_switch {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Option arguments or arguments &rarr;](super::_1_argument)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Options, switches or flags
                //! 
                //! Options or flags usually starts with a dash, a single dash for short options and a double dash for
                //! long one. Several short options can usually be squashed together with a single dash in front of
                //! them to save on typing: `-vvv` can be parsed the same as `-v -v -v`. Options don't have any
                //! other information apart from being there or not. Relative position usually does not matter and
                //! `--alpha --beta` should parse the same as `--beta --alpha`.
                //! 
                //! <div class="code-wrap">
                //! <pre>
                //! $ cargo <span style="font-weight: bold">--help</span>
                //! $ ls <span style="font-weight: bold">-la</span>
                //! $ ls <span style="font-weight: bold">--time --reverse</span>
                //! </pre>
                //! </div>
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/switch.md"))]
                //! 
                //! For more detailed info see [`NamedArg::switch`] and
                //! [`NamedArg::flag`]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Option arguments or arguments &rarr;](super::_1_argument)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _1_argument {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Options, switches or flags](super::_0_switch)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Operands or positional items &rarr;](super::_2_positional)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Option arguments or arguments
                //! 
                //! Option arguments are similar to regular options but they come with an extra value attached.
                //! Value can be separated by a space, `=` or directly adjacent to a short name. Same as with
                //! options - their relative position usually doesn't matter.
                //! 
                //! <div class="code-wrap">
                //! <pre>
                //! $ cargo build <span style="font-weight: bold">--package bpaf</span>
                //! $ cargo test <span style="font-weight: bold">-j2</span>
                //! $ cargo check <span style="font-weight: bold">--bin=megapotato</span>
                //! </pre>
                //! </div>
                //! 
                //! In the generated help message or documentation they come with a placeholder metavariable,
                //! usually a short, all-caps word describing what the value means: `NAME`, `AGE`, `SPEC`, and `CODE`
                //! are all valid examples.
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/argument.md"))]
                //! 
                //! For more detailed info see [`NamedArg::argument`]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Options, switches or flags](super::_0_switch)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Operands or positional items &rarr;](super::_2_positional)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _2_positional {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Option arguments or arguments](super::_1_argument)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Commands or subcommands &rarr;](super::_3_command)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Operands or positional items
                //! 
                //! Operands are usually items that are present on a command line and not prefixed by a short or
                //! long name. They are usually used to represent the most important part of the operation:
                //! `cat Cargo.toml` - display THIS file, `rm -rf target` - remove THIS folder and so on.
                //! 
                //! <div class="code-wrap">
                //! <pre>
                //! $ cat <span style="font-weight: bold">/etc/passwd</span>
                //! $ rm -rf <span style="font-weight: bold">target</span>
                //! $ man <span style="font-weight: bold">gcc</span>
                //! </pre>
                //! </div>
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/positional.md"))]
                //! 
                //! For more detailed info see [`positional`](crate::positional) and
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Option arguments or arguments](super::_1_argument)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Commands or subcommands &rarr;](super::_3_command)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _3_command {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Operands or positional items](super::_2_positional)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Exotic schemas &rarr;](super::_4_exotic)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Commands or subcommands
                //! 
                //! Commands are similar to positional items, but instead of representing an item they start
                //! a whole new parser, usually with its help and other arguments. Commands allow a single
                //! application to perform multiple different functions. The command parser will be able to parse all
                //! the command line options to the right of the command name
                //! 
                //! <div class="code-wrap">
                //! <pre>
                //! $ cargo <span style="font-weight: bold">build --release</span>
                //! $ cargo <span style="font-weight: bold">clippy</span>
                //! $ cargo <span style="font-weight: bold">asm --intel --everything</span>
                //! </pre>
                //! </div>
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/command.md"))]
                //! 
                //! For more detailed info see [`OptionParser::command`]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Operands or positional items](super::_2_positional)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Exotic schemas &rarr;](super::_4_exotic)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _4_exotic {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Commands or subcommands](super::_3_command)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Exotic schemas
                //! 
                //! While modern software tends to use just the options listed above you can still encounter
                //! programs created before those options became the norm and they use something completely different,
                //! let me give a few examples, see [the parsing cookbook](crate::_documentation::_2_howto)
                //! about actually parsing them
                //! 
                //! `su` takes an option that consists of a single dash `-`
                //! 
                //! <div class="code-wrap"><pre>
                //! $ su <span style="font-weight: bold">-</span>
                //! </pre></div>
                //! 
                //! `find` considers everything between `--exec` and `;` to be a single item.
                //! this example calls `ls -l` on every file `find` finds.
                //! 
                //! <div class="code-wrap"><pre>
                //! $ find /etc --exec ls -l '{}' \;
                //! </pre></div>
                //! 
                //! `Xorg` and related tools use flag-like items that start with a single `+` to enable a
                //! feature and with `-` to disable it.
                //! 
                //! <div class="code-wrap"><pre>
                //! $ xorg -backing +xinerama
                //! </pre></div>
                //! 
                //! `dd` takes several key-value pairs, this would create a 100M file
                //! <div class="code-wrap"><pre>
                //! $ dd if=/dev/zero of=dummy.bin bs=1M count=100
                //! </pre></div>
                //! 
                //! Most of the command line arguments in Turbo C++ 3.0 start with `/`. For example, option
                //! `/x` tells it to use all available extended memory, while `/x[=n]` limits it to n kilobytes
                //! <div class="code-wrap"><pre>
                //! C:\PROJECT>TC /x=200
                //! </pre></div>
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Commands or subcommands](super::_3_command)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Types of arguments &uarr;](super::super::_0_types_of_arguments)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
        use crate::*;
        }
        pub mod _1_combinatoric_api {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Types of arguments](super::_0_types_of_arguments)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Derive API tutorial &rarr;](super::_2_derive_api)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Combinatoric API
            //! Parse arguments without using proc macros
            //! 
            //! When making a parser in the Combinatoric style API you usually go through those steps
            //! 
            //! 1. Design data type your application will receive
            //! 2. Design command line options user will have to pass
            //! 3. Create a set of simple parsers
            //! 4. Combine and transform simple parsers to create the final data type
            //! 5. Transform the resulting [`Parser`] into [`OptionParser`] and run it
            //! 
            //! Let's go through some of them in more detail:
            //!
            //! - [Making a simple parser](_0_simple_parser)
            //! - [Transforming parsers](_1_chaining)
            //! - [Combining multiple simple parsers](_2_combining)
            //! - [Subcommand parsers](_3_subcommands)
            //! - [Improving the user experience](_4_decorating)
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Types of arguments](super::_0_types_of_arguments)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Derive API tutorial &rarr;](super::_2_derive_api)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            pub mod _0_simple_parser {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Transforming parsers &rarr;](super::_1_chaining)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Making a simple parser
                //! 
                //! In this chapter we'll go over making a few simple parsers.
                //!
                //! - [Switch parser](_0_switch)
                //! - [Argument parser](_1_argument)
                //! - [Positional item parser](_2_positional)
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Transforming parsers &rarr;](super::_1_chaining)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                pub mod _0_switch {
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //! 
                    //! [Argument parser &rarr;](super::_1_argument)
                    //! 
                    //!   </td>
                    //! </tr></table>
                    //! 
                    //! #### Switch parser
                    //! 
                    //! Let's start with the simplest possible one - a simple switch that gets parsed into a `bool`.
                    //! 
                    //! First of all - the switch needs a name - you can start with [`short`] or [`long`] and add more
                    //! names if you want: `long("simple")` or `short('s').long("simple")`. This gives something with
                    //! the type [`NamedArg`]:
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! use bpaf::parsers::NamedArg;
                    //! fn simple_switch() -> NamedArg {
                    //!     short('s').long("simple")
                    //! }
                    //! ```
                    //! 
                    //! From `NamedArg` you make a switch parser by calling [`NamedArg::switch`]. Usually, you do it
                    //! right away without assigning `NamedArg` to a variable.
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! fn simple_switch() -> impl Parser<bool> {
                    //!     short('s').long("simple").switch()
                    //! }
                    //! ```
                    //! 
                    //! The switch parser we just made implements trait [`Parser`] and to run it you convert it to [`OptionParser`] with
                    //! [`Parser::to_options`] and run it with [`OptionParser::run`]
                    //! 
                    //! Full example with some sample inputs and outputs:
                    #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_switch.md"))]
                    //! 
                    //! 
                    //! With [`NamedArg::help`] you can attach a help message that will be used in `--help` output.
                    //!
                    //!
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //! 
                    //! [Argument parser &rarr;](super::_1_argument)
                    //! 
                    //!   </td>
                    //! </tr></table>
                    //! 
                use crate::*;
                }
                pub mod _1_argument {
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //! 
                    //! [&larr; Switch parser](super::_0_switch)
                    //! 
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //! 
                    //! [Positional item parser &rarr;](super::_2_positional)
                    //! 
                    //!   </td>
                    //! </tr></table>
                    //! 
                    //! #### Argument parser
                    //! 
                    //! Next in complexity would be a parser to consume a named argument, such as `-p my_crate`. Same
                    //! as with the switch parser it starts from a `NamedArg` but the next method is [`NamedArg::argument`].
                    //! This method takes a metavariable name - a short description that will be used in the `--help`
                    //! output. `rustc` also needs to know the parameter type you are trying to parse, there are
                    //! several ways to do it:
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! # use std::path::PathBuf;
                    //! fn simple_argument_1() -> impl Parser<u32> {
                    //!     // rustc figures out the type from returned value
                    //!     long("number").argument("NUM")
                    //! }
                    //! 
                    //! fn simple_argument_2() -> impl Parser<String> {
                    //!     // type is specified explicitly with turbofish
                    //!     long("name").argument::<String>("NAME")
                    //! }
                    //! 
                    //! fn file_parser() -> OptionParser<PathBuf> {
                    //!     // OptionParser is a type for finalized parser, at this point you can
                    //!     // start adding extra information to the `--help` message
                    //!     long("file").argument::<PathBuf>("FILE").to_options()
                    //! }
                    //! ```
                    //! 
                    //! You can use any type for as long as it implements [`FromStr`]. To parse items that don't
                    //! implement it you can first parse a `String` or `OsString` and then use [`Parser::parse`], see
                    //! [the next chapter](super::super::_1_chaining) on how to do that.
                    //! 
                    //! Full example with some sample inputs and outputs:
                    #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_argument.md"))]
                    //!
                    //!
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //! 
                    //! [&larr; Switch parser](super::_0_switch)
                    //! 
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //! 
                    //! [Positional item parser &rarr;](super::_2_positional)
                    //! 
                    //!   </td>
                    //! </tr></table>
                    //! 
                use crate::*;
                }
                pub mod _2_positional {
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //! 
                    //! [&larr; Argument parser](super::_1_argument)
                    //! 
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //!   </td>
                    //! </tr></table>
                    //! 
                    //! #### Positional item parser
                    //! 
                    //! And the last simple option type is a parser for positional items. Since there's no name you use
                    //! the [`positional`] function directly. Similar to [`NamedArg::argument`] this method takes
                    //! a metavariable name and a type parameter in some form. You can also attach the help message
                    //! thanks to [`ParsePositional::help`]
                    //! 
                    //! Full example:
                    #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_positional.md"))]
                    //!
                    //!
                    //! &nbsp;
                    //! 
                    //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                    //!   <td style='width: 33%; text-align: left;'>
                    //! 
                    //! [&larr; Argument parser](super::_1_argument)
                    //! 
                    //!   </td>
                    //!   <td style='width: 34%; text-align: center;'>
                    //! 
                    //! [&uarr; Making a simple parser &uarr;](super::super::_0_simple_parser)
                    //! 
                    //!   </td>
                    //!   <td style='width: 33%; text-align: right;'>
                    //!   </td>
                    //! </tr></table>
                    //! 
                use crate::*;
                }
            use crate::*;
            }
            pub mod _1_chaining {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Making a simple parser](super::_0_simple_parser)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Combining multiple simple parsers &rarr;](super::_2_combining)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Transforming parsers
                //! 
                //! Once you have your primitive parsers done you might want to improve them a bit - add fallback
                //! values, or change them to consume multiple items, etc. Every primitive (or composite) parser
                //! implements [`Parser`] so most of the transformations are coming from this trait.
                //! 
                //! Say you have a parser that takes a crate name as a required argument you want to use in your own
                //! `cargo test` replacement
                //! 
                //! ```rust
                //! use bpaf::*;
                //! fn krate() -> impl Parser<String> {
                //!     long("crate").help("Crate name to process").argument("CRATE")
                //! }
                //! ```
                //! 
                //! You can turn it into, for example, an optional argument - something that returns
                //! `Some("my_crate")` if specified or `None` if it wasn't. Or to let the user to pass a multiple
                //! of them and collect them all into a `Vec`
                //! 
                //! 
                //! ```rust
                //! use bpaf::*;
                //! fn maybe_krate() -> impl Parser<Option<String>> {
                //!     long("crate")
                //!         .help("Crate name to process")
                //!         .argument("CRATE")
                //!         .optional()
                //! }
                //! 
                //! fn krates() -> impl Parser<Vec<String>> {
                //!     long("crate")
                //!         .help("Crate name to process")
                //!         .argument("CRATE")
                //!         .many()
                //! }
                //! ```
                //! 
                //! A complete example:
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_many.md"))]
                //! 
                //! Transforming a parser with a method from the `Parser` trait usually gives you a new parser back and
                //! you can chain as many transformations as you need.
                //! 
                //! Transformations available in the `Parser` trait things like adding fallback values, making
                //! the parser optional, making it so it consumes many but at least one value, changing how it is
                //! being shown in `--help` output, adding additional validation and parsing on top and so on.
                //! 
                //! The order of those chained transformations matters and for some operations using the right order
                //! makes code cleaner. For example, suppose you are trying to write a parser that takes an even
                //! number and this parser should be optional. There are two ways to write it:
                //! 
                //! Validation first:
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! fn even() -> impl Parser<Option<usize>> {
                //!     long("even")
                //!         .argument("N")
                //!         .guard(|&n| n % 2 == 0, "number must be even")
                //!         .optional()
                //! }
                //! ```
                //! 
                //! Optional first:
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! fn even() -> impl Parser<Option<usize>> {
                //!     long("even")
                //!         .argument("N")
                //!         .optional()
                //!         .guard(|&n| n.map_or(true, |n| n % 2 == 0), "number must be even")
                //! }
                //! ```
                //! 
                //! In later case validation function must deal with a possibility where a number is absent, for this
                //! specific example it makes code less readable.
                //! 
                //! One of the important types of transformations you can apply is a set of failing
                //! transformations. Suppose your application operates with numbers and uses `newtype` pattern to
                //! keep track of what numbers are odd or even. A parser that consumes an even number can use
                //! [`Parser::parse`] and may look like this:
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! pub struct Even(usize);
                //! 
                //! fn mk_even(n: usize) -> Result<Even, &'static str> {
                //!     if n % 2 == 0 {
                //!         Ok(Even(n))
                //!     } else {
                //!         Err("Not an even number")
                //!     }
                //! }
                //! 
                //! fn even() -> impl Parser<Even> {
                //!     long("even")
                //!         .argument::<usize>("N")
                //!         .parse(mk_even)
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Making a simple parser](super::_0_simple_parser)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Combining multiple simple parsers &rarr;](super::_2_combining)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _2_combining {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Transforming parsers](super::_1_chaining)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Subcommand parsers &rarr;](super::_3_subcommands)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Combining multiple simple parsers
                //! 
                //! A single-item option parser can only get you so far. Fortunately, you can combine multiple
                //! parsers with [`construct!`] macro.
                //! 
                //! For sequential composition (all the fields must be present) you write your code as if you are
                //! constructing a structure, enum variant or a tuple and wrap it with `construct!`. Both
                //! a constructor and parsers must be present in the scope. If instead of a parser you have a function
                //! that creates one - just add `()` after the name:
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! struct Options {
                //!     alpha: usize,
                //!     beta: usize
                //! }
                //! 
                //! fn alpha() -> impl Parser<usize> {
                //!     long("alpha").argument("ALPHA")
                //! }
                //! 
                //! fn both() -> impl Parser<Options> {
                //!     let beta = long("beta").argument("BETA");
                //!     // call `alpha` function, and use result to make parser
                //!     // for field `alpha`, use parser `beta` for field `beta`
                //!     construct!(Options { alpha(), beta })
                //! }
                //! ```
                //! 
                //! Full example:
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_construct.md"))]
                //! 
                //! If you are using positional parsers - they must go to the right-most side and will run in
                //! the order you specify them. For named parsers order affects only the `--help` message.
                //! 
                //! The second type of composition `construct!` offers is a parallel composition. You pass multiple
                //! parsers that produce the same result type in `[]` and `bpaf` selects one that fits best with
                //! the data user gave.
                //! 
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_choice.md"))]
                //! 
                //! If parsers inside parallel composition can parse the same object - the longest possible match
                //! should go first since `bpaf` picks an earlier parser if everything else is equal, otherwise it
                //! does not matter. In this example `construct!([miles, km])` produces the same results as
                //! `construct!([km, miles])` and only `--help` message is going to be different.
                //! 
                //! Parsers created with [`construct!`] still implement the [`Parser`] trait so you can apply more
                //! transformation on top. For example same as you can make a simple parser optional - you can make
                //! a composite parser optional. Parser transformed this way will succeed if both `--alpha` and
                //! `--beta` are present or neither of them:
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! struct Options {
                //!     alpha: usize,
                //!     beta: usize
                //! }
                //! 
                //! fn parser() -> impl Parser<Option<Options>> {
                //!     let alpha = long("alpha").argument("ALPHA");
                //!     let beta = long("beta").argument("BETA");
                //!     construct!(Options { alpha, beta }).optional()
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Transforming parsers](super::_1_chaining)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Subcommand parsers &rarr;](super::_3_subcommands)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _3_subcommands {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Combining multiple simple parsers](super::_2_combining)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Improving the user experience &rarr;](super::_4_decorating)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Subcommand parsers
                //! 
                //! To make a parser for a subcommand you make an `OptionParser` for that subcommand first as if it
                //! was the only thing your application would parse then turn it into a regular [`Parser`]
                //! you can further compose with [`OptionParser::command`].
                //! 
                //! This gives [`ParseCommand`] back, you can add aliases or tweak the help message if you want to.
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_command.md"))]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Combining multiple simple parsers](super::_2_combining)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Improving the user experience &rarr;](super::_4_decorating)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _4_decorating {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Subcommand parsers](super::_3_subcommands)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Improving the user experience
                //! 
                //! Once you have the final parser done there are still a few ways you can improve user experience.
                //! [`OptionParser`] comes equipped with a few methods that let you set version number,
                //! description, help message header and footer and so on.
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_to_options.md"))]
                //! 
                //! There are a few other things you can do:
                //! 
                //! - group some of the primitive parsers into logical blocks for `--help` message with
                //!   [`Parser::group_help`]
                //! - add tests to make sure important combinations are handled the way they are supposed to
                //!   after any future refactors with [`OptionParser::run_inner`]
                //! - add a test to make sure that bpaf internal invariants are satisfied with
                //!   [`OptionParser::check_invariants`]
                //! - generate user documentation in manpage and markdown formats with
                //!   [`OptionParser::render_manpage`] and [`OptionParser::render_markdown`]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Subcommand parsers](super::_3_subcommands)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Combinatoric API &uarr;](super::super::_1_combinatoric_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
        use crate::*;
        }
        pub mod _2_derive_api {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Combinatoric API](super::_1_combinatoric_api)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Designing a good datatype &rarr;](super::_3_picking_type)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Derive API tutorial
            //! Create a parser by defining a structure
            //! 
            //! 
            //! When making a parser using Derive API you should go through approximately following steps:
            //! 
            //! 1. Design data type your application will receive
            //! 2. Design command line options user will have to pass
            //! 3. Add `#[derive(Bpaf, Debug, Clone)]` on top of your type or types
            //! 4. Add `#[bpaf(xxx)]` annotations on types and fields
            //! 5. And `#[bpaf(options)]` to the top type
            //! 6. Run the resulting parser
            //! 
            //! 
            //! Let’s go through some of them in more detail:
            //!
            //! - [Getting started with derive macro](_0_intro)
            //! - [Customizing flag and argument names](_1_custom_names)
            //! - [Customizing the consumers](_2_custom_consumers)
            //! - [Transforming parsed values](_3_postpr)
            //! - [Parsing structs and enums](_4_enums_and_structs)
            //! - [What gets generated](_5_generate)
            //! - [Making nested parsers](_6_nesting)
            //! - [Parsing subcommands](_7_commands)
            //! - [Making a cargo command](_8_cargo)
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Combinatoric API](super::_1_combinatoric_api)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Designing a good datatype &rarr;](super::_3_picking_type)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            pub mod _0_intro {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Customizing flag and argument names &rarr;](super::_1_custom_names)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Getting started with derive macro
                //! 
                //! Let's take a look at a simple example
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_intro.md"))]
                //! 
                //! `bpaf` is trying hard to guess what you are trying to achieve just from the types so it will
                //! pick up types, doc comments, presence or absence of names, but it is possible to customize all
                //! of it, add custom transformations, validations and more.
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Customizing flag and argument names &rarr;](super::_1_custom_names)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _1_custom_names {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Getting started with derive macro](super::_0_intro)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Customizing the consumers &rarr;](super::_2_custom_consumers)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Customizing flag and argument names
                //! 
                //! By default names for flag names are taken directly from the field names so usually you don't
                //! have to do anything about it, but you can change it with annotations on the fields themselves:
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_custom_name.md"))]
                //! 
                //! Rules for picking the name are:
                //! 
                //! 1. With no annotations field name longer than a single character becomes a long name,
                //!    single character name becomes a short name
                //! 2. Adding either `long` or `short` disables item 1, so adding `short` disables the long name
                //! 3. `long` or `short` annotation without a parameter derives a value from a field name
                //! 4. `long` or `short` with a parameter uses that instead
                //! 5. You can have multiple `long` and `short` annotations, the first of each type becomes a
                //!    visible name, remaining are used as hidden aliases
                //! 
                //! And if you decide to add names - they should go to the left side of the annotation list
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Getting started with derive macro](super::_0_intro)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Customizing the consumers &rarr;](super::_2_custom_consumers)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _2_custom_consumers {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Customizing flag and argument names](super::_1_custom_names)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Transforming parsed values &rarr;](super::_3_postpr)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Customizing the consumers
                //! 
                //! By default, `bpaf` picks parsers depending on a field type according to those rules:
                //! 
                //! 1. `bool` fields are converted into switches: [`NamedArg::switch`](crate::parsers::NamedArg::switch)
                //! 2. `()` (unit) fields, unit variants of an enum or unit structs themselves are handled as
                //!    [`NamedArg::req_flag`](crate::parsers::NamedArg::req_flag) and thus users must always specify
                //!    them for the parser to succeed
                //! 3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but
                //!    smartly, so non-utf8 `PathBuf`/`OsString` are working as expected.
                //! 4. For values wrapped in `Option` or `Vec` bpaf derives the inner parser and then applies
                //!    applies logic from [`Parser::optional`] and [`Parser::many`] respectively.
                //! 
                //! You can change it with annotations like `switch`, `argument` or `positional`
                //! 
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_custom_consumer.md"))]
                //! 
                //! With arguments that consume a value you can specify its type using turbofish-line syntax
                //! 
                //! 
                //! ```no_run
                //! # use bpaf::*;
                //! #[derive(Debug, Clone, Bpaf)]
                //! #[bpaf(options)]
                //! pub struct Options {
                //!     /// A custom argument
                //!     #[bpaf(positional::<usize>("LENGTH"))]
                //!     argument: usize,
                //! }
                //! 
                //! fn main() {
                //!     let opts = options().run();
                //!     println!("{:?}", opts)
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Customizing flag and argument names](super::_1_custom_names)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Transforming parsed values &rarr;](super::_3_postpr)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _3_postpr {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Customizing the consumers](super::_2_custom_consumers)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Parsing structs and enums &rarr;](super::_4_enums_and_structs)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Transforming parsed values
                //! 
                //! Once the field has a consumer you can start applying transformations from the [`Parser`] trait.
                //! Annotation share the same names and follow the same composition rules as in Combinatoric API.
                //! 
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_postpr.md"))]
                //! 
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Customizing the consumers](super::_2_custom_consumers)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Parsing structs and enums &rarr;](super::_4_enums_and_structs)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _4_enums_and_structs {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Transforming parsed values](super::_3_postpr)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [What gets generated &rarr;](super::_5_generate)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Parsing structs and enums
                //! 
                //! To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
                //! it for some other purpose as well and want to skip them during parsing you can use [`pure`] to
                //! fill in values in member fields and `#[bpaf(skip)]` on enum variants you want to ignore, see
                //! combinatoric example in [`Parser::last`].
                //! 
                //! If you use `#[derive(Bpaf)]` on an enum parser will produce a variant for which all the parsers
                //! succeed.
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_enum.md"))]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Transforming parsed values](super::_3_postpr)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [What gets generated &rarr;](super::_5_generate)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _5_generate {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Parsing structs and enums](super::_4_enums_and_structs)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Making nested parsers &rarr;](super::_6_nesting)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### What gets generated
                //! 
                //! Usually calling derive macro on a type generates code to derive a trait implementation for this
                //! type. With bpaf it's slightly different. It instead generates a function with a name that
                //! depends on the name of the type and gives either a composable parser (`Parser`) or option parser
                //! (`OptionParser`) back.
                //! 
                //! You can customize the function name with `generate` annotation at the top level:
                //! 
                //! ```no_run
                //! # use bpaf::*;
                //! #[derive(Debug, Clone, Bpaf)]
                //! #[bpaf(options, generate(my_options))]
                //! pub struct Options {
                //!     /// A simple switch
                //!     switch: bool
                //! }
                //! 
                //! 
                //! fn main() {
                //!     let opts = my_options().run();
                //!     println!("{:?}", opts);
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Parsing structs and enums](super::_4_enums_and_structs)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Making nested parsers &rarr;](super::_6_nesting)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _6_nesting {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; What gets generated](super::_5_generate)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Parsing subcommands &rarr;](super::_7_commands)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Making nested parsers
                //! 
                //! Up to this point, we've been looking at cases where fields of a structure are all simple
                //! parsers, possibly wrapped in `Option` or `Vec`, but it is also possible to nest derived parsers
                //! too:
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_nesting.md"))]
                //! 
                //! 
                //! `external` takes an optional function name and will call that function to make the parser for
                //! the field. You can chain more transformations after the `external` and if the name is absent -
                //! `bpaf` would use the field name instead, so you can also write the example above as
                //! 
                //! 
                //! ```rust
                //! # use bpaf::*;
                //! #[derive(Debug, Clone, Bpaf)]
                //! pub enum Format {
                //!     /// Produce output in HTML format
                //!     Html,
                //!     /// Produce output in Markdown format
                //!     Markdown,
                //!     /// Produce output in manpage format
                //!     Manpage,
                //! }
                //! 
                //! #[derive(Debug, Clone, Bpaf)]
                //! #[bpaf(options)]
                //! pub struct Options {
                //!     /// File to process
                //!     input: String,
                //!     #[bpaf(external)]
                //!     format: Format,
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; What gets generated](super::_5_generate)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Parsing subcommands &rarr;](super::_7_commands)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _7_commands {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Making nested parsers](super::_6_nesting)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Making a cargo command &rarr;](super::_8_cargo)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Parsing subcommands
                //! 
                //! The easiest way to define a group of subcommands is to have them inside the same enum with variant
                //! constructors annotated with `#[bpaf(command("name"))]` with or without the name
                //! 
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_commands.md"))]
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Making nested parsers](super::_6_nesting)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //! 
                //! [Making a cargo command &rarr;](super::_8_cargo)
                //! 
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
            pub mod _8_cargo {
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Parsing subcommands](super::_7_commands)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
                //! #### Making a cargo command
                //! 
                //! To make a cargo command you should pass its name as a parameter to `options`. In this example,
                //! `bpaf` will parse extra parameter cargo passes and you will be able to use it either directly
                //! with `cargo run` from the repository, running it by `cargo-asm` name or with `cargo asm` name.
                //! 
                //! ```no_run
                //! # use bpaf::*;
                //! #[derive(Debug, Clone, Bpaf)]
                //! #[bpaf(options("asm"))]
                //! pub struct Options {
                //!     /// A simple switch
                //!     switch: bool
                //! }
                //! 
                //! 
                //! fn main() {
                //!     let opts = options().run();
                //!     println!("{:?}", opts);
                //! }
                //! ```
                //!
                //!
                //! &nbsp;
                //! 
                //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
                //!   <td style='width: 33%; text-align: left;'>
                //! 
                //! [&larr; Parsing subcommands](super::_7_commands)
                //! 
                //!   </td>
                //!   <td style='width: 34%; text-align: center;'>
                //! 
                //! [&uarr; Derive API tutorial &uarr;](super::super::_2_derive_api)
                //! 
                //!   </td>
                //!   <td style='width: 33%; text-align: right;'>
                //!   </td>
                //! </tr></table>
                //! 
            use crate::*;
            }
        use crate::*;
        }
        pub mod _3_picking_type {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Derive API tutorial](super::_2_derive_api)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Designing a good datatype
            //! bpaf allows you to reduce the size of legal values to valid ones
            //! 
            //! Parsing usually starts with deciding what kind of data your application wants to get from the user.
            //! You should try to take advantage of the Rust type system, try to represent the result such that more
            //! validation can be done during parsing.
            //! 
            //! Data types can represent a set of *legal* states - for example, for u8 this is all the numbers
            //! from 0 to 255, while your app logic may only operate correctly only on some set of *valid*
            //! states: if this u8 represents a fill ratio for something in percents - only valid numbers are
            //! from 0 to 100. You can try to narrow down the set of legal states to valid states with [newtype
            //! pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html). This newtype will
            //! indicate through the type when you've already done validation. For the fill ratio example you can
            //! implement a newtype along with `FromStr` implementation to get validation for free during
            //! parsing.
            //! 
            //! 
            //! ```no_run
            //! # use std::str::FromStr;
            //! # use bpaf::*;
            //! #[derive(Debug, Clone, Copy)]
            //! pub struct Ratio(u8);
            //! 
            //! impl FromStr for Ratio {
            //!     type Err = &'static str;
            //! 
            //!     fn from_str(s: &str) -> Result<Self, Self::Err> {
            //!         match s.parse() {
            //!             Ok(n) if n <= 100 => Ok(Ratio(n)),
            //!             _ => Err("Invalid fill ratio")
            //!         }
            //!     }
            //! }
            //! 
            //! #[derive(Debug, Clone, Bpaf)]
            //! #[bpaf(options)]
            //! struct Options {
            //!     /// Fill ratio
            //!     ratio: Ratio
            //! }
            //! 
            //! fn main() {
            //!     println!("{:?}", options().run());
            //! }
            //! ```
            //! 
            //! 
            //! Try using enums instead of structs for mutually exclusive options:
            //! 
            //! ```no_check
            //! /// Good format selection
            //! #[derive(Debug, Clone, Bpaf)]
            //! #[bpaf(options)]
            //! enum OutputFormat {
            //!     Intel,
            //!     Att,
            //!     Llvm
            //! }
            //! 
            //! fn main() {
            //!     let format = output_format().run();
            //! 
            //!     // `rustc` ensures you handle each case, parser won't try to consume
            //!     // combinations of flags it can't represent. For example it won't accept
            //!     // both `--intel` and `--att` at once
            //!     // (unless it can collect multiple of them in a vector)
            //!     match format {
            //!         OutputFormat::Intel => ...,
            //!         OutputFormat::Att => ...,
            //!         OutputFormat::Llvm => ...,
            //!     }
            //! }
            //! ```
            //! 
            //! While it's easy to see how flags like `--intel` and `--att` maps to each of those bools,
            //! consuming inside your app is more fragile
            //! 
            //! ```no_check
            //! /// Bad format selection
            //! #[derive(Debug, Clone, Bpaf)]
            //! #[bpaf(options)]
            //! struct OutputFormat {
            //!     intel: bool,
            //!     att: bool,
            //!     llvm: bool,
            //! }
            //! 
            //! fn main() {
            //!     let format = output_format().run();
            //!     // what happens when none matches? Or all of them?
            //!     // What happens when you add a new output format?
            //!     if format.intel {
            //!         ...
            //!     } else if format.att {
            //!         ...
            //!     } else if format.llvm {
            //!         ...
            //!     } else {
            //!         // can this branch be reached?
            //!     }
            //! }
            //! ```
            //! 
            //! Mutually exclusive things are not limited to just flags. For example if your program can take
            //! input from several different sources such as file, database or interactive input it's a good
            //! idea to use enum as well:
            //! 
            //! ```no_check
            //! /// Good input selection
            //! #[derive(Debug, Clone, Bpaf)]
            //! enum Input {
            //!     File {
            //!         filepath: PathBuf,
            //!     }
            //!     Database {
            //!         user: String,
            //!         password: String.
            //!     }
            //!     Interactive,
            //! }
            //! ```
            //! 
            //! If your codebase uses newtype pattern - it's a good idea to use it starting from the command
            //! options:
            //! 
            //! ```no_check
            //! #[derive(Debug, Clone, Bpaf)]
            //! struct Options {
            //!     // better than taking a String and parsing internally
            //!     date: NaiveDate,
            //!     // f64 might work too, but you can start from some basic sanity checks
            //!     speed: Speed
            //! }
            //! ```
            //! 
            //! 
            //! # More reading
            //! 
            //! - <https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/>
            //! - <https://geeklaunch.io/blog/make-invalid-states-unrepresentable/>
            //! - <https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/>
            //! - <https://khalilstemmler.com/articles/typescript-domain-driven-design/make-illegal-states-unrepresentable/>
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Derive API tutorial](super::_2_derive_api)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Tutorials &uarr;](super::super::_1_tutorials)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
    use crate::*;
    }
    pub mod _2_howto {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Tutorials](super::_1_tutorials)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Parsing cookbook &rarr;](super::_3_cookbook)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### HOWTO - practical, oriented to solving problems guides
        //!
        //! - [Testing your parsers](_0_testing)
        //! - [Dynamic shell completion](_1_completion)
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Tutorials](super::_1_tutorials)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Parsing cookbook &rarr;](super::_3_cookbook)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        pub mod _0_testing {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HOWTO - practical, oriented to solving problems guides &uarr;](super::super::_2_howto)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Dynamic shell completion &rarr;](super::_1_completion)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Testing your parsers
            //! 
            //! You can test values your parser produces and expected output
            //! 
            //! ```no_run
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
            //!         .run_inner(&["--help"])
            //!         .unwrap_err()
            //!         .unwrap_stdout();
            //!     let expected_help = "\
            //! Usage --user=ARG
            //! <skip>
            //! ";
            //! 
            //!     assert_eq!(help, expected_help);
            //! }
            //! 
            //! #[test]
            //! fn test_value() {
            //!     let value = options()
            //!          .run_inner(&["--user", "Bob"])
            //!          .unwrap();
            //!     assert_eq!(value.user, "Bob");
            //! }
            //! ```
            //! 
            //! [`OptionParser::run_inner`] takes [`Args`] or anything that can be converted to it, in most
            //! cases using a static slice with strings is enough.
            //! 
            //! Easiest way to consume [`ParseFailure`] for testing purposes is with
            //! [`ParseFailure::unwrap_stderr`] and [`ParseFailure::unwrap_stdout`] - result will lack any colors
            //! even with them enabled which makes testing easier.
            //! 
            //! Successful result parse produces a value, "failed" parse produces stdout or stderr outputs -
            //! stdout to print help message or version number and stderr to print the error message.
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HOWTO - practical, oriented to solving problems guides &uarr;](super::super::_2_howto)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Dynamic shell completion &rarr;](super::_1_completion)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _1_completion {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Testing your parsers](super::_0_testing)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HOWTO - practical, oriented to solving problems guides &uarr;](super::super::_2_howto)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Dynamic shell completion
            //! 
            //! `bpaf` implements shell completion to allow to automatically fill in not only flag and command
            //! names, but also argument and positional item values.
            //! 
            //! 1. Enable `autocomplete` feature:
            //! 
            //! 
            //! 	```toml
            //! 	bpaf = { version = "0.9", features = ["autocomplete"] }
            //! 	```
            //! 
            //! 2. Decorate [`argument`](crate::parsers::NamedArg::argument) and [`positional`] parsers with
            //!     [`Parser::complete`] to provide completion functions for arguments
            //! 
            //! 
            //! 3. Depending on your shell generate appropriate completion file and place it to whereever your
            //!     shell is going to look for it, name of the file should correspond in some way to name of
            //!     your program. Consult manual for your shell for the location and named conventions:
            //! 
            //! 	 1. **bash**
            //! 		```console
            //! 		$ your_program --bpaf-complete-style-bash >> ~/.bash_completion
            //! 		```
            //! 
            //! 	 1. **zsh**: note `_` at the beginning of the filename
            //! 		```console
            //! 		$ your_program --bpaf-complete-style-zsh > ~/.zsh/_your_program
            //! 		```
            //! 
            //! 	 1. **fish**
            //! 		```console
            //! 		$ your_program --bpaf-complete-style-fish > ~/.config/fish/completions/your_program.fish
            //! 		```
            //! 
            //! 	 1. **elvish**
            //! 		```console
            //! 		$ your_program --bpaf-complete-style-elvish >> ~/.config/elvish/rc.elv
            //! 		```
            //! 
            //! 4. Restart your shell - you need to done it only once or optionally after bpaf major version
            //!     upgrade: generated completion files contain only instructions how to ask your program for
            //!     possible completions and don’t change even if options are different.
            //! 
            //! 
            //! 5. Generated scripts rely on your program being accessible in $PATH
            //! 
            //! 
            //! 
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Testing your parsers](super::_0_testing)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HOWTO - practical, oriented to solving problems guides &uarr;](super::super::_2_howto)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
    use crate::*;
    }
    pub mod _3_cookbook {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; HOWTO - practical, oriented to solving problems guides](super::_2_howto)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Theory explanation &rarr;](super::_4_explanation)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### Parsing cookbook
        //! How to parse less frequent combinations
        //! 
        //! While `bpaf`'s design tries to cover the most common use cases, mostly
        //! [posix conventions](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/basedefs/V1_chap12.html),
        //! it can also handle some more unusual requirements. It might come at the cost of having to write
        //! more code, more confusing error messages or worse performance, but it will get the job done.
        //!
        //! - [`find(1)`: `find -exec commands -flags terminated by \;`](_00_find)
        //! - [`dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`](_01_dd)
        //! - [`Xorg(1)`: `Xorg +xinerama +extension name`](_02_xorg)
        //! - [Command chaining](_03_command_chaining) - Lets you do things like `setup.py sdist bdist`: [command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining)
        //! - [Multi-value arguments: `--foo ARG1 ARG2 ARG3`](_04_multi_value)
        //! - [Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`](_05_struct_groups)
        //! - [Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`](_06_multi_flag)
        //! - [Skipping optional positional items if parsing or validation fails](_07_skip_positional)
        //! - [Implementing cargo commands](_08_cargo_helper)
        //! - [Numeric flags - compression levels like in zip](_09_numeric_flags)
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; HOWTO - practical, oriented to solving problems guides](super::_2_howto)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //! 
        //! [Theory explanation &rarr;](super::_4_explanation)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        pub mod _00_find {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [`dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000` &rarr;](super::_01_dd)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### `find(1)`: `find -exec commands -flags terminated by \;`
            //! 
            //! An Example for `find` shows how to implement 3 different unusual options:
            //! 
            //! - an option with a long name but a single dash as a prefix: `-user bob`
            //! - an option that captures everything until the next fixed character
            //! - an option that takes a set of characters: `-mode -rw`, `mode /rw`
            //! 
            //! In all cases, long name with a single dash is implemented by the [`literal`] with
            //! [`ParseAny::anywhere`](crate::parsers::ParseAny::anywhere) with some items made `adjacent` to it.
            //! 
            //! To parse `-user bob` this is simply literal `-user` adjacent to a positional item with `map` to
            //! focus on the interesting part.
            //! 
            //! For `-exec more things here ;` this is a combination of literal `-exec`, followed by `many`
            //! items that are not `;` parsed positionally with `any` followed by `;` - again with `any`, but
            //! `literal` works too.
            //! 
            //! And lastly to parse mode - after the tag, we accept `any` to be able to handle a combination of
            //! modes that may or may not start with `-` and use [`Parser::parse`] to parse them or fail.
            //! 
            //! All special cases are made optional with [`Parser::optional`], but [`Parser::fallback`] also
            //! works.
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/find.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [`dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000` &rarr;](super::_01_dd)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _01_dd {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `find(1)`: `find -exec commands -flags terminated by \;`](super::_00_find)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [`Xorg(1)`: `Xorg +xinerama +extension name` &rarr;](super::_02_xorg)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`
            //! 
            //! This example implements syntax similar to `dd` command. The main idea is to implement something to
            //! make it simple to make parsers for `PREFIX=SUFFIX`, where prefix is fixed for each parser - for
            //! example `if=` or `of=` and suffix is parsed with usual [`FromStr`](std::str::FromStr) trait.
            //! 
            //! The function `tag` serves this purpose. It performs the following steps:
            //! 
            //! - consume any item that starts with a prefix at any argument position with [`any`] and
            //!   [`ParseAny::anywhere`]
            //! - attaches help message and custom metadata to make `--help` friendlier
            //! - parses suffix with [`Parser::parse`]
            //! 
            //! The rest of the parser simply uses `tag` to parse a few of `dd` arguments
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/dd.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `find(1)`: `find -exec commands -flags terminated by \;`](super::_00_find)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [`Xorg(1)`: `Xorg +xinerama +extension name` &rarr;](super::_02_xorg)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _02_xorg {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`](super::_01_dd)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Command chaining &rarr;](super::_03_command_chaining)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### `Xorg(1)`: `Xorg +xinerama +extension name`
            //! 
            //! This example implements syntax similar to used by `Xorg` or similar programs. As usual with
            //! strange examples [`any`] serves an important role.
            //! 
            //! The example implements the following parsers:
            //! 
            //! - enable or disable an extension using `+ext name` and `-ext name` like syntax
            //! - enable or disable specific extensions with syntax like `-xinerama` or `+backing`
            //! 
            //! Both parsers use [`any`] with [`ParseAny::anywhere`]
            //! 
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/xorg.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`](super::_01_dd)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Command chaining &rarr;](super::_03_command_chaining)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _03_command_chaining {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `Xorg(1)`: `Xorg +xinerama +extension name`](super::_02_xorg)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Multi-value arguments: `--foo ARG1 ARG2 ARG3` &rarr;](super::_04_multi_value)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Command chaining
            //! Lets you do things like `setup.py sdist bdist`: [command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining)
            //! 
            //! With [`adjacent`](crate::parsers::ParseCommand::adjacent)
            //! `bpaf` allows you to have several commands side by side instead of being nested.
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_command.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; `Xorg(1)`: `Xorg +xinerama +extension name`](super::_02_xorg)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Multi-value arguments: `--foo ARG1 ARG2 ARG3` &rarr;](super::_04_multi_value)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _04_multi_value {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Command chaining](super::_03_command_chaining)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3` &rarr;](super::_05_struct_groups)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Multi-value arguments: `--foo ARG1 ARG2 ARG3`
            //! 
            //! By default arguments take at most one value, you can create multi value options by using
            //! [`adjacent`](crate::parsers::ParseCon::adjacent) modifier
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_0.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Command chaining](super::_03_command_chaining)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3` &rarr;](super::_05_struct_groups)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _05_struct_groups {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Multi-value arguments: `--foo ARG1 ARG2 ARG3`](super::_04_multi_value)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2` &rarr;](super::_06_multi_flag)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`
            //! 
            //! Groups of options that can be specified multiple times. All such groups should be kept without
            //! overwriting previous one.
            //! 
            //! ```console
            //!  $ prometheus_sensors_exporter \
            //!      \
            //!      `# 2 physical sensors located on physycial different i2c bus or address` \
            //!      --sensor \
            //!          --sensor-device=tmp102 \
            //!          --sensor-name="temperature_tmp102_outdoor" \
            //!          --sensor-i2c-bus=0 \
            //!          --sensor-i2c-address=0x48 \
            //!      --sensor \
            //!          --sensor-device=tmp102 \
            //!          --sensor-name="temperature_tmp102_indoor" \
            //!          --sensor-i2c-bus=1 \
            //!          --sensor-i2c-address=0x49 \
            //! ```
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_1.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Multi-value arguments: `--foo ARG1 ARG2 ARG3`](super::_04_multi_value)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2` &rarr;](super::_06_multi_flag)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _06_multi_flag {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`](super::_05_struct_groups)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Skipping optional positional items if parsing or validation fails &rarr;](super::_07_skip_positional)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`
            //! 
            //! So you can parse things while parsing things. Not sure why you might need this, but you can
            //! :)
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_4.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`](super::_05_struct_groups)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Skipping optional positional items if parsing or validation fails &rarr;](super::_07_skip_positional)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _07_skip_positional {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`](super::_06_multi_flag)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Implementing cargo commands &rarr;](super::_08_cargo_helper)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Skipping optional positional items if parsing or validation fails
            //! 
            //! Combinations like [`Parser::optional`] and
            //! [`ParseOptional::catch`](crate::parsers::ParseOptional::catch) allow to try to parse something
            //! and then handle the error as if pase attempt never existed
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/numeric_prefix.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`](super::_06_multi_flag)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Implementing cargo commands &rarr;](super::_08_cargo_helper)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _08_cargo_helper {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Skipping optional positional items if parsing or validation fails](super::_07_skip_positional)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Numeric flags - compression levels like in zip &rarr;](super::_09_numeric_flags)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Implementing cargo commands
            //! 
            //! With [`cargo_helper`](crate::batteries::cargo_helper) you can use your application as a `cargo` command.
            //! You will need to enable `batteries` feature while importing `bpaf`.
            //! 
            #![cfg_attr(not(doctest), doc = include_str!("docs2/cargo_helper.md"))]
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Skipping optional positional items if parsing or validation fails](super::_07_skip_positional)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //! 
            //! [Numeric flags - compression levels like in zip &rarr;](super::_09_numeric_flags)
            //! 
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
        pub mod _09_numeric_flags {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Implementing cargo commands](super::_08_cargo_helper)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Numeric flags - compression levels like in zip
            //! 
            //! While you can add flags in a usual way for compression levels using `short(1)`, `short(2)`, etc
            //! combined with `req_flag`, you can also parse all of then using [`any`]
            //! 
            //! ```no_run
            //! use bpaf::{doc::Style, *};
            //! 
            //! fn compression() -> impl Parser<usize> {
            //!     any::<isize, _, _>("COMP", |x: isize| {
            //!         if (-9..=-1).contains(&x) {
            //!             Some(x.abs().try_into().unwrap())
            //!         } else {
            //!             None
            //!         }
            //!     })
            //!     .metavar(&[
            //!         ("-1", Style::Literal),
            //!         (" to ", Style::Text),
            //!         ("-9", Style::Literal),
            //!     ])
            //!     .help("Compression level")
            //!     .anywhere()
            //! }
            //! 
            //! fn main() {
            //!     let opts = compression().to_options().run();
            //! 
            //!     println!("{:?}", opts);
            //! }
            //! ```
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //! 
            //! [&larr; Implementing cargo commands](super::_08_cargo_helper)
            //! 
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; Parsing cookbook &uarr;](super::super::_3_cookbook)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
        use crate::*;
        }
    use crate::*;
    }
    pub mod _4_explanation {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Parsing cookbook](super::_3_cookbook)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //!   </td>
        //! </tr></table>
        //! 
        //! #### Theory explanation
        //! Theoretical information about abstractions used by the library, oriented for understanding
        //! 
        //! 
        //! # Applicative functors, Category Theory? What is it about?
        //! 
        //! You don't need to read/understand this chapter in order to use the library but it might
        //! help to understand what makes it tick.
        //! 
        //! `bpaf` uses ideas from functional proggramming, specifically Functor, Applicative and
        //! Alternative to create a composable interface. Exposed API and the fact that individual
        //! components obey certain laws ensures that any composition of parsers is valid even if it
        //! doesn't make any sense.
        //! 
        //! ## Category theory
        //! 
        //! Category theory, also called Abstract Nonsense, is a general theory about mathematical
        //! structures and their relations. *Category* in CT constists of two sorts of abstractions:
        //! *objects* and *morphisms* along with some extra rules:
        //! - objects don't expose any information other than the name and only serve as start and end points for morphisms
        //! - morphisms must compose with associative composition
        //! - there must be an *identity morphism* for every object that maps the object to itself
        //! 
        //! A simple example of a category would be a category where objects are Rust types (here: `u8` ..
        //! `u64`) and morphisms are functions between those types (here: `a`, `b` and `c`):
        //! 
        //! ```rust
        //! fn a(i: u8) -> u16 {
        //!     3000 + i as u16
        //! }
        //! 
        //! fn b(i: u16) -> u32 {
        //!     40000 + i as u32
        //! }
        //! 
        //! fn c(i: u32) -> u64 {
        //!     40000 + i as u64
        //! }
        //! 
        //! /// Identity morphism
        //! fn id<T>(i: T) -> T {
        //!     i
        //! }
        //! 
        //! /// morphism composition:
        //! /// `comp (a, comp(b, c))` gives the same results as `comp(comp(a, b), c)`
        //! fn comp<F, G, A, B, C>(f: F, g: G) -> impl Fn(A) -> C
        //! where
        //!     F: Fn(A) -> B,
        //!     G: Fn(B) -> C,
        //! {
        //!     move |i| g(f(i))
        //! }
        //! ```
        //! 
        //! ## Composition and decomposition
        //! 
        //! Decomposition is one of the keys to solving big problems - you break down big problem into a
        //! bunch of small problems, solve them separately and compose back a solution. Decomposition is
        //! not required by computers but makes it easier to think about a problem: magical number for
        //! human short term memory is 7 plus minus 2 objects. Category theory, studies relations and
        //! composition can be a valuable tool: after all decomposition only makes sense when you can
        //! combine components back into a solution. Imperative algorithms that operate in terms of
        //! mutating variables are harder decompose - individual pieces need to be aware of the variables,
        //! functional and declarative approaches make it easier: calculating a sum of all the numbers in a
        //! vector can be decomposed into running an iterator over it and applying `fold` to it: `fold`
        //! doesn't need to know about iteration shape, iterator doesn't need to know about how values are
        //! used.
        //! 
        //! In category theory you are not allowed to look inside the objects at all and can distinguish
        //! between them only by means of the composition so as long as implemented API obeys the
        //! restrictions set by category theory - it should be very composable.
        //! 
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
        //! 
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
        //! 
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
        //! 
        //! ## `Parser` trait and `construct!` macro
        //! 
        //! [`Parser`] trait defines a context for values and gives access to `Functor` laws and [`construct!`]
        //! macro allows to compose several values according to `Applicative` and `Alternative` laws.
        //! 
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
        //! 
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
        //! 
        //! ## Taking the results out
        //! 
        //! The rest of the execution is relatively simple: getting console arguments from OS, doing the
        //! initial split into short/long flags and standalone words, disambiguating groups of short
        //! options from short options with attached values and applying all the transformations like
        //! `Reader::run` above would do.
        //!
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Parsing cookbook](super::_3_cookbook)
        //! 
        //!   </td>
        //!   <td style='width: 34%; text-align: center;'>
        //! 
        //! [&uarr; Project documentation &uarr;](super::super::_documentation)
        //! 
        //!   </td>
        //!   <td style='width: 33%; text-align: right;'>
        //!   </td>
        //! </tr></table>
        //! 
    use crate::*;
    }
use crate::*;
