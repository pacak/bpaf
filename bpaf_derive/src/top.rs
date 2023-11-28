use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token, Attribute, Error, Expr, Ident, LitChar, LitStr, Result, Visibility,
};

use crate::{
    attrs::{parse_bpaf_doc_attrs, EnumPrefix, PostDecor, StrictName},
    field::StructField,
    help::Help,
    td::{EAttr, Ed, Mode, TopAttr, TopInfo},
    utils::{to_kebab_case, to_snake_case, LineIter},
};

#[derive(Debug)]
/// Top level container
pub(crate) struct Top {
    // {{{
    /// Name of a parsed or produced type, possibly with generics
    ty: Ident,

    /// Visibility, derived from the type visibility or top level annotation
    vis: Visibility,

    /// Name of a generated function, usually derived from type,
    /// but can be specified with generate
    generate: Ident,

    /// single branch or multipe branches for enum
    body: Body,
    mode: Mode,
    boxed: bool,
    cargo_helper: Option<LitStr>,
    attrs: Vec<TopAttr>,
}

fn ident_to_long(ident: &Ident) -> LitStr {
    LitStr::new(&to_kebab_case(&ident.to_string()), ident.span())
}

fn ident_to_short(ident: &Ident) -> LitChar {
    LitChar::new(
        to_kebab_case(&ident.to_string()).chars().next().unwrap(),
        ident.span(),
    )
}

impl Parse for Top {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let (top_decor, mut help) = parse_bpaf_doc_attrs::<TopInfo>(&attrs)?;
        let TopInfo {
            private,
            custom_name,
            boxed,
            mode,
            attrs: td_attrs,
            ignore_rustdoc,
        } = top_decor.unwrap_or_default();

        if ignore_rustdoc {
            help = None;
        }
        let vis = input.parse::<Visibility>()?;

        let mut body = Body::parse(input)?;
        let ty = body.ty();
        let mut cargo_helper = None;
        let mut attrs = Vec::with_capacity(td_attrs.len());
        let mut options_ix = None;
        for attr in td_attrs {
            match attr {
                TopAttr::CargoHelper(h) => {
                    cargo_helper = Some(h);
                }
                TopAttr::ToOptions => {
                    options_ix = Some(attrs.len());
                    attrs.push(attr);

                    if let Some(h) = std::mem::take(&mut help) {
                        split_help_into(h, &mut attrs);
                    }
                }
                TopAttr::Usage(_)
                | TopAttr::Version(_)
                | TopAttr::Header(_)
                | TopAttr::Footer(_)
                | TopAttr::Adjacent
                | TopAttr::CommandShort(_)
                | TopAttr::CommandLong(_) => {
                    attrs.push(attr);
                }
                TopAttr::NamedCommand(ref n) => {
                    body.set_named_command(n.span())?;
                    attrs.push(TopAttr::ToOptions);

                    if let Some(h) = std::mem::take(&mut help) {
                        split_help_into(h, &mut attrs);
                    }

                    attrs.push(attr);
                }
                TopAttr::UnnamedCommand => {
                    body.set_unnamed_command();
                    attrs.push(TopAttr::ToOptions);

                    if let Some(h) = std::mem::take(&mut help) {
                        attrs.push(TopAttr::Descr(h));
                    }
                    attrs.push(TopAttr::NamedCommand(ident_to_long(&ty)));
                }

                TopAttr::Descr(_) => unreachable!(),
                TopAttr::PostDecor(_) => {
                    if let Some(ix) = options_ix {
                        attrs.insert(ix, attr);
                    } else {
                        attrs.push(attr);
                    }
                }
            }
        }

        if let Some(h) = std::mem::take(&mut help) {
            let doc = match h {
                Help::Custom(c) => c,
                Help::Doc(d) => parse_quote!(#d),
            };
            let span = Span::call_site();
            attrs.push(TopAttr::PostDecor(PostDecor::GroupHelp { doc, span }));
        }

        Ok(Top {
            vis: if private { Visibility::Inherited } else { vis },
            mode,

            generate: custom_name
                .unwrap_or_else(|| Ident::new(&to_snake_case(&ty.to_string()), ty.span())),
            ty,
            attrs,
            body,
            boxed,
            cargo_helper,
        })
    }
}

