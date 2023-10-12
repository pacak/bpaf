//! # Testing your parsers
//!
//! You can test values your parser produces and expected output
//!
//! ````rust
//! # use bpaf::*;
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)]
//! pub struct Options {
//!     pub user: String
//! }
//!
//! #[test]
//! fn generated_help_looks_correct() {
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
//! fn user_parser_works() {
//!     let value = options()
//!          .run_inner(&["--user", "Bob"])
//!          .unwrap();
//!     assert_eq!(value.user, "Bob");
//! }
//! ````
//!
//! [`OptionParser::run_inner`](OptionParser::run_inner) takes [`Args`](Args) or anything that can be converted to it, in most
//! cases using a static slice with static string slices (something like `&["--name", "value"]`)
//! is enough.
//!
//! Easiest way to consume [`ParseFailure`](ParseFailure) for testing purposes is with
//! [`ParseFailure::unwrap_stderr`](ParseFailure::unwrap_stderr) and [`ParseFailure::unwrap_stdout`](ParseFailure::unwrap_stdout) - result will lack any colors
//! even with them enabled: this makes testing easier.
//!
//! Successful result parse produces a value, "failed" parse produces stdout or stderr outputs -
//! stdout to print help message or version number and stderr to print the error message.
#[allow(unused_imports)] use crate::{*, parsers::*};
