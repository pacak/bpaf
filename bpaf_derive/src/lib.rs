//! # Derive macro for bpaf command line parser
//!
//! For documentation refer to `bpaf` library docs: <https://docs.rs/bpaf/latest/bpaf/>

mod attr;
mod decls;
mod field;
mod fields;
mod outer;
mod postpr;
mod utils;

#[cfg(test)]
mod body_tests;
#[cfg(test)]
mod field_tests;

mod help;

mod body;

use crate::decls::{Generate, NAME_DECL, OPTION_PARSER};
use crate::fields::{Name, push, resolve_name};
use crate::help::{Help, Help3};
use crate::outer::parse_outer;
use crate::utils::doc_comment;
use crate::utils::ident_to_long;
use proc_macro2::TokenStream;
use syn::{Attribute, Error};

use crate::attr::{Decl, lookup_op};

/// Derive macro for bpaf command line parser
///
/// For documentation refer to bpaf library: <https://docs.rs/bpaf/latest/bpaf/derive.Bpaf.html>
#[proc_macro_derive(Bpaf, attributes(bpaf))]
pub fn derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::Parser::parse(parse_outer, input) {
        Ok(v) => v.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
pub(crate) fn derive_bpaf(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    syn::parse::Parser::parse2(parse_outer, input)
}

pub(crate) fn split_attrs(attrs: Vec<Attribute>) -> syn::Result<(Vec<String>, TokenStream)> {
    let mut help = Vec::new();
    let mut bpaf = TokenStream::new();
    for attr in attrs {
        if attr.path().is_ident("bpaf") {
            let syn::Meta::List(inner) = attr.meta else {
                let error = Error::new_spanned(attr, "Expected `#[bpaf(...)]`");
                return Err(error);
            };
            if inner.tokens.is_empty() {
                continue; // empty #[bpaf()] block?
            }
            if !bpaf.is_empty() {
                bpaf.extend(quote::quote!(,));
            }
            bpaf.extend(inner.tokens);
        } else if attr.path().is_ident("doc") {
            help.extend(doc_comment(&attr));
        }
    }
    Ok((help, bpaf))
}