fn split_help_into(h: Help, attrs: &mut Vec<TopAttr>) {
    match &h {
        Help::Custom(_) => attrs.push(TopAttr::Descr(h)),
        Help::Doc(c) => {
            let mut chunks = LineIter::from(c.as_str());
            if let Some(s) = chunks.next() {
                attrs.push(TopAttr::Descr(Help::Doc(s)));
            }
            if let Some(s) = chunks.next() {
                if !s.is_empty() {
                    attrs.push(TopAttr::Header(Help::Doc(s)));
                }
            }
            if let Some(s) = chunks.rest() {
                attrs.push(TopAttr::Footer(Help::Doc(s)));
            }
        }
    }
}

fn split_ehelp_into(h: Help, opts_at: usize, attrs: &mut Vec<EAttr>) {
    match &h {
        Help::Custom(_) => attrs.push(EAttr::Descr(h)),
        Help::Doc(c) => {
            let mut chunks = LineIter::from(c.as_str());
            if let Some(s) = chunks.next() {
                attrs.insert(opts_at, EAttr::Descr(Help::Doc(s)));
            }
            if let Some(s) = chunks.next() {
                if !s.is_empty() {
                    attrs.insert(opts_at, EAttr::Header(Help::Doc(s)));
                }
            }
            if let Some(s) = chunks.rest() {
                attrs.insert(opts_at, EAttr::Footer(Help::Doc(s)));
            }
        }
    }
}

impl ToTokens for Top {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Top {
            ty,
            vis,
            generate,
            body,
            mode,
            attrs,
            boxed,
            cargo_helper,
        } = self;

        if matches!(mode, Mode::Options) {
            let body = match cargo_helper {
                Some(cargo) => quote!(::bpaf::cargo_helper(#cargo, #body)),
                None => quote!(#body),
            };

            quote! {
                #vis fn #generate() -> ::bpaf::OptionParser<#ty> {
                    #[allow(unused_imports)]
                    use ::bpaf::Parser;
                    #body
                    #(.#attrs)*
                }
            }
        } else {
            let boxed = if *boxed { quote!(.boxed()) } else { quote!() };
            quote! {
                #vis fn #generate() -> impl ::bpaf::Parser<#ty> {
                    #[allow(unused_imports)]
                    use ::bpaf::Parser;

                    #body
                    #(.#attrs)*
                    #boxed
                }
            }
        }
        .to_tokens(tokens);
    }
}

// }}}

/// Describes the actual fields,
/// can be either a single branch for struct or multiple enum variants
#[derive(Debug)]
enum Body {
    // {{{
    Single(Branch),
    Alternatives(Ident, Vec<EnumBranch>),
}

impl Parse for Body {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(token::Struct) {
            input.parse::<token::Struct>()?;
            let branch = Self::Single(input.parse::<Branch>()?);
            if input.peek(token::Semi) {
                input.parse::<token::Semi>()?;
            }
            Ok(branch)
        } else if input.peek(token::Enum) {
            input.parse::<token::Enum>()?;
            let name = input.parse::<Ident>()?;
            let content;
            braced!(content in input);

            let branches = content
                .parse_terminated(ParsedEnumBranch::parse, token::Comma)?
                .into_iter()
                .filter_map(|p| p.resolve(&name).transpose())
                .collect::<Result<Vec<_>>>()?;

            //            let mut branches
            /*
                        for b in branches.iter_mut() {
                            b.postprocess_with_name(&name);
                        }

                        let branches = branches.into_iter().filter(|b| !b.skip).collect::<Vec<_>>();
            */

            Ok(Self::Alternatives(name, branches))
        } else {
            Err(input.error("Only structs and enums are supported"))
        }
    }
}

impl Body {
    fn ty(&self) -> Ident {
        match self {
            Body::Single(b) => &b.ident,
            Body::Alternatives(n, _) => n,
        }
        .clone()
    }
}

impl Body {
    fn set_named_command(&mut self, span: Span) -> Result<()> {
        match self {
            Body::Single(branch) => {
                branch.set_command();
                Ok(())
            }
            Body::Alternatives(_, _) => Err(Error::new(
                span,
                "You can't annotate `enum` with a named command.",
            )),
        }
    }

