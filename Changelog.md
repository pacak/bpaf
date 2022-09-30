# Change Log

## bpaf [0.6.1] - unreleased
- cosmetic improvements
- completion info in sensors example
- better errors in partially consumed optional items
- better handling of -- during autcomplete

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

