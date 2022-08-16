#![doc = include_str!("../README.md")]
use quote::ToTokens;
use syn::parse_macro_input;

mod field;
mod kw;
mod top;
mod utils;
use top::Top;

#[proc_macro_derive(Bpaf, attributes(bpaf))]
pub fn derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Top).to_token_stream().into()
}