    fn set_unnamed_command(&mut self) {
        match self {
            Body::Single(b) => b.set_unnamed_command(),
            Body::Alternatives(_, _branches) => {
                //                for branch in branches {
                todo!();
                //                    branch.set_unnamed_command()?;
                //                }
                //Ok(())
            }
        }
    }
}

impl ToTokens for Body {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Body::Single(branch) => quote!(#branch),
            Body::Alternatives(_name, b) if b.len() == 1 => {
                let branch = &b[0];
                quote!(#branch)
            }
            Body::Alternatives(_name, b) => {
                let branches = b.iter();
                let mk = |i| Ident::new(&format!("alt{}", i), Span::call_site());
                let name_f = b.iter().enumerate().map(|(n, _)| mk(n));
                let name_t = name_f.clone();
                quote! {{
                    #( let #name_f = #branches; )*
                    ::bpaf::construct!([ #( #name_t, )* ])
                }}
            }
        }
        .to_tokens(tokens);
    }
}

// }}}

/// Generating code for enum branch needs enum name which is not available from
/// parsing at that moment so operations are performed in two steps:
/// 1. parse ParsedEnumBranch
/// 2. resolve it into EnumBranch (or skip if skip is present)
pub(crate) struct ParsedEnumBranch {
    // {{{
    branch: Branch,
    attrs: Vec<Attribute>,
}

impl ParsedEnumBranch {
    fn resolve(self, enum_name: &Ident) -> Result<Option<EnumBranch>> {
        let ParsedEnumBranch { mut branch, attrs } = self;

        branch.enum_name = Some(EnumPrefix(enum_name.clone()));

        let (enum_decor, mut help) = parse_bpaf_doc_attrs::<Ed>(&attrs)?;
        let Ed { attrs: ea, skip } = enum_decor.unwrap_or_default();
        if skip {
            return Ok(None);
        }

        let mut attrs = Vec::with_capacity(ea.len());
        let mut has_options = None;

        for attr in ea {
            match attr {
                EAttr::NamedCommand(_) => {
                    branch.set_command();
                    attrs.push(EAttr::ToOptions);
                    has_options = Some(attrs.len());
                    attrs.push(attr);
                }
                EAttr::UnnamedCommand => {
                    branch.set_command();
                    attrs.push(EAttr::ToOptions);
                    has_options = Some(attrs.len());
                    attrs.push(EAttr::NamedCommand(ident_to_long(&branch.ident)));
                }

                EAttr::CommandShort(_) | EAttr::CommandLong(_) => {
                    // TODO should probably be a bit more careful here,
                    // new_derive macro addresses that though
                    attrs.push(attr);
                }

                EAttr::UnitShort(n) => branch.set_unit_name(StrictName::Short {
                    name: n.unwrap_or_else(|| ident_to_short(&branch.ident)),
                }),
                EAttr::UnitLong(n) => branch.set_unit_name(StrictName::Long {
                    name: n.unwrap_or_else(|| ident_to_long(&branch.ident)),
                }),
                EAttr::Env(name) => branch.set_unit_name(StrictName::Env { name }),

                EAttr::Usage(_) => {
                    if let Some(o) = attrs.iter().position(|i| matches!(i, EAttr::ToOptions)) {
                        attrs.insert(o + 1, attr);
                    } else {
                        unreachable!();
                    }
                }
                EAttr::Adjacent | EAttr::Hide => attrs.push(attr),
                EAttr::Header(_) | EAttr::Footer(_) | EAttr::Descr(_) => {
                    if let Some(o) = attrs.iter().position(|i| matches!(i, EAttr::ToOptions)) {
                        attrs.insert(o + 1, attr);
                    }
                }
                EAttr::ToOptions => unreachable!(),
            }
        }

        if let Some(opts_at) = has_options {
            if let Some(h) = std::mem::take(&mut help) {
                split_ehelp_into(h, opts_at, &mut attrs);
            }
        }
        branch.set_inplicit_name();
        if let Some(help) = help {
            branch.push_help(help);
        }

        Ok(Some(EnumBranch { branch, attrs }))
    }
}

impl Parse for ParsedEnumBranch {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ParsedEnumBranch {
            attrs: input.call(Attribute::parse_outer)?,
            branch: input.parse::<Branch>()?,
        })
    }
}

