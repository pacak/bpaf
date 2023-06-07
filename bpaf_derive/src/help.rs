use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Expr;

#[derive(Debug)]
pub(crate) enum Help {
    Custom(Box<Expr>),
    Doc(String),
}

impl ToTokens for Help {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Help::Custom(c) => c.to_tokens(tokens),
            Help::Doc(d) => d.to_tokens(tokens),
        }
    }
}

impl From<&str> for Help {
    fn from(value: &str) -> Self {
        Help::Doc(value.to_string())
    }
}
