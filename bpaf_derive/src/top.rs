use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse, parse2, token, Attribute, Expr, Ident, LitChar, LitStr, Result,
    Token, Visibility,
};

use crate::field::{parse_ident, ConstrName, Doc, Field, ReqFlag};
use crate::kw;
use crate::utils::{snake_case_ident, to_snake_case, LineIter};

#[derive(Debug)]
pub struct Top {
    /// generated function name
    name: Ident,

    /// visibility for the generated function
    vis: Visibility,

    /// Type for generated function:
    ///
    /// T in Parser<T> or OptionParser<T>
    outer_ty: Ident,

    kind: ParserKind,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum ParserKind {
    BParser(BParser),
    OParser(OParser),
}

#[derive(Debug)]
enum BParser {
    Command(LitStr, CommandAttr, Box<OParser>),
    CargoHelper(LitStr, Box<BParser>),
    CompStyle(Box<Expr>, Box<BParser>),
    Constructor(ConstrName, Fields),
    Singleton(ReqFlag),
    Fold(Vec<BParser>),
}

#[derive(Debug)]
struct OParser {
    inner: Box<BParser>,
    decor: Decor,
}

#[derive(Debug)]
struct Decor {
    descr: Option<String>,
    header: Option<String>,
    footer: Option<String>,
    version: Option<Box<Expr>>,
}

impl ToTokens for Decor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(descr) = &self.descr {
            if !descr.is_empty() {
                quote!(.descr(#descr)).to_tokens(tokens);
            }
        }
        if let Some(header) = &self.header {
            if !header.is_empty() {
                quote!(.header(#header)).to_tokens(tokens);
            }
        }
        if let Some(footer) = &self.footer {
            if !footer.is_empty() {
                quote!(.footer(#footer)).to_tokens(tokens);
            }
        }
        if let Some(ver) = &self.version {
            quote!(.version(#ver)).to_tokens(tokens);
        }
    }
}

/// A collection of fields, corresponds to a single constructor in enum or the whole struct but
/// without the name
#[derive(Clone, Debug)]
enum Fields {
    Named(Punctuated<Field, Token![,]>),
    Unnamed(Punctuated<Field, Token![,]>),
    NoFields,
}
impl Parse for Fields {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let content;
        if input.peek(token::Brace) {
            let _ = braced!(content in input);
            let fields = content.parse_terminated(Field::parse_named)?;
            Ok(Fields::Named(fields))
        } else if input.peek(token::Paren) {
            let _ = parenthesized!(content in input);
            let fields: Punctuated<_, Token![,]> =
                content.parse_terminated(Field::parse_unnamed)?;
            Ok(Fields::Unnamed(fields))
        } else {
            Err(input.error("Expected named or unnamed struct"))
        }
    }
}

#[derive(Clone, Debug)]
enum OuterKind {
    Construct,
    Options(Option<LitStr>),
    Command(CommandAttr),
}

#[derive(Clone, Debug)]
enum OuterAttr {
    Options(Option<LitStr>),
    Construct,
    Private,
    Generate(Ident),
    Command(CommandAttr),
    Version(Option<Box<Expr>>),
    CompStyle(Box<Expr>),
}

#[derive(Clone, Debug)]
struct CommandAttr {
    name: Option<LitStr>,
    shorts: Vec<LitChar>,
    longs: Vec<LitStr>,
}

impl Parse for CommandAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(kw::command) {
            input.parse::<kw::command>()?;
            let name;

            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let lit = content.parse::<LitStr>()?;
                name = Some(lit);
            } else {
                name = None;
            };
            let mut shorts = Vec::new();
            let mut longs = Vec::new();
            loop {
                if input.peek(token::Comma) && input.peek2(kw::short) {
                    input.parse::<token::Comma>()?;
                    input.parse::<kw::short>()?;
                    let content;
                    let _ = parenthesized!(content in input);
                    shorts.push(content.parse::<LitChar>()?);
                } else if input.peek(token::Comma) && input.peek2(kw::long) {
                    input.parse::<token::Comma>()?;
                    input.parse::<kw::long>()?;
                    let content;
                    let _ = parenthesized!(content in input);
                    longs.push(content.parse::<LitStr>()?);
                } else {
                    break;
                }
            }
            Ok(Self {
                name,
                shorts,
                longs,
            })
        } else {
            Err(input.error("Unexpected attribute"))
        }
    }
}

#[derive(Debug)]
struct Outer {
    kind: Option<OuterKind>,
    version: Option<Expr>,
    comp_style: Option<Expr>,
    generate: Option<Ident>,
    help: Option<Decor>,
}

impl Outer {
    fn make(attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = Outer {
            kind: None,
            version: None,
            comp_style: None,
            generate: None,
            help: None,
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
                    if keyword == "generate" {
                        res.generate = Some(parse_ident(&input)?);
                    } else {
                        return Err(input_copy.error("Unexpected attribute"));
                    }

                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;
                })?;
            } else {
                unreachable!("Shouldn't get any attributes other than bpaf and doc")
            }
        }
        todo!("{:?}, {:?}", res, help);
    }
}

