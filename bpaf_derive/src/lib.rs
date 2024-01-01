//! # Derive macro for bpaf command line parser
//!
//! For documentation refer to `bpaf` library docs: <https://docs.rs/bpaf/latest/bpaf/>

mod attrs;
mod field;
mod named_field;
mod top;
mod utils;

#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod top_tests;

mod help;

mod td;
mod custom_path;

use top::Top;

/// Derive macro for bpaf command line parser
///
/// For documentation refer to bpaf library: <https://docs.rs/bpaf/latest/bpaf/>
#[proc_macro_derive(Bpaf, attributes(bpaf))]
pub fn derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote::ToTokens::to_token_stream(&syn::parse_macro_input!(input as Top)).into()
}
