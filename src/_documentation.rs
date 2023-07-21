//! #### Project documentation
//!
//! - [Introduction and design goals](_0_intro) - A quick intro. What, why and how
//! - [Tutorials](_1_tutorials) - practical, learning oriented guides
//! - [HowTo](_2_howto) - Practical solutions to common problems
//! - [Structured API reference](_3_reference) - A better overview of available functions
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
        //! ##### A quick intro. What, why and how
        //! 
        //! Bpaf is a lightweight and flexible command line parser that uses both combinatoric and derive
        //! style API
        //! 
        //! Combinatoric API usually means a bit more typing but no dependency on proc macros and more help
        //! from the IDE, derive API uses proc macro to save on typing but your IDE will be less likely to
        //! help you. Picking one API style does not lock you out from using the other style, you can mix
        //! and match both in a single parser
        //! 
        //! # Examples for both styles
        //! 
        #![cfg_attr(not(doctest), doc = include_str!("docs2/intro.md"))]
        //! 
        //! # Design goals
        //! 
        //! ## Parse, don't validate
        //! 
        //! `bpaf` tries hard to let you to move as much invariants about the user input you are
        //! trying to parse into rust type: for mutually exclusive options you can get `enum` with
        //! exclusive items going into separate branches, you can collect results into types like
        //! [`BTreeSet`](std::collections::BTreeSet), or whatever custom type you might have with
        //! custom parsing. Ideas for
        //! [making invalid states unrepresentable](https://geeklaunch.io/blog/make-invalid-states-unrepresentable/)
        //! and [using parsing over validation](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
        //! are not new.
        //! 
        //! That said you can also validate your inputs if this fits your problem better. If you want to
        //! ensure that sum of every numeric fields must be divisible by both 3 and 5, but only when it's
        //! Thursday - you can do that too.
        //! 
        //! ## Flexibility
        //! 
        //! While aiming to be a general purpose command line parser `bpaf` offers a few backdoors that
        //! allow to parse pretty much anything you want: chained commands, custom blocks of options, DOS
        //! style options (`/ofile.pas`), `dd` style options (`if=file of=out`), etc. Similar idea applies
        //! for what the parser can produce - your app operates with boxed string slices internally? `bpaf`
        //! will give you `Box<str>` instead of `String` if you ask it to.
        //! 
        //! The only restriction being that you cannot use information from items parsed earlier (but not
        //! the fact that something was parsed successfully or not) to decide to how to parse further
        //! options, and even then you can side step this restrictions by passing some shared state as a
        //! parameter to the parsers.
        //! 
        //! ## Reusability
        //! 
        //! Parsers in `bpaf` are not monolithic and you can share their parts across multiple binaries,
        //! workspace members or even independent projects. Say you have a multiple binaries in a workspace
        //! that perform different operations on some input. You can declare a parser for the input
        //! specifically, along with all the validations, help messages or shell dynamic completion
        //! functions you need and use it across all the binaries alongside with the arguments specific to
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
        //! While performance is an explicit non goal - `bpaf` does nothing that would pessimize it either,
        //! so performance is on par or better compared to fully featured parsers.
        //! 
        //! ## Correctness
        //! 
        //! `bpaf` would parse only items it can represent and will reject anything it cannot represent
        //! in the output. Say your parser accepts both `--intel` and `--att` flags, but encodes the result
        //! into `enum Style { Intel, Att }`, `bpaf` will accept those flags separately, but not if they
        //! are used both at once. If parser later collects multipe styles into a `Vec<Style>` then it
        //! will accept any combinationof those flags.
        //! 
        //! ## User friendly
        //! 
        //! `bpaf` tries to provide user friendly error messages, suggestions for typos but also scripts
        //! for shell completion, `man` pages and markdown documentation for web
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
        //! [HowTo &rarr;](super::_2_howto)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### Tutorials
        //! ##### practical, learning oriented guides
        //!
        //! - [Types of arguments](_0_types_of_arguments) - common types of line options and conventions
        //! - [Combinatoric API](_1_combinatoric_api) - parse without using proc macros
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
        //! [HowTo &rarr;](super::_2_howto)
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
            //! ##### common types of line options and conventions
            //! 
            //! This chapter serves as an introduction to available command line options and tries to set the
            //! terminology. If you are familiar with command line argument parsers in general - feel free top
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
            //! q!
            //! options.
            //! 
            //! Nowdays programs share mostly similar conventions about what a command line argument is, it
            //! wasn't the case before though. Let's cover the basic types
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
                //! Options or flags usually starts with a dash, single dash for short options and double dash for
                //! long one. Several short options usually can be squashed together with a single dash between
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
                //! options - relative position usually doesn't matter.
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
                //! usually a short, all caps word describing what the value means: `NAME`, `AGE`, `SPEC`, `CODE`
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
                //! Commands are similar to positional items, but instead of representing an item they represent
                //! a new parser, usually with its own help and other arguments. Commands allow a single
                //! applications to perform multiple different functions. Command parser will be able to parse all
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
                //! programs created before those options became norm and they use something complitely different,
                //! let me give a few examples, see [exotic howto](crate::_documentation::_2_howto::_1_exotic)
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
                //! `Xorg` and related tools use flag like items that start with a single `+` to enable a
                //! feature and with `-` to disable it.
                //! 
                //! <div class="code-wrap"><pre>
                //! $ xorg -backing +xinerama
                //! </pre></div>
                //! 
                //! `dd` takes several key value pairs, this would create a 100M file
                //! <div class="code-wrap"><pre>
                //! $ dd if=/dev/zero of=dummy.bin bs=1M count=100
                //! </pre></div>
                //! 
                //! Most of the command line arguments in Turbo C++ 3.0 start with `/`. For example option
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
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Combinatoric API
            //! ##### parse without using proc macros
            //! 
            //! When making parser in the Combinatoric style API you usually go though those steps
            //! 
            //! 1. Design data type your application will receive
            //! 2. Design command line options user will have to pass
            //! 3. Create a set of simple parsers
            //! 4. Combine and transform simple parsers to create the final data type
            //! 5. Transform resulting [`Parser`] into [`OptionParser`] and run it
            //! 
            //! Let's go though some of them in more details:
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
                    //! Let's start with the simpliest possible one - a simple switch that gets parsed into a `bool`.
                    //! 
                    //! First of all - switch needs a name - you can start with [`short`] or [`long`] and add more
                    //! names if you want: `long("simple")` or `short('s').long("simple")`. This gives something with
                    //! type [`NamedArg`]:
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! use bpaf::parsers::NamedArg;
                    //! fn simple_switch() -> NamedArg {
                    //!     short('s').long("simple")
                    //! }
                    //! ```
                    //! 
                    //! From `NamedArg` you make a switch parser by calling [`NamedArg::switch`]. Usually you do it
                    //! right away without assigning `NamedArg` to a variable.
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! fn simple_switch() -> impl Parser<bool> {
                    //!     short('s').long("simple").switch()
                    //! }
                    //! ```
                    //! 
                    //! Switch parser we just implements trait [`Parser`] and to run it you convert it to [`OptionParser`] with
                    //! [`Parser::to_options`] and run it with [`OptionParser::run`]
                    //! 
                    //! Full example with some sample inputs and outputs, click to open
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
                    //! as with switch parser it starts from a `NamedArg` but next method is [`NamedArg::argument`].
                    //! Method takes a metavariable name - a short description that will be used in the `--help`
                    //! output. `rustc` also needs to know the type of a variable you are trying to parse, there's
                    //! several ways to do it:
                    //! 
                    //! ```rust
                    //! # use bpaf::*;
                    //! # use std::path::PathBuf;
                    //! fn simple_argument_1() -> impl Parser<u32> {
                    //!     long("number").argument("NUM")
                    //! }
                    //! 
                    //! fn simple_argument_2() -> impl Parser<String> {
                    //!     long("name").argument::<String>("NAME")
                    //! }
                    //! 
                    //! fn file_parser() -> OptionParser<PathBuf> {
                    //!     long("file").argument::<PathBuf>("FILE").to_options()
                    //! }
                    //! ```
                    //! 
                    //! You can use any type for as long as it implements [`FromStr`]. See the next chapter about
                    //! parsing items that don't implement [`FromStr`]
                    //! 
                    //! Full example with some sample inputs and outputs, click to open
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
                    //! And the last simple option type is parser for positional items. Since there's no name you use
                    //! [`positional`] method directly which behaves similarly to [`NamedArg::argument`] - takes
                    //! metavariable name and a type parameter in some form. You can also attach help message directly
                    //! to it thanks to [`ParsePositional::help`]
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
                //! Say you a parser that takes a crate name as a required argument you want to use in you own
                //! `cargo test` replacement
                //! 
                //! ```rust
                //! use bpaf::*;
                //! fn krate() -> impl Parser<String> {
                //!     long("crate").help("Crate name to process").argument("CRATE")
                //! }
                //! ```
                //! 
                //! You can turn it into, for example, optional argument - something that returns
                //! `Some("my_crate")` if specified or `None` if it wasn't. Or to let user to pass multiple
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
                //! Transforming a parser with a method from `Parser` trait usually gives you a new parser back and
                //! you can chain as many transformations as you need.
                //! 
                //! Transformations available in the `Parser` trait things like adding fallback values, making
                //! parser optional, making it so it consumes many but at least one value, changing how it is
                //! being shown in `--help` output, adding additional validation and parsing on top and so on.
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
                //! A single item option parser can only get you so far. Fortunately you can combine multiple
                //! parsers together with [`construct!`] macro.
                //! 
                //! For sequential composition (all the fields must be present) you write your code as if you are
                //! constructing a structure, enum variant or a tuple and wrap it with `construct!`. Both
                //! constructor and parsers must be present in scope. If instead of a parser you have a function
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
                //!     construct!(Options { alpha(), beta })
                //! }
                //! ```
                //! 
                //! What you get back also implements a `Parser` trait so you can keep combining or applying extra
                //! transformations on top.
                //! 
                //! Full example:
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_construct.md"))]
                //! 
                //! If you are using positional parsers - they must go to the right most side and will run in
                //! order you specify them. For named parsers order affects only the --help` message.
                //! 
                //! Second type of composition `construct!` offers is a parallel composition. You pass multiple
                //! parsers that produce the same result type and `bpaf` runs one that fits best with the data user
                //! gave.
                //! 
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_choice.md"))]
                //! 
                //! If parsers inside parallel composition can parse the same object - longest possible match
                //! should go first since `bpaf` picks earlier parser if everything else is equal, otherwise it
                //! does not matter. In this example `construct!([miles, km])` produces the same results as
                //! `construct!([km, miles])` and only `--help` message is going to be different.
                //! 
                //! Parsers created with [`construct!`] still implement [`Parser`] trait so you can apply more
                //! transformation on top. For example same as you can make a simple parser optional - you can make
                //! composite parser optional. Such parser will succeed iff both `--alpha` and `--beta` are
                //! present.
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
                //! Once you have the final parser done there's still a few ways you can improve user experience.
                //! [`OptionParser`] comes equipped with a few methods that let you set version number,
                //! description, help message header and footer and so on.
                //! 
                #![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_to_options.md"))]
                //! 
                //! There's a few other things you can do:
                //! 
                //! - group some of the primitive parsers into logical blocks for `--help` message with
                //!   [`Parser::group_help`]
                //! - add tests to make sure important combinations are handled the way they supposed to
                //!   after any future refactors with [`OptionParser::run_inner`]
                //! - add a test to make sure that bpaf internal invariants are satisfied with
                //!   [`OptionParser::check_invariants`]
                //! - generate user documentation in manpage and markdown formats with
                //!   [`OptionParser::render_manpage] and [`OptionParser::render_markdown`]
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
        //! [Structured API reference &rarr;](super::_3_reference)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        //! #### HowTo
        //! ##### Practical solutions to common problems
        //!
        //! - [Parsing exotic options](_1_exotic) - If you need it
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
        //! [Structured API reference &rarr;](super::_3_reference)
        //! 
        //!   </td>
        //! </tr></table>
        //! 
        pub mod _1_exotic {
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HowTo &uarr;](super::super::_2_howto)
            //! 
            //!   </td>
            //!   <td style='width: 33%; text-align: right;'>
            //!   </td>
            //! </tr></table>
            //! 
            //! #### Parsing exotic options
            //! ##### If you need it
            //!
            //!
            //! &nbsp;
            //! 
            //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
            //!   <td style='width: 33%; text-align: left;'>
            //!   </td>
            //!   <td style='width: 34%; text-align: center;'>
            //! 
            //! [&uarr; HowTo &uarr;](super::super::_2_howto)
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
    pub mod _3_reference {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; HowTo](super::_2_howto)
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
        //! #### Structured API reference
        //! ##### A better overview of available functions
        //!
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; HowTo](super::_2_howto)
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
    use crate::*;
    }
    pub mod _4_explanation {
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Structured API reference](super::_3_reference)
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
        //! ##### Theoretical information about abstractions used by the library, oriented for understanding
        //!
        //!
        //! &nbsp;
        //! 
        //! <table width='100%' cellspacing='0' style='border: hidden;'><tr>
        //!   <td style='width: 33%; text-align: left;'>
        //! 
        //! [&larr; Structured API reference](super::_3_reference)
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