impl Parse for OuterAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let content;
        if input.peek(kw::private) {
            let _: kw::private = input.parse()?;
            Ok(Self::Private)
        } else if input.peek(kw::generate) {
            let _: kw::generate = input.parse()?;
            let _ = parenthesized!(content in input);
            let name = content.parse()?;
            Ok(Self::Generate(name))
        } else if input.peek(kw::construct) {
            let _: kw::construct = input.parse()?;
            Ok(Self::Construct)
        } else if input.peek(kw::options) {
            let _: kw::options = input.parse()?;
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let lit = content.parse::<LitStr>()?;
                Ok(Self::Options(Some(lit)))
            } else {
                Ok(Self::Options(None))
            }
        } else if input.peek(kw::complete_style) {
            input.parse::<kw::complete_style>()?;
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::CompStyle(Box::new(expr)))
        } else if input.peek(kw::command) {
            Ok(Self::Command(input.parse::<CommandAttr>()?))
        } else if input.peek(kw::version) {
            let _: kw::version = input.parse()?;
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let expr = content.parse::<Expr>()?;
                Ok(Self::Version(Some(Box::new(expr))))
            } else {
                Ok(Self::Version(None))
            }
        } else {
            Err(input.error("Unexpected attribute"))
        }
    }
}

struct InnerAttr(Option<LitStr>);
impl Parse for InnerAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(kw::command) {
            let _: kw::command = input.parse()?;
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let lit = content.parse::<LitStr>()?;
                Ok(Self(Some(lit)))
            } else {
                Ok(Self(None))
            }
        } else {
            Err(input.error("Unexpected attribute"))
        }
    }
}

pub fn split_help_and<T: Parse>(attrs: &[Attribute]) -> Result<(Vec<String>, Vec<T>)> {
    let mut help = Vec::new();
    let mut res = Vec::new();
    for attr in attrs {
        if attr.path.is_ident("doc") {
            let Doc(doc) = parse2(attr.tokens.clone())?;
            help.push(doc);
        } else if attr.path.is_ident("bpaf") {
            res.extend(attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)?);
        }
    }

    Ok((help, res))
}

impl Parse for Top {
    #[allow(clippy::too_many_lines)]
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;

        if input.peek(token::Struct) {
            input.parse::<token::Struct>()?;
            Self::parse_struct(attrs, vis, input)
        } else if input.peek(token::Enum) {
            input.parse::<token::Enum>()?;
            Self::parse_enum(attrs, vis, input)
        } else {
            Err(input.error("Only struct and enum types are supported"))
        }
    }
}

