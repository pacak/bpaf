use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::{parenthesized, parse2, token, Attribute, Expr, Ident, LitChar, LitStr, Result};

use crate::field::{as_long_name, as_short_name, fill_in_name, ConstrName, Doc, StrictNameAttr};
use crate::utils::LineIter;

#[derive(Debug)]
pub struct ReqFlag {
    value: ConstrName,
    naming: Vec<StrictNameAttr>,
    help: Option<String>,
    is_hidden: bool,
    is_default: bool,
}

impl ReqFlag {
    pub fn make(value: ConstrName, attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = ReqFlag {
            value,
            naming: Vec::new(),
            help: None,
            is_hidden: false,
            is_default: false,
        };
        let mut help = Vec::new();
        for attr in attrs {
            if attr.path.is_ident("doc") {
                help.push(parse2::<Doc>(attr.tokens)?.0);
            } else if attr.path.is_ident("bpaf") {
                attr.parse_args_with(|input: ParseStream| loop {
                    if input.is_empty() {
                        break Ok(());
                    }
                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;

                    let content;
                    if keyword == "long" {
                        res.naming.push(if input.peek(token::Paren) {
                            let _ = parenthesized!(content in input);
                            StrictNameAttr::Long(content.parse::<LitStr>()?)
                        } else {
                            StrictNameAttr::Long(as_long_name(&res.value))
                        })
                    } else if keyword == "short" {
                        res.naming.push(if input.peek(token::Paren) {
                            let _ = parenthesized!(content in input);
                            StrictNameAttr::Short(content.parse::<LitChar>()?)
                        } else {
                            StrictNameAttr::Short(as_short_name(&res.value))
                        })
                    } else if keyword == "env" {
                        let _ = parenthesized!(content in input);
                        let env = Box::new(content.parse::<Expr>()?);
                        res.naming.push(StrictNameAttr::Env(env));
                    } else if keyword == "hide" {
                        res.is_hidden = true;
                    } else if keyword == "default" {
                        res.is_default = true;
                    } else {
                        break Err(
                            input_copy.error("Not a valid enum singleton constructor attribute")
                        );
                    };
                    if !input.is_empty() {
                        input.parse::<token::Comma>()?;
                    }
                })?;
            } else {
                unreachable!("Shouldn't get any attributes other than bpaf and doc")
            }
        }
        res.help = LineIter::from(&help[..]).next();
        fill_in_name(&res.value, &mut res.naming);
        Ok(res)
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
