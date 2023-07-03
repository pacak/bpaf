//! Documentation generation system
//!
//! # Documentation fragments to use inside `--help` messages
//!
//! `bpaf` tries to use semantic approach to documentation generation, instead of describing what
//! color specific string slice should be you need to specify what this string slice supposed to
//! mean.
//!
//! Most of the help related functions take `Into<Doc>` parameters, normally you would pass one of
//! following things:
//!
//! 1. Ready made `Doc` - usually with combinatoric API
//! ```ignore
//! # use bpaf::doc::Doc;
//! let mut doc = Doc::default();
//! doc.emphasis("Usage: ");
//! doc.literal("my_program");
//! // do something with it
//! drop(doc)
//! ```
//! 2. A string slice - `&str` can be converted into a fully plain text `Doc` which is enough
//!    for most applications
//!
//! 3. A slice of style value pairs
#![cfg_attr(not(doctest), doc = include_str!("docs2/help.md"))]
//!
//! 4. A structure from your own crate that can be converted into `Doc`
//!
//! # Command line parser documentation generation
//!
//! [`OptionParser`] implements two methods: [`render_html`](OptionParser::render_html) and
//! [`render_manpage`](OptionParser::render_manpage) that create a documentation in a mix of
//! html/markdown and ROFF formats respectively.
//!
//! To use it you should do something like this
//! ```ignore
//! #[test]
//! fn update_doc() {
//!     let options = options();
//!     let html = options.render_html("app_name");
//!     let roff = options.render_manpage("app_name", Section::General, None, None, None);
//!     // then save those docs into a files
//!     // If you commit those docs into your repo and optionally fail a test if there
//!     // are changes - CI will ensure that documentation is always up to date
//! }
//! ```

#[doc(inline)]
pub use crate::buffer::{Doc, MetaInfo, Section, Style};

#[cfg(doc)]
use crate::*;