impl Top {
    fn parse_struct(
        attrs: Vec<Attribute>,
        mut vis: Visibility,
        input: ParseStream,
    ) -> Result<Self> {
        let mut name = None;
        let mut version = None;

        let kind;

        let (help, outer) = split_help_and::<OuterAttr>(&attrs)?;
        let mut outer_kind = None;
        let mut comp_style = None;
        for attr in outer {
            match attr {
                OuterAttr::Options(n) => outer_kind = Some(OuterKind::Options(n)),
                OuterAttr::Construct => outer_kind = Some(OuterKind::Construct),
                OuterAttr::Generate(n) => name = Some(n.clone()),
                OuterAttr::Command(n) => outer_kind = Some(OuterKind::Command(n)),
                OuterAttr::Version(Some(ver)) => version = Some(ver.clone()),
                OuterAttr::Version(None) => {
                    version = Some(syn::parse_quote!(env!("CARGO_PKG_VERSION")));
                }
                OuterAttr::Private => {
                    vis = Visibility::Inherited;
                }
                OuterAttr::CompStyle(style) => {
                    comp_style = Some(style);
                }
            }
        }

        let outer_ty = input.parse::<Ident>()?;
        let bra = input.parse::<Fields>()?;

        if bra.struct_definition_followed_by_semi() {
            input.parse::<Token![;]>()?;
        }

        let constr = ConstrName {
            namespace: None,
            constr: outer_ty.clone(),
        };
        let inner = BParser::Constructor(constr, bra);
        match outer_kind.unwrap_or(OuterKind::Construct) {
            OuterKind::Construct => {
                let inner = match comp_style {
                    Some(style) => BParser::CompStyle(style, Box::new(inner)),
                    None => inner,
                };
                kind = ParserKind::BParser(inner);
            }
            OuterKind::Options(n) => {
                let decor = Decor::new(&help, version.take());
                let inner = match n {
                    Some(name) => BParser::CargoHelper(name, Box::new(inner)),
                    None => inner,
                };
                let oparser = OParser {
                    decor,
                    inner: Box::new(inner),
                };
                kind = ParserKind::OParser(oparser);
            }
            OuterKind::Command(cmd_attr) => {
                let decor = Decor::new(&help, version.take());
                let oparser = OParser {
                    decor,
                    inner: Box::new(inner),
                };
                let cmd_name = cmd_attr.name.clone().unwrap_or_else(|| {
                    let n = to_snake_case(&outer_ty.to_string());
                    LitStr::new(&n, outer_ty.span())
                });
                let cmd = BParser::Command(cmd_name, cmd_attr, Box::new(oparser));
                kind = ParserKind::BParser(cmd);
            }
        }

        Ok(Top {
            name: name.unwrap_or_else(|| snake_case_ident(&outer_ty)),
            vis,
            outer_ty,
            kind,
        })
    }

