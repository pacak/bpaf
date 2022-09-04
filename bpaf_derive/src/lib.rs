#![doc = include_str!("../README.md")]
use quote::ToTokens;
use syn::parse_macro_input;

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
