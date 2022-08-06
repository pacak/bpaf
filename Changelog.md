# Change Log

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

