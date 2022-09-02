use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse, parse2, token, Attribute, Expr, Ident, LitChar, LitStr, Result,
    Token, Visibility,
};

use crate::field::{ConstrName, Doc, EnumSingleton, FieldParser, OptNameAttr, ReqFlag};
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
    Named(Punctuated<FieldParser, Token![,]>),
    Unnamed(Punctuated<FieldParser, Token![,]>),
    NoFields,
}
impl Parse for Fields {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let content;
        if input.peek(token::Brace) {
            let _ = braced!(content in input);
            let fields = content.parse_terminated(FieldParser::parse_named)?;
            Ok(Fields::Named(fields))
        } else if input.peek(token::Paren) {
            let _ = parenthesized!(content in input);
            let fields: Punctuated<_, Token![,]> =
                content.parse_terminated(FieldParser::parse_unnamed)?;
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
        let mut vis = input.parse::<Visibility>()?;

        let outer_ty;
        let mut name = None;
        let mut version = None;

        let kind;

        if input.peek(Token![struct]) {
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

            let _ = input.parse::<Token![struct]>()?;
            outer_ty = input.parse::<Ident>()?;
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
        } else if input.peek(Token![enum]) {
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

            let _ = input.parse::<Token![enum]>()?;
            outer_ty = input.parse::<Ident>()?;
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

                let branch = if enum_contents.peek(token::Paren) || enum_contents.peek(token::Brace)
                {
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
                    let (help, inner) = split_help_and::<EnumSingleton<OptNameAttr>>(&attrs)?;
                    BParser::Singleton(ReqFlag::new(constr, inner, &help))
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
                    kind = ParserKind::BParser(BParser::Command(
                        cmd_name,
                        cmd_attr,
                        Box::new(oparser),
                    ));
                }
            }
        } else {
            return Err(input.error("Only struct and enum types are supported"));
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

#[cfg(test)]
mod test {
    use super::Top;
    use quote::{quote, ToTokens};
    use syn::parse_quote;

    #[test]
    fn cargo_command_helper() {
        let top: Top = parse_quote! {
            #[bpaf(options("asm"))]
            struct Opts {
                verbose: bool
            }
        };

        let expected = quote! {
            fn opts() -> ::bpaf::OptionParser<Opts> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    ::bpaf::cargo_helper("asm", {
                        let verbose = ::bpaf::long("verbose").switch();
                        ::bpaf::construct!(Opts { verbose })
                    })
                }
                .to_options()
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn top_struct_construct() {
        let top: Top = parse_quote! {
            #[bpaf(construct)]
            struct Opt { verbose: bool }
        };

        let expected = quote! {
            fn opt() -> impl ::bpaf::Parser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let verbose = ::bpaf::long("verbose").switch();
                    ::bpaf::construct!(Opt { verbose })
                }
            }
        };

        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn top_enum_construct() {
        let top: Top = parse_quote! {
            #[bpaf(construct)]
            enum Opt { Foo { verbose_name: bool }}
        };

        let expected = quote! {
            fn opt() -> impl ::bpaf::Parser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let verbose_name = ::bpaf::long("verbose-name").switch();
                    ::bpaf::construct!(Opt::Foo { verbose_name })
                }
            }
        };

        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn top_struct_options1() {
        let top: Top = parse_quote! {
            /// those are options
            #[bpaf(options)]
            struct Opt {}
        };

        let expected = quote! {
            fn opt() -> ::bpaf::OptionParser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                    {
                        ::bpaf::construct!(Opt {})
                    }
                    .to_options()
                    .descr("those are options")
            }
        };

        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn struct_options2() {
        let input: Top = parse_quote! {
            #[bpaf(options)]
            /// those are options
            struct Opt {}
        };

        let expected = quote! {
            fn opt() -> ::bpaf::OptionParser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    ::bpaf::construct!(Opt {})
                }
                .to_options()
                .descr("those are options")
            }
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn struct_command() {
        let input: Top = parse_quote! {
            /// those are options
            #[bpaf(command)]
            struct Opt {}
        };

        let expected = quote! {
            fn opt() -> impl ::bpaf::Parser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let inner_cmd =
                        { ::bpaf::construct!(Opt {}) }
                        .to_options()
                        .descr("those are options")
                    ;
                    ::bpaf::command("opt", inner_cmd).help("those are options")
                }
            }
        };
        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn enum_command() {
        let input: Top = parse_quote! {
            /// those are options
            enum Opt {
                #[bpaf(command("foo"))]
                /// foo doc
                Foo { field: usize },
                /// bar doc
                #[bpaf(command)]
                Bar { field: bool }
            }
        };

        let expected = quote! {
            fn opt() -> impl ::bpaf::Parser<Opt> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    let alt0 = {
                        let inner_cmd = {
                            let field = ::bpaf::long("field").argument("ARG").from_str::<usize>();
                            ::bpaf::construct!(Opt::Foo { field })
                        }
                        .to_options()
                        .descr("foo doc");
                        ::bpaf::command("foo", inner_cmd).help("foo doc")
                    };
                    let alt1 = {
                        let inner_cmd = {
                            let field = ::bpaf::long("field").switch();
                            ::bpaf::construct!(Opt::Bar { field })
                        }
                        .to_options()
                        .descr("bar doc");
                        ::bpaf::command("bar", inner_cmd).help("bar doc")
                    };
                    ::bpaf::construct!([alt0, alt1])
                }
            }
        };
        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn unnamed_struct() {
        let top: Top = parse_quote! {
            #[bpaf(options)]
            struct Opt(
                /// help
                PathBuf
            );
        };

        let expected = quote! {
            fn opt() -> ::bpaf::OptionParser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let f0 = ::bpaf::positional_os("ARG").help("help").map(PathBuf::from);
                    ::bpaf::construct!(Opt(f0))
                }
                .to_options()
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn unnamed_enum() {
        let top: Top = parse_quote! {
            #[bpaf(options, version)]
            enum Opt1 {
                Con1(PathBuf, usize)
            }
        };

        let expected = quote! {
            fn opt1() -> ::bpaf::OptionParser<Opt1> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let f0 = ::bpaf::positional_os("ARG").map(PathBuf::from);
                    let f1 = ::bpaf::positional("ARG").from_str::<usize>();
                    ::bpaf::construct!(Opt1::Con1(f0, f1))
                }
                .to_options()
                .version(env!("CARGO_PKG_VERSION"))
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn enum_to_flag_and_switches() {
        let top: Top = parse_quote! {
            #[bpaf(construct)]
            pub enum Opt {
                #[bpaf(long("Foo"))]
                Foo,
                #[bpaf(short)]
                Pff,
                BarFoo,
                Baz(#[bpaf(long("bazz"))] String),
                Strange { strange: String },
                #[bpaf(command("alpha"))]
                Alpha,
                #[bpaf(command)]
                Omega,
            }
        };

        let expected = quote! {
            pub fn opt() -> impl ::bpaf::Parser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let alt0 = ::bpaf::long("Foo").req_flag(Opt::Foo);
                    let alt1 = ::bpaf::short('p').req_flag(Opt::Pff);
                    let alt2 = ::bpaf::long("bar-foo").req_flag(Opt::BarFoo);
                    let alt3 = {
                        let f0 = ::bpaf::long("bazz").argument("ARG");
                        ::bpaf::construct!(Opt::Baz(f0))
                    };
                    let alt4 = {
                        let strange = ::bpaf::long("strange").argument("ARG");
                        ::bpaf::construct!(Opt::Strange { strange })
                    };
                    let alt5 = {
                        let inner_cmd = ::bpaf::pure(Opt::Alpha).to_options();
                        ::bpaf::command("alpha", inner_cmd)
                    };
                    let alt6 = {
                        let inner_cmd = ::bpaf::pure(Opt::Omega).to_options() ;
                        ::bpaf::command("omega", inner_cmd)
                    };
                    ::bpaf::construct!([alt0, alt1, alt2, alt3, alt4, alt5, alt6])
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn help_generation() {
        let top: Top = parse_quote! {
            /// descr
            ///   a
            ///
            ///
            ///
            ///
            /// footer
            ///  a
            #[bpaf(options)]
            struct Opt(PathBuf);
        };

        let expected = quote! {
            fn opt() -> ::bpaf::OptionParser<Opt> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let f0 = ::bpaf::positional_os("ARG").map(PathBuf::from);
                    ::bpaf::construct!(Opt(f0))
                }
                .to_options()
                .descr("descr\n  a")
                .footer("footer\n a")
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn version_with_commands() {
        let top: Top = parse_quote! {
            #[bpaf(options, version)]
            enum Action {
                Alpha,
                Beta,
            }
        };
        let expected = quote! {
            fn action() -> ::bpaf::OptionParser<Action> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let alt0 = ::bpaf::long("alpha").req_flag(Action::Alpha);
                    let alt1 = ::bpaf::long("beta").req_flag(Action::Beta);
                    ::bpaf::construct!([alt0, alt1])
                }
                .to_options()
                .version(env!("CARGO_PKG_VERSION"))
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn command_with_aliases() {
        let top: Top = parse_quote! {
            #[bpaf(command, short('c'), long("long"))]
            struct Command {
                i: bool,
            }
        };

        let expected = quote! {
            fn command() -> impl ::bpaf::Parser<Command> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    let inner_cmd = {
                        let i = ::bpaf::short('i').switch();
                        ::bpaf::construct!(Command { i })
                    }
                    .to_options();
                    ::bpaf::command("command", inner_cmd)
                        .short('c')
                        .long("long")
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn version_with_commands_with_cargo_helper() {
        let top: Top = parse_quote! {
            #[bpaf(options("subcargo"), version)]
            enum Action {
                #[bpaf(command)]
                Alpha,
                #[bpaf(command)]
                Beta,
            }
        };

        let expected = quote! {
            fn action() -> ::bpaf::OptionParser<Action> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    ::bpaf::cargo_helper("subcargo", {
                        let alt0 = {
                            let inner_cmd = ::bpaf::pure(Action::Alpha).to_options();
                            ::bpaf::command("alpha", inner_cmd)
                        };
                        let alt1 = {
                            let inner_cmd = ::bpaf::pure(Action::Beta).to_options();
                            ::bpaf::command("beta", inner_cmd)
                        };
                        ::bpaf::construct!([alt0, alt1])
                    })
                }
                .to_options()
                .version(env!("CARGO_PKG_VERSION"))
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn named_to_positional_with_metavar() {
        let top: Top = parse_quote! {
            struct Options {
                #[bpaf(positional_os("PATH"))]
                path: PathBuf,
            }

        };

        let expected = quote! {
            fn options() -> impl ::bpaf::Parser<Options> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let path = ::bpaf::positional_os("PATH").map(PathBuf::from);
                    ::bpaf::construct!(Options { path })
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn named_to_positional_without_metavar() {
        let top: Top = parse_quote! {
            struct Options {
                #[bpaf(positional_os)]
                path: PathBuf,
            }

        };

        let expected = quote! {
            fn options() -> impl ::bpaf::Parser<Options> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let path = ::bpaf::positional_os("ARG").map(PathBuf::from);
                    ::bpaf::construct!(Options { path })
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn comp_visibility_struct() {
        let top: Top = parse_quote! {
            #[bpaf(complete_style(x))]
            pub struct Options {
                path: PathBuf,
            }
        };
        let expected = quote! {
            pub fn options() -> impl ::bpaf::Parser<Options> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                    :: bpaf :: construct ! (Options { path })
                }.complete_style(x)
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn comp_visibility_enum() {
        let top: Top = parse_quote! {
            #[bpaf(complete_style(x))]
            pub enum Foo {
                Bar {
                    path: PathBuf,
                }
            }
        };
        let expected = quote! {
            pub fn foo() -> impl ::bpaf::Parser<Foo> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                    :: bpaf :: construct ! (Foo::Bar { path })
                }
                .complete_style(x)
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn private_visibility() {
        let top: Top = parse_quote! {
            #[bpaf(private)]
            pub struct Options {
                path: PathBuf,
            }

        };

        let expected = quote! {
            fn options() -> impl ::bpaf::Parser<Options> {
                #[allow (unused_imports)]
                use ::bpaf::Parser;
                {
                    let path = ::bpaf::long("path").argument_os("ARG").map(PathBuf::from);
                    ::bpaf::construct!(Options { path })
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn hidden_default_enum_singleton() {
        let top: Top = parse_quote! {
            enum Decision {
                Yes,
                #[bpaf(default, hide)]
                No,
            }
        };

        let expected = quote! {
            fn decision() -> impl ::bpaf::Parser<Decision> {
                #[allow(unused_imports)]
                use ::bpaf::Parser;
                {
                    let alt0 = ::bpaf::long("yes").req_flag(Decision::Yes);
                    let alt1 = ::bpaf::long("no")
                        .flag(Decision::No, Decision::No)
                        .hide();
                    ::bpaf::construct!([alt0, alt1])
                }
            }
        };
        assert_eq!(top.to_token_stream().to_string(), expected.to_string());
    }
}
