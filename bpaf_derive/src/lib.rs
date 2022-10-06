//! # Derive macro for bpaf command line parser
//!
//! For documentation refer to `bpaf` library docs: <https://docs.rs/bpaf/latest/bpaf/>

#![allow(clippy::manual_range_contains)]
#![allow(clippy::single_match_else)]
use quote::ToTokens;
use syn::parse_macro_input;
extern crate proc_macro;

mod field;
#[cfg(test)]
mod field_tests;
mod top;
mod utils;
use top::Top;
#[cfg(test)]
mod top_tests;

/// Derive macro for bpaf command line parser
///
/// For documentation refer to bpaf library: <https://docs.rs/bpaf/latest/bpaf/>
#[proc_macro_derive(Bpaf, attributes(bpaf))]
pub fn derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Top).to_token_stream().into()
}
