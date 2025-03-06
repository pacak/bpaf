# Change Log

## bpaf [0.9.18] - 2025-03-06
- Several small documentation fixes (#414, #413)
  thanks @yerke
- `fallback_to_usage` only applies if parser fails (#415)
  Previously it would print a usage info even if parser can succeed without any input

## bpaf [0.9.17], bpaf_derive [0.5.17] - 2025-03-01
- A new method `format_fallback` allows to format fallback values using
  a custom formatting function. This extends functionality offered by `format_debug` and
  `format_display` that use `Debug` and `Display` instances respectively
  thanks @antalsz

## bpaf [0.9.16], 2025-01-24
- treat `pure` as an implicit consumer - don't add unnecessary `.optional()` or `.many()`
- unbrainfart one of the examples

## bpaf [0.9.15], 2024-10-08
- a fix for a previous fix of fish completions, again - regenerate the files

## bpaf [0.9.14], 2024-09-19
- add license files (#388)
  thanks @davide
- fix fish completions - you'll need to regenerate completion files for them to work

## bpaf [0.9.13], bpaf_derive [0.5.13] - 2024-09-06
- You can now use `fallback_to_usage` in derive macro for options and subcommands (#376)
- Bugfixes related to shell completion and file masks
  thanks @ozwaldorf
- `not_strict` restriction for positional items (TODO - check the docs)
  thanks @ozwaldorf
- more shell completion bugfixes (#384, #382, #381)
- `ParseFailure::print_mesage` (with one `s` is deprecated in favor of the right spelling


## bpaf [0.9.12] - 2024-04-29
- better error messages

## bpaf [0.9.11] - 2024-03-24
- better error messages

## bpaf [0.9.10], bpaf_derive [0.5.10] - 2024-03-19
- due to dependency change colored output for legacy version is no longer supported
- Added `OptionParser::max_width` and `#[bpaf(max_width(xxx))]` to specify maximum
  width of help output
- Added a custom path attribute to allow using of reexported bpaf
  thanks @bzm3r
- support anywhere in bpaf_derive
  thanks @vallentin
- small documentation improvements
  thanks @Boshen
- minor shell completion improvements
- avoid panic in case of hidden but required parser argument (#345)
- somewhat breaking - `ParseFailure::exit_code` is separated into
  `ParseFailure::exit_code` and `ParseFailure::print_message`, one produces
  exit code, one prints the message

## bpaf [0.9.9] - 2024-01-17
- fix formatting in ambiguity error message
- relax upper range on owo-colors

## bpaf [0.9.8] - 2023-12-06
- fix docs.rs build
- bump deps

## bpaf [0.9.7], bpaf_derive [0.5.7] - 2023-11-28
- updated documentation
- support for `#[bpaf(ignore_rustdoc)]`

## bpaf [0.9.6], bpaf_derive [0.5.6] - 2023-10-30
- make sure env-only arguments and flags are working
- support raw identifiers in derive macro (#282)
- better error messages for unexpected values that prevent positional parses
- bugfix in completions generator for bash
  thanks @akinomyoga
- `choice` combinator to efficiently construct alternative parsers at runtime

## bpaf [0.9.5], bpaf_derive [0.5.5] - 2023-08-24
- fancier squashing: parse `-abfoo` as `-a -b=foo` if b is a short argument
- `bpaf_derive`: make sure command aliases are actually working

## bpaf [0.9.4] - 2023-08-08
- add `help` to `ParseFlag` and `ParseArgument`
- stop deprecating `Parser::run`
- Lots of docs.rs documentation improvements
- changes to rendered markdown

## bpaf [0.9.3], bpaf_derive [0.5.3] - 2023-07-26
- `Parser::collect` allows to collect multiple items into an arbitrary `FromIterator` collection
- Bugfix in parsing for unit structs
- docs.rs documentation update

## bpaf [0.9.2], bpaf_derive [0.5.2] - 2023-07-13
- with `docgen` feature you can render documentation as markdown
- cosmetic changes to error messages

## bpaf [0.9.1], bpaf_derive [0.5.1] - 2023-07-05
- add a way to print usage when called with no argument_os
- since 0.9.0 bpaf splits help messages into "full" and "partial", displaying
  full only when `--help` flag is passed twice or when rendering the documentation,
  see https://docs.rs/bpaf/0.9.0/bpaf/parsers/struct.NamedArg.html#method.help
- `display_fallback` and `debug_fallback` now can be used with `fallback_with`
- regression fixes

## bpaf [0.9.0] - 2023-07-03
- more errors are now passed as ADTs rather than plain strings
- conflicts are now tracked with indices rather than parser meta
- documentation improvements
- better error messages
- smaller generated binary

### Breaking changes
- `bpaf_derive 0.5.0` comes with some breaking changes
-  documentation generation now comes under `docgen` feature instead of `manpage` and some
   things are renamed
-  standalone `command` function was deprecated in favor of `.command` method on `OptionParser`
-  hidden no-op helper type `FromUtf8` was removed
-  "{usage}" override is removed in favor of new `OptionParser::with_usage`


## bpaf_derive [0.5.0] - 2023-07-03

should be faster to compile and a bit more flexible with respect to what is accepted

### Braking changes
- explicit `construct` annotation is gone and used by default if `options` and `command` are
  missing
- `options` and `command` now must be specified at the beginning of `#[bpaf(...` macro
- `default` annotation for enum variants is gone, you can use `fallback` on top instead


## bpaf [0.8.1], bpaf_derive [0.4.1] - 2023-05-30
- combination of `command` and `hide` now works as expected in `bpaf_derive`

## bpaf [0.8.0], bpaf_derive [0.4.0] - 2023-05-10

### Breaking changes

- `any` now takes a function that checks if it matches the input or not.
  You can still apply usual filtering with `guard`, etc after it but initial
  filtering inside a function leads to better error messages.
- `anywhere` is now a method on `any` instead of being a parser method
  and should now be used to make an arbitrary looking flag like parsers.
  You can still parse blocks from arbitrary places using remaining `adjacent`
  method
- `many` and `some` will now collect one result from a parser
  that does not consume anything from an argument list allowing
  for easier composition with parsers that consume from both
  command line and environment variables. If your code depends on
  the original behavior you should replace non failing parsers under
  `many` with failing parsers: `req_flag` instead of `switch`.

### Improvements

- parsing combinators `many`, `some`, `optional` and `anywhere` will
  now propagate parsing errors outwards, you can regain the old
  behvior by specifying `catch`
- better error messages related to `anywhere` parsers
- `anywhere` parsers are now given an attempt to consume an empty list
- support deriving `req_flag` consumers
- support deriving `catch` annotation
- errors generated by `some` can now be handled with `fallback`/`fallback_with`
- fallback values can be made visible in `--help` with `display_fallback`/`debug_fallback`
- `bpaf_derive`: top level doc comments on a regular parser are now turned into a `group_help`
- better error messages for invalid user input
- env fallback can now be fully hidden
- meta description refactor - invididual parsers should be described more consistently
  in all sorts of messages
- better error messages with positionals and inside anywhere blocks

### Migration guide 0.7.x -> 0.8.x
1. if you used `any` to consume items without validations just pass `Some` as a parameter
   and add two wildcard generic type parameters:
    ```diff
    -let rest = any<OsString>("RESt").many();
    +let rest = any<OsString, _, _>("REST", Some);
    ```
2. If you used `any` with extra validation to decide if something should be consumed at all
   you can move this validation inside of any. If validation fails - any behaves as if this
   argument wasn't specified at all:
   ```diff
   -let name = any("NAME").guard(|x| x == "Bob", "Only Bob is allowed").optional().catch();
   +let name = any("NAME", |x| (x == "Bob").then_some(x));
   ```
3. You can replicate most of the behavior from old `anywhere` modifier with new `adjacent`:
   ```diff
   -let set = construct!(set, name, value).anywhere();
   +let set = construct!(set, name, value).adjacent();
   ```
4. If you previously used `switch` or an option with fallback in combination with `many`
   you need to replace `switch` with something that needs at least one item and move fallback
   outside:
   ```diff
   -let verbose = short('v').switch().many().map(|x| x.len());
   +let verbose = short('v').req_flag(()).many().map(|x| x.len());
   ```


## bpaf [0.7.10], bpaf_derive [0.3.5] - 2023-03-19
- improve error messages for typos like `-llvm` instead of `--llvm`
- improve error messages when a flag is accepted by a command but not directly
- allow to derive position bool
- derive anywhere and boxed
- dynamic layout for --help messages
- bump syn to 2.0

## bpaf [0.7.9], bpaf_derive [0.3.4] - 2023-02-14
- `ParseFailure::exit_code`
- A way to specify custom usage in derive macro

## bpaf [0.7.8] - 2023-01-01
- manpage generation bugfixes,
  thanks to @ysndr
- internal cleanups
- avoid impossible shell completions

## bpaf [0.7.7] - 2022-12-04
- manpage generation

## bpaf [0.7.6] - 2022-11-29
- fix docs.rs issues

## bpaf [0.7.5] - 2022-11-29
- improve error messages when several conflicting options are specified
- improve category theory docs
- improve docs for batteries

## bpaf [0.7.4], bpaf_derive [0.3.3] - 2022-11-19
- bpaf_derive: improve error message
- bpaf: bugfix for bash static shell completion

## bpaf [0.7.3] - 2022-11-14
- `try_run`
   thanks to @ysndr
- `-Obits=2048` is now parsed as short flag `O` with a value of `bits=2048` instad of crashing
- `complete_shell` - a way to call to static shell completion functions, bash and zsh only for
  now

## bpaf [0.7.2] - 2022-10-27
- drop tainting logic, should be redundant
- improve error messages for guard and conflicting branches

## bpaf [0.7.1] - 2022-10-15
- colors similar to cargo'some
   thanks to @kramer425
- support for empty structs/enums in `construct!`

## bpaf [0.7.0] - 2022-10-11
- `pure_with` implementation
   thanks to @xitep
- `FromOsStr` is replaced with magical uses of `Any` trait
- `hide_usage`
- `bright-color` and `dull-color` features
- accept fully qualified names in more places in `bpaf_derive`
- cosmetic improvements
- documentation improvements

# Migration guide 0.6.x -> 0.7.x
1. Remove FromUtf8 annotations if you have any
   ```diff
   -let coin = short('c').argument::<FromUtf8<Coin>>("COIN");
   +let coin = short('c').argument::<Coin>("COIN");
   ```
   In many cases rustc should be able to derive what the type
2. Replace `FromOsStr` implementations for your types with `FromStr`
   if you have any. If your type requires parsing `OsString` directly
   you can perform it in two steps - consuming `OsString` + parsing it
   with `Parser::parse`
3. If you want to provide your users with colored output - expose
   `bright-color` and/or `dull-color` features

## bpaf [0.6.1] - 2022-09-30
- cosmetic improvements
- completion info in `sensors` example
- better errors in partially consumed optional items
- better handling of -- during autcomplete
- initial release of `bpaf_cauwugo`

## bpaf [0.6.0] - 2022-09-22
# What's new in 0.6.0
- `adjacent` restriction to parse things in a tighter context
- `catch` for `many`, `some` and `optional` to handle parse errors
- `any` positional like, to consume pretty much anything from a command line
- improved documentation with more detailed examples
- cosmetic improvements
- a sneaky reminder to use `to_options` on `Parser` before trying to run it
- removed OsString specific `positional_os` and `argument_os`
- a way to make boxed parsers with single item `construct!` macro for making dynamic parsers
- a horrible way to reduce Yoda-talk coding by placing primitive definitions inside the `construct!` macro

With new additions you should be able to parse pretty much anything and then some more :)

# Migration guide 0.5.x -> 0.6.x

1. Replace any uses of `positional_os` and `argument_os` with `positional` and `argument` plus turbofish:
    ```diff
    -let file = positional_os("FILE").help("File to use").map(PathBuf::from);
    +let file = positional::<PathBuf>("FILE").help("File to use");
    ```
2. Replace any uses of `from_str` with either turbofish on the consumer or with `parse`, if `String` is generated inside the parser:
    ```diff
    -let number = short('n').argument("N").from_str::<usize>();
    +let number = short('n').argument::<usize>("N");
    ```
    You can still use it for your own types if you implement `FromOsStr`, alternatively `parse` still works:
    ```diff
    -let my = long("my-type").argument("MAGIC").from_str::<MyType>();
    +let my = long("my-type").argument::<String>("MAGIC").parse(|s| MyType::from_str(s));
    ```
3. You shouldn't be using those names directly in your code, but there are some renames
    - `Positional` -> `ParsePositional`
    - `BuildArgument` -> `ParseArgument`
    - `Command` -> `ParseCommand`
    - `Named` -> `NamedArg`

## bpaf [0.5.7] - 2022-09-04
- bugfix with zsh autocomplete #46
- reimplement bpaf derive - should be faster to compile and easier to work with

## bpaf [0.5.6] - 2022-09-03
- minor doc fixes
- bugfix for dynamic completion

## bpaf [0.5.5] - 2022-09-02
- invariant checker - for tests
- more error message improvements
- non-utf8 support in --foo=bar / -f=bar
- dynamic shell completion: bash, zsh, fish, elvish
- toggle flag battery
- templated usage string: can use "{usage}" in custom overrides

## bpaf [0.5.4] - 2022-08-25
minor bugfixes
- more consistent alternative selection
- handle "missing" inside a subparser

## bpaf [0.5.3] - 2022-08-23
and a bit more cosmetics - preserve suggestion context when returning from a subcommand

## bpaf [0.5.2] - 2022-08-23
- fix a regression in error messaged caused by 0.5.1
- guard now displays the problematic input if it's a single argument issue

## bpaf [0.5.1] - 2022-08-22
improve error messages if argument parsing fails:
  - matcher no longer escapes inner command if it gets there
  - detect and try to suggest for possible typos

## bpaf [0.5.0] - 2022-08-21
A big rewrite, performance should stay mostly unchanged, binary overhead
should be down by a third or so. Some minor cosmetic and correctness changes.
Documentation rewrite.

Migration guide:
1. add `use bpaf::Parser;` if you don't have it already - it is now a trait
   and needs to be in scope
2. replace `fn foo() -> Parser<T>`
   with `fn foo() -> impl Parser<T>`
3. replace `Info::default().descr("xxx").for_parser(parser)`
   with `parser.to_options().descr("xxx")`
4. replace `a.or_else(b).or_else(c)
   with `construct!([a, b, c])`
5. replace `command("foo", Some("bar"), subparser)`
   with `subparser.command("foo").help("bar")`

## bpaf [0.4.12] - 2022-08-08
- bpaf now depends on a specific version of bpaf_derive

## bpaf [0.4.11] - 2022-08-07
- meta and item refactors, changed the formatting a bit

## bpaf_derive [0.1.4] - 2022-08-07
- docs type: option -> optional
- metavar name for positional, positional_os, argument, argument_os is now optional
- bpaf_derive is now more strict about derive attribute parsing
- derived items can be made module private
- env is now supported

## bpaf [0.4.10] - 2022-08-03
- bugfix for custom usage formatting

## bpaf [0.4.9] - 2022-08-03
- bugfix for help rendering with fallback

## bpaf [0.4.8] - 2022-08-02
- support for env

## bpaf [0.4.7] - 2022-06-28
- support arbitrary long paths in construct! macro

## bpaf [0.4.5] - 2022-06-26
- use $crate:: inside construct to allow using it without importing

## bpaf [0.4.4] - 2022-06-04
- lower minimum supported rustc version to 1.56

## bpaf_derive [0.1.3] - 2022-04-25
- deriving for version should be working now

## bpaf [0.4.3]
- version now uses -V instead of -v

## bpaf_derive [0.1.2] - 2022-04-13
- support for version and version(expr) annotations on a top level
- improve help generation from doc comments

## bpaf [0.4.2] - 2022-04-10
- derive macro
- some takes an error message
- support for or_else construct!([alt1, alt2, alt3])

## [0.3.2] - 2022-03-18
- cargo_helper, hide + cosmetic bugfixes

## [0.3.1] - 2022-03-17
- parsers produced by functions inside construct!() macro

## [0.3.0] - 2022-03-15
- optimizations
- renamed Parser::help to Parser::group_help to reduce confusion

## [0.2.1] - 2022-03-14
- publish API for users to write tests for parsers

## [0.2.0] - 2022-03-10
### First public release
