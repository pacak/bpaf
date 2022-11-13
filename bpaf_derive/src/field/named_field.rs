use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse2, parse_quote, token, Attribute, Expr, Ident, LitChar, LitStr, Path,
    PathArguments, PathSegment, Result, Token, Type,
};

use crate::utils::{to_kebab_case, LineIter};

use super::{
    as_long_name, as_short_name, parse_optional_arg, split_type, ConsumerAttr, Doc, Name,
    PostprAttr, Shape,
};

#[derive(Debug, Clone)]
pub struct Field {
    ty: Type,
    naming: Vec<Name>,
    env: Option<Box<Expr>>,
    consumer: Option<ConsumerAttr>,
    external: Option<Path>,
    name: Option<Ident>,
    postpr: Vec<PostprAttr>,
    help: Option<String>,
}

fn check_stage(prev: &mut usize, new: usize, keyword: &Ident) -> Result<()> {
    let stages = ["naming", "consumer", "external", "postprocessing"];
    if *prev > new {
        return Err(syn::Error::new(
            keyword.span(),
            format!(
                "{} is a {} can't follow previous stage ({})",
                keyword, stages[new], stages[*prev]
            ),
        ));
    }
    if new == 3 && *prev != 0 {
        return Err(syn::Error::new(
            keyword.span(),
            "Processing chain must start with external if external is present".to_owned(),
        ));
    }
    if *prev == 2 && new == 2 {
        return Err(syn::Error::new(
            keyword.span(),
            "You can have only one consumer".to_owned(),
        ));
    }
    *prev = new;
    Ok(())
}

fn parse_arg<T: Parse>(input: ParseStream) -> Result<T> {
    let content;
    let _ = parenthesized!(content in input);
    content.parse::<T>()
}

pub fn parse_opt_arg<T: Parse>(input: ParseStream) -> Result<Option<T>> {
    if input.peek(token::Paren) {
        let content;
        let _ = parenthesized!(content in input);
        Ok(Some(content.parse::<T>()?))
    } else {
        Ok(None)
    }
}

#[inline(never)]
pub fn parse_lit_char(input: ParseStream) -> Result<LitChar> {
    parse_arg(input)
}

#[inline(never)]
pub fn parse_lit_str(input: ParseStream) -> Result<LitStr> {
    parse_arg(input)
}

#[inline(never)]
pub fn parse_ident(input: ParseStream) -> Result<Ident> {
    parse_arg(input)
}

#[inline(never)]
pub fn parse_path(input: ParseStream) -> Result<Path> {
    parse_arg(input)
}

#[inline(never)]
pub fn parse_expr(input: ParseStream) -> Result<Box<Expr>> {
    Ok(Box::new(parse_arg(input)?))
}

