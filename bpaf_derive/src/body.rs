use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Error, Ident, braced, parenthesized, parse::ParseStream, token};

use crate::{
    Generate, Help,
    decls::{CMD, EXTERNAL_DECL, FLAG_MOD, LITERAL, NEST, SKIP, postpr_keep_type},
    fields, lookup_op,
    outer::{Trigger, parse_command, parse_construct, parse_nested},
    split_attrs,
    utils::snake_case_ident,
};

/// handles whole of struct or enum, delegating fields to separate parsers
pub(crate) fn parse_inner(input: ParseStream) -> syn::Result<(Ident, TokenStream)> {
    let lookahead = input.lookahead1();
    let content;
    if lookahead.peek(token::Struct) {
        input.parse::<token::Struct>()?;
        let ty = input.parse::<Ident>()?;
        let name = quote!(#ty);
        let inner = input.lookahead1();
        if inner.peek(token::Brace) {
            // `struct Foo { field: ty, ..}`
            braced!(content in input);
            if content.is_empty() {
                Ok((ty, quote!(::bpaf::pure(#name {}))))
            } else {
                Ok((ty, struct_or_variant(true, name, &content)?))
            }
        } else if inner.peek(token::Paren) {
            // `struct Foo (ty, ..);`
            parenthesized!(content in input);
            input.parse::<token::Semi>()?;
            if content.is_empty() {
                Ok((ty, quote!(::bpaf::pure(#name()))))
            } else {
                Ok((ty, struct_or_variant(false, name, &content)?))
            }
        } else if inner.peek(token::Semi) {
            // `struct Foo;`
            input.parse::<token::Semi>()?;
            Ok((ty, quote!(::bpaf::pure(#name))))
        } else {
            Err(inner.error())
        }
    } else if lookahead.peek(token::Enum) {
        input.parse::<token::Enum>()?;
        let ty = input.parse::<Ident>()?;
        braced!(content in input);
        let mut branches = Vec::new();
        while !content.is_empty() {
            use syn::parse::Parser as _;
            let (help, bpaf) = split_attrs(content.call(Attribute::parse_outer)?)?;
            (|input: ParseStream| {
                let skipped = lookup_op(&[SKIP], input)?.is_some();
                let body = parse_variant(help, input, &ty, &content)?;
                if !skipped {
                    branches.push(body);
                }
                Ok(())
            })
            .parse2(bpaf)?
        }
        if branches.is_empty() {
            Err(Error::new_spanned(ty, "enum with no branches?"))
        } else if branches.len() == 1 {
            Ok((ty, branches.remove(0)))
        } else {
            let span = Span::call_site();
            let mut make = TokenStream::new(); // declare variant
            let mut plop = TokenStream::new(); // combine variants together
            for (ix, var) in branches.iter().enumerate() {
                let name = Ident::new(&format!("alt{ix}"), span);
                make.extend(quote::quote!(let #name = #var; ));
                plop.extend(quote::quote!(#name,));
            }
            Ok((ty, quote::quote! {{ #make ::bpaf::construct!([ #plop ])}}))
        }
    } else {
        Err(lookahead.error())
    }
}

/// handles a non empty collection of named or unnamed fields
fn struct_or_variant(
    named: bool,
    name: TokenStream,
    input: ParseStream,
) -> syn::Result<TokenStream> {
    let mut calc = TokenStream::new();
    let mut plop = TokenStream::new();
    let mut cnt = 0;
    while !input.is_empty() {
        let (name, body) = if named {
            fields::parse_named(input)?
        } else {
            let name = Ident::new(&format!("f{cnt}"), Span::call_site());
            (name, fields::parse_unnamed(input)?)
        };
        cnt += 1;
        calc.extend(quote::quote! { let #name = #body; });
        plop.extend(quote::quote! { #name, });
    }
    let plop = if named {
        quote!({ #plop })
    } else {
        quote!(( #plop ))
    };
    Ok(quote! {{
        #calc
        ::bpaf::construct!(#name #plop)
    }})
}

/// Parser payload of an enum variant before a variant-level consumer applies
enum VariantPayload {
    /// `Enum::Variant`, `Enum::Variant {}` or `Enum::Variant()`: the fallback
    /// is a required flag
    Unit(TokenStream),
    /// Parser built from the variant fields, the fallback is `construct!(..)`
    Fields(TokenStream),
}

impl VariantPayload {
    /// Parser to feed into a variant-level `command`/`nest`/`literal`
    fn into_parser(self) -> TokenStream {
        match self {
            VariantPayload::Unit(tag) => quote!(::bpaf::pure(#tag)),
            VariantPayload::Fields(body) => body,
        }
    }
}

/// Parse one enum variant together with its `#[bpaf(...)]` and doc attributes
/// and emit its token stream right away. Returns `None` for skipped variants.
fn parse_variant(
    doc: Vec<String>,
    bpaf: ParseStream,
    enum_name: &Ident,
    input: ParseStream,
) -> syn::Result<TokenStream> {
    let ident = input.parse::<Ident>()?;
    let name = quote::quote!(#enum_name::#ident);
    let content;
    let (body, unit) = if input.peek(token::Brace) {
        braced!(content in input);
        if content.is_empty() {
            (TokenStream::new(), Some(quote!(#name {})))
        } else {
            (struct_or_variant(true, name, &content)?, None)
        }
    } else if input.peek(token::Paren) {
        parenthesized!(content in input);
        // TODO: here I can check if content is a single variant
        //       with no annotations, so those two invovations are equivalent
        //
        // #[bpaf(short('x'))
        // Foo(String)
        //
        // Foo(
        //     #[bpaf(short('x'))]
        //     String
        // )
        if content.is_empty() {
            (TokenStream::new(), Some(quote!(#name())))
        } else {
            (struct_or_variant(false, name, &content)?, None)
        }
    } else if input.peek(token::Comma) || input.is_empty() {
        (TokenStream::new(), Some(name))
    } else {
        return Err(input.error("Unexpected token"));
    };

    // trailing comma or variant separator
    input.parse::<Option<token::Comma>>()?;

    let payload = if let Some(unit) = unit {
        VariantPayload::Unit(unit)
    } else if bpaf.is_empty() {
        return Ok(body);
    } else {
        VariantPayload::Fields(body)
    };

    let genr = Generate {
        vis: None,
        name: ident.clone(),
    };
    if let Some(ext) = lookup_op(EXTERNAL_DECL, bpaf)? {
        let fname = snake_case_ident(&ident);
        Ok(ext.unwrap_or_else(|| quote!(#fname())))
    } else if let Some(cmd) = lookup_op(&[CMD], bpaf)? {
        let inner = payload.into_parser();
        Ok(parse_command(doc, genr, cmd, &ident, inner, bpaf)?.expr())
    } else if lookup_op(&[NEST], bpaf)?.is_some() {
        let inner = payload.into_parser();
        Ok(parse_nested(doc, genr, &ident, Trigger::Named, inner, bpaf)?.expr())
    } else if let Some(lit) = lookup_op(&[LITERAL], bpaf)? {
        let inner = payload.into_parser();
        let trigger = Trigger::Literal(lit);
        Ok(parse_nested(doc, genr, &ident, trigger, inner, bpaf)?.expr())
    } else {
        match payload {
            // nothing, `short`/`long`/`env`: a required flag
            VariantPayload::Unit(tag) => {
                let (mut body, has_name) = fields::parse_names(Some(&ident), bpaf)?;
                fields::default_name(&mut body, has_name, Some(&ident), bpaf.span())?;
                body.extend(quote!(.req_flag(#tag)));
                let mut tail = TokenStream::new();
                while let Some(ts) = lookup_op(FLAG_MOD, bpaf)? {
                    tail.extend(ts);
                }
                let mut help = Help::help(doc);
                while postpr_keep_type(bpaf, &mut tail)? || help.lookup(bpaf)? {}
                Ok(quote!(#body #help #tail))
            }
            VariantPayload::Fields(body) => {
                Ok(parse_construct(doc, genr, ident.clone(), body, bpaf)?.expr())
            }
        }
    }
}
