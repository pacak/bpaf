use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Expr;

use crate::field::{as_long_name, as_short_name, ConstrName, Name};
use crate::top::Inner;

#[derive(Debug)]
pub struct ReqFlag {
    value: ConstrName,
    naming: Vec<Name>,
    env: Vec<Expr>,
    help: Option<String>,
    is_hidden: bool,
    is_default: bool,
}

impl ReqFlag {
    pub fn make(value: ConstrName, mut inner: Inner) -> Self {
        if inner.longs.is_empty() && inner.shorts.is_empty() {
            if value.constr.to_string().chars().nth(1).is_some() {
                inner.longs.push(as_long_name(&value.constr));
            } else {
                inner.shorts.push(as_short_name(&value.constr));
            }
        }

        let longs = inner.longs.into_iter().map(Name::Long);
        let short = inner.shorts.into_iter().map(Name::Short);
        ReqFlag {
            value,
            naming: longs.chain(short).collect::<Vec<_>>(),
            env: inner.envs,
            help: inner.help.first().cloned(), // TODO
            is_hidden: inner.is_hidden,
            is_default: inner.is_default,
        }
    }
}

impl ToTokens for ReqFlag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut first = true;
        for naming in &self.naming {
            if first {
                quote!(::bpaf::).to_tokens(tokens);
            } else {
                quote!(.).to_tokens(tokens);
            }
            naming.to_tokens(tokens);
            first = false;
        }
        for env in &self.env {
            quote!(.env(#env)).to_tokens(tokens);
        }
        if let Some(help) = &self.help {
            // help only makes sense for named things
            if !first {
                quote!(.help(#help)).to_tokens(tokens);
            }
        }
        let value = &self.value;

        if self.is_default {
            quote!(.flag(#value, #value)).to_tokens(tokens);
        } else {
            quote!(.req_flag(#value)).to_tokens(tokens);
        }
        if self.is_hidden {
            quote!(.hide()).to_tokens(tokens);
        }
    }
}
