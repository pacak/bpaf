//! # Questions asked at least once

//! ## Positionals with special meaning
//! **Q:** I'm trying to parse a structure that more or less looks like this:
//!
//! ```rust
//! enum Mode {
//!     Foo,
//!     Bar { baz: String }
//! }
//! ```
//!
//! Ideally, `app foo` parses to `Mode::Foo` and `app bar quux` parses to `Mode::Bar { baz: "quux" }`.
//!
//! I've tried combining positional parsers and even implementing my own, but I'm pretty sure I'm
//! barking up the wrong tree. Is there a relatively simple, canonical way of doing this with `bpaf`?
//!
//! **A:** Positional with a special meaning that changes the meaning of subsequent parsers is usually a
//! [`command`](OptionParser::command) and you combine them with `construct!([foo, bar])`.
//!
//! See a complete example here: <https://github.com/pacak/bpaf/blob/master/examples/git.rs>

#[allow(unused_imports)]
use crate::*;