    fn parse_enum(attrs: Vec<Attribute>, mut vis: Visibility, input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut version = None;

        let kind;

        let (help, outer) = split_help_and::<OuterAttr>(&attrs)?;
        let mut outer_kind = None;
        let mut comp_style = None;
        for attr in outer {
            match attr {
                OuterAttr::Options(n) => outer_kind = Some(OuterKind::Options(n)),
                OuterAttr::Construct => outer_kind = Some(OuterKind::Construct),
                OuterAttr::Generate(n) => name = Some(n.clone()),
                OuterAttr::Version(Some(ver)) => version = Some(ver.clone()),
                OuterAttr::Version(None) => {
                    version = Some(syn::parse_quote!(env!("CARGO_PKG_VERSION")));
                }
                OuterAttr::Command(n) => outer_kind = Some(OuterKind::Command(n)),
                OuterAttr::Private => {
                    vis = Visibility::Inherited;
                }
                OuterAttr::CompStyle(style) => {
                    comp_style = Some(style);
                }
            }
        }

        let outer_ty = input.parse::<Ident>()?;
        let mut branches: Vec<BParser> = Vec::new();

        let enum_contents;
        let _ = braced!(enum_contents in input);
        loop {
            if enum_contents.is_empty() {
                break;
            }
            let attrs = enum_contents.call(Attribute::parse_outer)?;

            let inner_ty = enum_contents.parse::<Ident>()?;

            let constr = ConstrName {
                namespace: Some(outer_ty.clone()),
                constr: inner_ty.clone(),
            };

            let branch = if enum_contents.peek(token::Paren) || enum_contents.peek(token::Brace) {
                let (help, mut inner) = split_help_and::<CommandAttr>(&attrs)?;

                let bra = enum_contents.parse::<Fields>()?;

                assert!(inner.len() <= 1);
                if let Some(cmd_arg) = inner.pop() {
                    let cmd_name = cmd_arg.name.clone().unwrap_or_else(|| {
                        let n = to_snake_case(&inner_ty.to_string());
                        LitStr::new(&n, inner_ty.span())
                    });
                    let decor = Decor::new(&help, None);
                    let oparser = OParser {
                        inner: Box::new(BParser::Constructor(constr, bra)),
                        decor,
                    };
                    BParser::Command(cmd_name, cmd_arg, Box::new(oparser))
                } else {
                    BParser::Constructor(constr, bra)
                }
            } else if let Ok((help, Some(inner))) = split_help_and::<CommandAttr>(&attrs)
                .map(|(h, a)| (h, (a.len() == 1).then(|| a.first().cloned()).flatten()))
            {
                let cmd_name = inner.name.clone().unwrap_or_else(|| {
                    let n = to_snake_case(&inner_ty.to_string());
                    LitStr::new(&n, inner_ty.span())
                });

                let decor = Decor::new(&help, None);
                let fields = Fields::NoFields;
                let oparser = OParser {
                    inner: Box::new(BParser::Constructor(constr, fields)),
                    decor,
                };
                BParser::Command(cmd_name, inner, Box::new(oparser))
            } else {
                let req_flag = ReqFlag::make(constr, attrs)?;
                BParser::Singleton(req_flag)
            };

            let branch = match &comp_style {
                Some(style) => BParser::CompStyle(style.clone(), Box::new(branch)),
                None => branch,
            };
            branches.push(branch);

            if !enum_contents.is_empty() {
                enum_contents.parse::<Token![,]>()?;
            }
        }

        let inner = BParser::Fold(branches);
        match outer_kind.unwrap_or(OuterKind::Construct) {
            OuterKind::Construct => {
                kind = ParserKind::BParser(inner);
            }
            OuterKind::Options(n) => {
                let decor = Decor::new(&help, version.take());
                let inner = match n {
                    Some(name) => BParser::CargoHelper(name, Box::new(inner)),
                    None => inner,
                };
                let oparser = OParser {
                    decor,
                    inner: Box::new(inner),
                };
                kind = ParserKind::OParser(oparser);
            }
            OuterKind::Command(cmd_attr) => {
                let cmd_name = cmd_attr.name.clone().unwrap_or_else(|| {
                    let n = to_snake_case(&outer_ty.to_string());
                    LitStr::new(&n, outer_ty.span())
                });
                let decor = Decor::new(&help, version.take());
                let oparser = OParser {
                    inner: Box::new(inner),
                    decor,
                };
                kind = ParserKind::BParser(BParser::Command(cmd_name, cmd_attr, Box::new(oparser)));
            }
        }

        Ok(Top {
            name: name.unwrap_or_else(|| snake_case_ident(&outer_ty)),
            vis,
            outer_ty,
            kind,
        })
    }
}

impl ToTokens for Top {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Top {
            name,
            vis,
            outer_ty,
            kind,
        } = self;
        let outer_kind = match kind {
            ParserKind::BParser(_) => quote!(impl ::bpaf::Parser<#outer_ty>),
            ParserKind::OParser(_) => quote!(::bpaf::OptionParser<#outer_ty>),
        };
        quote!(
            #vis fn #name() -> #outer_kind {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                #kind
            }
        )
        .to_tokens(tokens);
    }
}

impl ToTokens for ParserKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ParserKind::BParser(bp) => bp.to_tokens(tokens),
            ParserKind::OParser(op) => op.to_tokens(tokens),
        }
    }
}

impl ToTokens for OParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let OParser { inner, decor } = self;
        quote!(#inner.to_options()#decor).to_tokens(tokens);
    }
}