// }}}

#[derive(Debug)]
pub(crate) struct EnumBranch {
    // {{{
    branch: Branch,
    attrs: Vec<EAttr>,
}

impl ToTokens for EnumBranch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumBranch { branch, attrs } = self;
        quote!(#branch #(.#attrs)*).to_tokens(tokens);
    }
}

// }}}

#[derive(Debug)]
pub struct Branch {
    // {{{
    enum_name: Option<EnumPrefix>,
    ident: Ident,
    fields: FieldSet,
}

impl Branch {
    fn set_command(&mut self) {
        if let FieldSet::Unit(_, _, _) = self.fields {
            let ident = &self.ident;
            let enum_name = &self.enum_name;
            self.fields = FieldSet::Pure(parse_quote!(::bpaf::pure(#enum_name #ident)));
        }
    }

    fn set_unnamed_command(&mut self) {
        if let FieldSet::Unit(_, _, _) = self.fields {
            self.set_command();
        }
    }

    fn set_unit_name(&mut self, name: StrictName) {
        if let FieldSet::Unit(_, names, _) = &mut self.fields {
            names.push(name);
        }
    }

    fn set_inplicit_name(&mut self) {
        if let FieldSet::Unit(_, names, _) = &mut self.fields {
            if !names
                .iter()
                .any(|n| matches!(n, StrictName::Long { .. } | StrictName::Short { .. }))
            {
                names.push(StrictName::Long {
                    name: ident_to_long(&self.ident),
                });
            }
        }
    }
    fn push_help(&mut self, help: Help) {
        if let FieldSet::Unit(_, _, h) = &mut self.fields {
            *h = Some(help);
            //        } else {
            //            todo!("use GroupHelp here");
            // TODO use GroupHelp here?
        }
    }
}

impl Parse for Branch {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let content;
        let fields = if input.peek(token::Paren) {
            parenthesized!(content in input);
            FieldSet::Unnamed(content.parse_terminated(StructField::parse_unnamed, token::Comma)?)
        } else if input.peek(token::Brace) {
            braced!(content in input);
            FieldSet::Named(content.parse_terminated(StructField::parse_named, token::Comma)?)
        } else {
            if input.peek(token::Semi) {
                input.parse::<token::Semi>()?;
            }
            FieldSet::Unit(ident.clone(), Vec::new(), None)
        };

        Ok(Branch {
            enum_name: None,
            ident,
            //decor,
            fields,
        })
    }
}

impl ToTokens for Branch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Branch {
            enum_name,
            ident,
            fields,
        } = self;
        match fields {
            FieldSet::Named(fields) => {
                let name = fields
                    .iter()
                    .enumerate()
                    .map(|(ix, field)| field.var_name(ix));
                let result = name.clone();
                let value = fields.iter();
                quote! {{
                    #( let #name = #value; )*
                    ::bpaf::construct!( #enum_name #ident { #( #result , )* })
                }}
            }
            FieldSet::Unnamed(fields) => {
                let name = fields
                    .iter()
                    .enumerate()
                    .map(|(ix, field)| field.var_name(ix));
                let result = name.clone();
                let value = fields.iter();
                quote! {{
                    #( let #name = #value; )*
                    ::bpaf::construct!( #enum_name #ident ( #( #result , )* ))
                }}
            }
            FieldSet::Unit(ident, names, help) => {
                let help = help.iter();
                if names.is_empty() {
                    let name = StrictName::Long {
                        name: ident_to_long(ident),
                    };
                    quote!(::bpaf:: #name.#(help(#help).)* req_flag(#enum_name #ident))
                } else {
                    quote!(::bpaf:: #( #names .)* #(help(#help).)* req_flag(#enum_name #ident))
                }
            }
            FieldSet::Pure(x) => quote!(#x),
        }
        .to_tokens(tokens);
    }
}
// }}}

#[derive(Debug)]
pub(crate) enum FieldSet {
    // {{{
    Named(Punctuated<StructField, token::Comma>),
    Unnamed(Punctuated<StructField, token::Comma>),
    Unit(Ident, Vec<StrictName>, Option<Help>),
    Pure(Box<Expr>),
}