impl Field {
    pub fn var_name(&self, ix: usize) -> Ident {
        let name = &self.name;
        match name {
            Some(name) => name.clone(),
            None => Ident::new(&format!("f{}", ix), Span::call_site()),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn make(ty: Type, name: Option<Ident>, attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = Field {
            ty,
            external: None,
            name,
            naming: Vec::new(),
            env: None,
            consumer: None,
            postpr: Vec::new(),
            help: None,
        };
        let mut help = Vec::new();

        let mut stage = 0;
        for attr in attrs {
            if attr.path.is_ident("doc") {
                help.push(parse2::<Doc>(attr.tokens)?.0);
            } else if attr.path.is_ident("bpaf") {
                #[allow(clippy::cognitive_complexity)]
                attr.parse_args_with(|input: ParseStream| loop {
                    if input.is_empty() {
                        break Ok(());
                    }
                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;
                    let span = keyword.span();

                    let content;

                    // naming
                    if keyword == "long" {
                        check_stage(&mut stage, 1, &keyword)?;
                        res.naming.push(if input.peek(token::Paren) {
                            let lit = parse_lit_str(input)?;
                            Name::Long(lit)
                        } else if let Some(name) = res.name.as_ref() {
                            Name::Long(as_long_name(name))
                        } else {
                            break Err(
                                input_copy.error("unnamed field needs to have a name specified")
                            );
                        });
                    } else if keyword == "short" {
                        check_stage(&mut stage, 1, &keyword)?;
                        res.naming.push(if input.peek(token::Paren) {
                            let lit = parse_lit_char(input)?;
                            Name::Short(lit)
                        } else if let Some(name) = res.name.as_ref() {
                            Name::Short(as_short_name(name))
                        } else {
                            break Err(
                                input_copy.error("unnamed field needs to have a name specified")
                            );
                        });
                    } else if keyword == "env" {
                        check_stage(&mut stage, 1, &keyword)?;

                        res.env = Some(parse_expr(input)?);
                    //
                    // consumer
                    } else if keyword == "argument" {
                        check_stage(&mut stage, 2, &keyword)?;
                        let ty = if input.peek(token::Colon) {
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Lt>()?;
                            let ty = input.parse::<Type>()?;
                            input.parse::<token::Gt>()?;
                            Some(Box::new(ty))
                        } else {
                            None
                        };
                        let arg = parse_optional_arg(input)?;
                        res.consumer = Some(ConsumerAttr::Arg(arg, ty));
                    } else if keyword == "positional" {
                        check_stage(&mut stage, 2, &keyword)?;
                        let ty = if input.peek(token::Colon) {
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Lt>()?;
                            let ty = input.parse::<Type>()?;
                            input.parse::<token::Gt>()?;
                            Some(Box::new(ty))
                        } else {
                            None
                        };
                        let arg = parse_optional_arg(input)?;

                        res.consumer = Some(ConsumerAttr::Pos(arg, ty));
                    } else if keyword == "any" {
                        check_stage(&mut stage, 2, &keyword)?;
                        let ty = if input.peek(token::Colon) {
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Colon>()?;
                            input.parse::<token::Lt>()?;
                            let ty = input.parse::<Type>()?;
                            input.parse::<token::Gt>()?;
                            Some(Box::new(ty))
                        } else {
                            None
                        };

                        let arg = parse_optional_arg(input)?;
                        res.consumer = Some(ConsumerAttr::Any(arg, ty));
                    } else if keyword == "switch" {
                        check_stage(&mut stage, 2, &keyword)?;
                        res.consumer = Some(ConsumerAttr::Switch);
                    } else if keyword == "flag" {
                        check_stage(&mut stage, 2, &keyword)?;
                        let _ = parenthesized!(content in input);
                        let a = content.parse()?;
                        content.parse::<token::Comma>()?;
                        let b = content.parse()?;
                        res.consumer = Some(ConsumerAttr::Flag(Box::new(a), Box::new(b)));
                    //
                    // external
                    } else if keyword == "external" {
                        check_stage(&mut stage, 3, &keyword)?;

                        if input.peek(token::Paren) {
                            res.external = Some(parse_path(input)?);
                        } else if let Some(name) = &res.name {
                            res.external = Some(ident_to_path(name.clone()));
                        } else {
                            break Err(
                                input_copy.error("unnamed fields needs to have a name specified")
                            );
                        }

                    //
                    // postpr
                    } else if keyword == "guard" {
                        check_stage(&mut stage, 4, &keyword)?;
                        let _ = parenthesized!(content in input);
                        let guard_fn = content.parse::<Path>()?;
                        let _ = content.parse::<Token![,]>()?;
                        let msg = content.parse::<Expr>()?;
                        res.postpr
                            .push(PostprAttr::Guard(span, guard_fn, Box::new(msg)));
                    } else if keyword == "fallback" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr
                            .push(PostprAttr::Fallback(span, parse_expr(input)?));
                    } else if keyword == "fallback_with" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr
                            .push(PostprAttr::FallbackWith(span, parse_expr(input)?));
                    } else if keyword == "parse" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Parse(span, parse_path(input)?));
                    } else if keyword == "map" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Map(span, parse_path(input)?));
                    } else if keyword == "complete" {
                        check_stage(&mut stage, 4, &keyword)?;
                        let f = parse_path(input)?;
                        res.postpr.push(PostprAttr::Complete(span, f));
                    } else if keyword == "complete_shell" {
                        check_stage(&mut stage, 4, &keyword)?;
                        let f = parse_expr(input)?;
                        res.postpr.push(PostprAttr::CompleteShell(span, f));
                    } else if keyword == "many" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Many(span, None));
                    } else if keyword == "some" {
                        check_stage(&mut stage, 4, &keyword)?;
                        let lit = parse_lit_str(input)?;
                        res.postpr.push(PostprAttr::Many(span, Some(lit)));
                    } else if keyword == "optional" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Optional(span));
                    } else if keyword == "hide" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Hide(span));
                    } else if keyword == "hide_usage" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::HideUsage(span));
                    } else if keyword == "catch" {
                        check_stage(&mut stage, 4, &keyword)?;
                        res.postpr.push(PostprAttr::Catch(span));
                    } else if keyword == "group_help" {
                        check_stage(&mut stage, 4, &keyword)?;
                        let expr = parse_expr(input)?;
                        res.postpr.push(PostprAttr::GroupHelp(span, expr));
                    } else {
                        return Err(input_copy.error("Unexpected attribute"));
                    }

                    if !input.is_empty() {
                        input.parse::<token::Comma>()?;
                    }
                })?;
            }
        }

        res.implicit_naming();
        res.implicit_consumer()?;
        res.help = LineIter::from(&help[..]).next();

        if !(res.external.is_some() || res.consumer.is_some()) {
            return Err(syn::Error::new(
                res.ty.span(),
                "Not sure how to parse this field, make a ticket?",
            ));
        }
        Ok(res)
    }
}

