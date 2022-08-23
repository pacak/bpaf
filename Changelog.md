# Change Log

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

