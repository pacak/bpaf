//! # Using the library in derive style

//! # About examples
//!
//! Most of the examples omit adding doc comments to the fields, to keep things clearer, you should do
//! that when possible for better enduser experience. Similarly examples define [`Parser`] instead
//! of [`OptionParser`] - to be able to run them you need to use `[bpaf(options)]` annotations on
//! the most outer structure.
//!
//! ```rust
//! # use bpaf::*;
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)] // <- important bit
//! struct Config {
//!     /// number used by the program
//!     number: u32,
//! }
//! ```
//!
//! In addition to examples in the documentation there's a bunch more in the github repository:
//! <https://github.com/pacak/bpaf/tree/master/examples>

//! # Recommended reading order
//!
//! Combinatoric and derive APIs share the documentation and most of the names, recommended reading order:
//! 1. [`construct!`] - what combinations are
//! 2. [`Named`], [`positional`] and [`command`] - on consuming data
//! 3. [`Parser`] - on transforming the data
//! 4. [`OptionParser`] - on running the result

//! # Getting started
//!
//! 1. To use derive style API you need to enable `"derive"` feature for `bpaf`, **by default it's not
//!    enabled**.
//!
//! 2. Define primitive parsers if you want to use any. While it's possible to define most of them
//!    in derive style - doing complex parsing or validation is often easier in combinatoric style
//!
//! 3. Define types used to derive parsers, structs correspond to *AND* combination and require for
//!    all the fields to have a value, enums to *OR* combinations and require (and consume) all the
//!    values for one branch only.
//!
//! 4. Add annotations to the top level of a struct if needed, there's several to choose from and
//!    you can specify several of them. For this annotation ordering doesn't matter.
//!
//!    - ### Generated function name: `generate`
//!
//!      Unlike usual derive macro `bpaf_derive` generates a function with a name
//!      derived from a struct name by transforming it from `CamelCase` to `snake_case`. `generate`
//!      annotation allows to override a name for the function
//!
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      #[bpaf(generate(make_config))] // function name is now make_config()
//!      pub struct Config {
//!          pub flag: bool
//!      }
//!      ```
//!
//!    - ### Generated function visibility: `private`
//!
//!      By default `bpaf` uses the same visibility as the datatype,
//!      `private` makes it module private:
//!
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      #[bpaf(private)] // config() is now private
//!      pub struct Config {
//!          pub flag: bool
//!      }
//!      ```
//!
//!    - ### Generated function types: `command`, `options`
//!
//!      By default `bpaf_derive` would generate a function that generates a regular [`Parser`]
//!      it's possible instead to turn it into a
//!      [`command`] with `command` annotation or into a top level [`OptionParser`] with `options`
//!      annotation.
//!      Those annotations are mutually exclusive. `options` annotation takes an optional argument
//!      to use for [`cargo_helper`], `command` annotation takes an optional argument to
//!      override a command name.
//!
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Flag { // impl Parser by default
//!          pub flag: bool
//!      }
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      #[bpaf(command)]
//!      pub struct Make { // generates a command "make"
//!          pub level: u32,
//!      }
//!
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      #[bpaf(options)] // config() is now OptionParser
//!      pub struct Config {
//!          pub flag: bool
//!      }
//!      ```
//!
//!    - ### Specify version for generated command: `version`
//!
//!      By default `bpaf_derive` embedds no version information. With `version` with no argument
//!      results in using version from `CARGO_PKG_VERSION` env variable (specified by cargo on
//!      compile time, usually originates from `Cargo.toml`), `version` with argument results in
//!      using tht specific version - can be string literal or static string expression.
//!      Only makes sense for `command` and `options` annotations. For more information see
//!      [`version`](OptionParser::version).
//!
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      #[bpaf(options, version("3.1415"))] // --version is now 3.1415
//!      pub struct Config {
//!          pub flag: bool
//!      }
//!      ```
//!
//! 5. Add annotations to individual fields. Structure for annotation for individual fields
//!    is similar to how you would write the same code with combinatoric API with exception
//!    of `external` and usually looks something like this:
//!
//!    `((<naming> <consumer>) | <external>) <postprocessing>`
//!
//!    - `naming` section corresponds to [`short`],  [`long`] and [`env`](env()). `short` takes an optional
//!      character literal as a parameter, `long` takes an optional string, `env` takes an
//!      expression of type `&'static str` as a parameter - could be a string literal or a
//!      constant.
//!
//!      + If parameter for `short`/`long` is parameter isn't present it's derived from the field
//!      name: first character and a whole name respectively.
//!
//!      + If either of `short` or `long` is present - `bpaf_derive` would not add the other one.
//!
//!      + If neither is present - `bpaf_derive` would add a `long` one.
//!
//!      ```rust
//!      # use bpaf::*;
//!      const DB: &str = "top_secret_database";
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Config {
//!         pub flag_1: bool,     // no annotation: --flag_1
//!
//!         #[bpaf(short)]
//!         pub flag_2: bool,     // explicit short suppresses long: -f
//!
//!         #[bpaf(short('z'))]
//!         pub flag_3: bool,     // explicit short with custom letter: -z
//!
//!         #[bpaf(short, long)]
//!         pub deposit: bool,    // explicit short and long: -d --deposit
//!
//!         #[bpaf(env(DB))]
//!         pub database: String, // --database + env variable from DB constant
//!
//!         #[bpaf(env("USER"))]  // --user + env variable "USER"
//!         pub user: String,
//!      }
//!      ```
//!
//!    - `consumer` section corresponds to [`argument`](Named::argument), [`positional`],
//!      [`flag`](Named::flag), [`switch`](Named::switch) and similar.
//!
//!      + With no consumer annotations tuple structs (`struct Config(String)`) are usually parsed
//!      as positional items, but it's possible to override it by giving it a name:
//!
//!      ```rust
//!      # use bpaf::*;
//!      # use std::path::PathBuf;
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      struct Opt(PathBuf); // stays positional
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      struct Config(#[bpaf(long("input"))] PathBuf); // turns into a named argument
//!      ```
//!
//!      + `bpaf_derive` handles fields of type `Option<Foo>` and `Vec<Foo>` with something
//!      that can consume possibly one or many items with [`optional`](Parser::optional)
//!      and [`many`](Parser::many) respectively, see `postprocessing` for more details.
//!
//!      + `bpaf_derive` handles `bool` fields with [`switch`](Named::switch),
//!      [`OsString`](std::ffi::OsString) and [`PathBuf`](std::path::PathBuf) with
//!      either [`positional_os`] or [`argument_os`](Named::argument_os).
//!
//!      + `bpaf_derive` consumes everything else as [`String`] with [`positional`] and
//!      [`argument`](Named::argument) and transforms it into a concrete type using
//!      [`FromStr`](std::str::FromStr) instance.
//!      See documentation for corresponding consumers for more details.
//!
//!    - If `external` is present - it usually serves function of `naming` and `consumer`, allowing
//!      more for `postprocessing` annotations after it. Takes an optional parameter - a function
//!      name to call, if not present - `bpaf_derive` uses field name for this purpose.
//!      Functions should return impl [`Parser`] and you can either declare them manually
//!      or derive with `Bpaf` macro.
//!
//!      ```rust
//!      # use bpaf::*;
//!      fn verbosity() -> impl Parser<usize> {
//!          short('v')
//!              .help("vebosity, can specify multiple times")
//!              .req_flag(())
//!              .many()
//!              .map(|x| x.len())
//!      }
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Username {
//!          pub user: String
//!      }
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Config {
//!         #[bpaf(external)]
//!         pub verbosity: usize,      // implicit name - "verbosity"
//!
//!         #[bpaf(external(username))]
//!         pub custom_user: Username, // explicit name - "username"
//!      }
//!      ```
//!
//!    - `postprocessing` - various methods from [`Parser`] trait, order matters, most of them are
//!      taken literal, see documentation for the trait for more details. `bpaf_derive` automatically
//!      uses [`many`](Parser::many) and [`optional`](Parser::optional) to handle `Vec<T>` and
//!      `Option<T>` fields respectively and inserts [`from_str`](Parser::from_str) for any field
//!      it doesn't know how to pricess.
//!
//!      Any operation that can change the type (such as [`parse`](Parser::parse) or [`map`](Parser::map))
//!      for disables this logic for the field and also requires to specify the consumer:
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      struct Options {
//!          // #[bpaf(argument("NUM"), many)] - fails due to type mismatch
//!          // #[bpaf(from_str(u32), many)] - fails due to missing consumer
//!          #[bpaf(argument("NUM"), from_str(u32), many)]
//!          numbers: Vec<u32>
//!      }
//!      ```
//!
//!    - field-less enum variants obey slightly different set of rules, see
//!    [`req_flag`](Named::req_flag) for more details.
//!
//!    - any constructor in `enum` can have `skip` annotation - bpaf_derive
//!      would ignore them when generating code:
//!      ```rust
//!      # use bpaf::*;
//!      #[derive(Debug, Clone, Bpaf)]
//!      enum Decision {
//!          Yes,
//!          No,
//!          #[bpaf(skip)]
//!          Maybe
//!      }
//!
//!      ```
//!
//! 6. Add documentation for help messages.
//!    `bpaf_derive` generates help messages from doc comments, it skips single empty lines and stops
//!    processing after double empty line:
//!
//!    ```rust
//!    # use bpaf::*;
//!    #[derive(Debug, Clone, Bpaf)]
//!    pub struct Username {
//!        /// this is a part of a help message
//!        ///
//!        /// so is this
//!        ///
//!        ///
//!        /// but this isn't
//!        pub user: String
//!    }
//!    ```
//!
//! 7. Add [`check_invariants`](OptionParser::check_invariants) to your test code.

#[allow(unused_imports)]
use crate::*;