impl Field {
    fn implicit_naming(&mut self) {
        if self.external.is_some() || !self.naming.is_empty() {
            return;
        }

        // Do we even need to derive the name here?
        if let Some(cons) = &self.consumer {
            match cons {
                ConsumerAttr::Any(_, _) | ConsumerAttr::Pos(_, _) => return,
                ConsumerAttr::Arg(_, _)
                | ConsumerAttr::Switch
                | ConsumerAttr::Flag(_, _)
                | ConsumerAttr::ReqFlag(_) => {}
            }
        }

        let name = match &self.name {
            Some(name) => to_kebab_case(&name.to_string()),
            None => return,
        };

        self.naming.push(if name.chars().nth(1).is_some() {
            Name::Long(LitStr::new(&name, self.name.span()))
        } else {
            let c = name.chars().next().unwrap();
            Name::Short(LitChar::new(c, self.name.span()))
        });
    }

    fn implicit_consumer(&mut self) -> Result<()> {
        // external presumably deals with that
        if self.external.is_some() {
            return Ok(());
        }

        // refuse to derive a consumer unless it's already present
        // decide about deriving postprocessor
        let derive_postpr = if let Some(wrong) = self.postpr.iter().find(|i| !i.can_derive()) {
            if self.consumer.is_none() {
                return Err(syn::Error::new(
                    wrong.span(),
                    "Can't derive implicit consumer with this annotation present",
                ));
            }
            false
        } else {
            true
        };

        // pick inner type
        let ty = match split_type(&self.ty) {
            Shape::Optional(ty) => {
                if derive_postpr {
                    self.postpr.insert(0, PostprAttr::Optional(ty.span()));
                }
                ty
            }
            Shape::Multiple(ty) => {
                if derive_postpr {
                    self.postpr.insert(0, PostprAttr::Many(ty.span(), None));
                }
                ty
            }
            Shape::Bool => {
                if self.naming.is_empty() {
                    return Err(syn::Error::new(
                        self.ty.span(),
                        "Can't derive consumer for unnamed boolean field",
                    ));
                }
                if self.consumer.is_none() {
                    self.consumer = Some(ConsumerAttr::Switch);
                }
                return Ok(());
            }
            Shape::Direct(ty) => ty,
            Shape::Unit => {
                if self.consumer.is_none() {
                    self.consumer = Some(ConsumerAttr::ReqFlag(parse_quote!(())));
                    return Ok(());
                }
                self.ty.clone()
            }
        };

        if let Some(cons) = &self.consumer {
            match cons {
                ConsumerAttr::Any(l, None) => {
                    self.consumer = Some(ConsumerAttr::Any(l.clone(), Some(Box::new(ty.clone()))));
                }
                ConsumerAttr::Arg(l, None) => {
                    self.consumer = Some(ConsumerAttr::Arg(l.clone(), Some(Box::new(ty.clone()))));
                }
                ConsumerAttr::Pos(l, None) => {
                    self.consumer = Some(ConsumerAttr::Pos(l.clone(), Some(Box::new(ty.clone()))));
                }
                _ => {}
            }
        }

        match &self.consumer {
            Some(cons) => match cons {
                ConsumerAttr::Arg(..)
                | ConsumerAttr::Switch
                | ConsumerAttr::Flag(_, _)
                | ConsumerAttr::ReqFlag(_) => {
                    if self.naming.is_empty() {
                        return Err(syn::Error::new(
                            self.ty.span(),
                            "This consumer needs a name, you can specify it with long(\"name\") or short('n')",
                        ));
                    }
                }
                ConsumerAttr::Pos(..) | ConsumerAttr::Any(..) => {}
            },
            None => {
                let arg = LitStr::new("ARG", ty.span());
                if self.naming.is_empty() {
                    self.consumer = Some(ConsumerAttr::Pos(arg, Some(Box::new(ty))));
                } else {
                    self.consumer = Some(ConsumerAttr::Arg(arg, Some(Box::new(ty))));
                }
            }
        }

        Ok(())
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ext) = &self.external {
            quote!(#ext()).to_tokens(tokens);
        } else if !self.naming.is_empty() {
            let name = &self.naming[0];
            quote!(::bpaf::#name).to_tokens(tokens);
            for rest in &self.naming[1..] {
                quote!(.#rest).to_tokens(tokens);
            }
            if let Some(env) = &self.env {
                quote!(.env(#env)).to_tokens(tokens);
            }
            if let Some(help) = &self.help {
                quote!(.help(#help)).to_tokens(tokens);
            }
            if let Some(cons) = &self.consumer {
                quote!(.#cons).to_tokens(tokens);
            }
        } else if let Some(pos) = &self.consumer {
            quote!(::bpaf::#pos).to_tokens(tokens);
            if let Some(help) = &self.help {
                quote!(.help(#help)).to_tokens(tokens);
            }
        } else {
            unreachable!("implicit_consumer bug?");
        }

        for postpr in &self.postpr {
            quote!(.#postpr).to_tokens(tokens);
        }
    }
}

fn ident_to_path(ident: Ident) -> Path {
    Path {
        leading_colon: None,
        segments: vec![PathSegment {
            ident,
            arguments: PathArguments::None,
        }]
        .into_iter()
        .collect(),
    }
}