impl ToTokens for BParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BParser::Command(cmd_name, cmd_attr, oparser) => {
                let mut names = quote!();
                for short in &cmd_attr.shorts {
                    names = quote!(#names .short(#short));
                }
                for long in &cmd_attr.longs {
                    names = quote!(#names .long(#long));
                }

                if let Some(msg) = &oparser.decor.descr {
                    quote!( {
                        let inner_cmd = #oparser;
                        ::bpaf::command(#cmd_name, inner_cmd).help(#msg)#names
                    })
                } else {
                    quote!({
                        let inner_cmd = #oparser;
                        ::bpaf::command(#cmd_name, inner_cmd)#names
                    })
                }
                .to_tokens(tokens);
            }
            BParser::CargoHelper(name, inner) => quote!({
                ::bpaf::cargo_helper(#name, #inner)
            })
            .to_tokens(tokens),
            BParser::Constructor(con, Fields::NoFields) => {
                quote!(::bpaf::pure(#con)).to_tokens(tokens);
            }
            BParser::Constructor(con, bra) => {
                let parse_decls = bra.parser_decls();
                quote!({
                    #(#parse_decls)*
                    ::bpaf::construct!(#con #bra)
                })
                .to_tokens(tokens);
            }
            BParser::Fold(xs) => {
                if xs.len() == 1 {
                    xs[0].to_tokens(tokens);
                } else {
                    let mk = |i| Ident::new(&format!("alt{}", i), Span::call_site());
                    let names = xs.iter().enumerate().map(|(ix, _)| mk(ix));
                    let parse_decls = xs.iter().enumerate().map(|(ix, parser)| {
                        let name = mk(ix);
                        quote!( let #name = #parser;)
                    });
                    quote!({
                        #(#parse_decls)*
                        ::bpaf::construct!([#(#names),*])
                    })
                    .to_tokens(tokens);
                }
            }
            BParser::Singleton(field) => field.to_tokens(tokens),
            BParser::CompStyle(style, inner) => {
                quote!(#inner.complete_style(#style)).to_tokens(tokens);
            }
        }
    }
}

impl ToTokens for Fields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Fields::Named(fields) => {
                //                let names = fields.iter().map(|f| f.name());
                let names = fields.iter().enumerate().map(|(ix, f)| f.var_name(ix));
                quote!({ #(#names),*}).to_tokens(tokens);
            }
            Fields::Unnamed(fields) => {
                let names = fields.iter().enumerate().map(|(ix, f)| f.var_name(ix));
                quote!(( #(#names),*)).to_tokens(tokens);
            }
            Fields::NoFields => {}
        }
    }
}

impl ToTokens for ConstrName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let constr = &self.constr;
        match &self.namespace {
            Some(namespace) => quote!(#namespace :: #constr).to_tokens(tokens),
            None => constr.to_tokens(tokens),
        }
    }
}

impl Fields {
    fn parser_decls(&self) -> Vec<TokenStream> {
        match self {
            Fields::Named(fields) => fields
                .iter()
                .enumerate()
                .map(|(ix, field)| {
                    let name = field.var_name(ix);
                    quote!(let #name = #field;)
                })
                .collect::<Vec<_>>(),
            Fields::Unnamed(fields) => fields
                .iter()
                .enumerate()
                .map(|(ix, field)| {
                    let name = field.var_name(ix);
                    quote!(let #name = #field;)
                })
                .collect::<Vec<_>>(),
            Fields::NoFields => Vec::new(),
        }
    }

    const fn struct_definition_followed_by_semi(&self) -> bool {
        match self {
            Fields::Named(_) | Fields::NoFields => false,
            Fields::Unnamed(_) => true,
        }
    }
}

impl Decor {
    fn new(help: &[String], version: Option<Box<Expr>>) -> Self {
        let mut iter = LineIter::from(help);
        Decor {
            descr: iter.next(),
            header: iter.next(),
            footer: iter.next(),
            version,
        }
    }
}
