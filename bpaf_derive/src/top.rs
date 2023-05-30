use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse, parse_quote, token, Attribute, Expr, Ident, LitChar, LitStr,
    Result, Token, Visibility,
};

use crate::field::{
    parse_expr, parse_ident, parse_lit_char, parse_lit_str, parse_opt_arg, ConstrName, Field,
    ReqFlag,
};
use crate::utils::{doc_comment, snake_case_ident, to_snake_case, LineIter};

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
    BParser(BParser, Option<String>),
    OParser(OParser),
}

#[derive(Debug)]
enum BParser {
    Command {
        attribute: CommandAttr,
        inner_parser: Box<OParser>,
        is_adjacent: bool,
        is_hidden: bool,
    },
    CargoHelper(LitStr, Box<BParser>),
    CompStyle(Box<Expr>, Box<BParser>),
    Constructor {
        name: ConstrName,
        fields: Fields,
        adjacent: bool,
        boxed: bool,
    },
    Singleton(Box<ReqFlag>),
    Fold(Vec<BParser>, Option<Box<Expr>>),
}

#[derive(Debug)]
struct OParser {
    inner: Box<BParser>,
    decor: Decor,
}

#[derive(Debug, Default)]
struct Decor {
    descr: Option<String>,
    header: Option<String>,
    footer: Option<String>,
    version: Option<Box<Expr>>,
    usage: Option<LitStr>,
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
        if let Some(usage) = &self.usage {
            quote!(.usage(#usage)).to_tokens(tokens);
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
            let fields = content.parse_terminated(Field::parse_named, token::Comma)?;
            Ok(Fields::Named(fields))
        } else if input.peek(token::Paren) {
            let _ = parenthesized!(content in input);
            let fields = content.parse_terminated(Field::parse_unnamed, token::Comma)?;
            Ok(Fields::Unnamed(fields))
        } else if input.peek(token::Semi) {
            let _ = input.parse::<token::Semi>();
            Ok(Fields::NoFields)
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
pub struct CommandAttr {
    name: LitStr,
    shorts: Vec<LitChar>,
    longs: Vec<LitStr>,
    usage: Option<LitStr>,
}

#[derive(Debug, Clone)]
pub struct Inner {
    pub command: Option<CommandAttr>,
    pub help: Vec<String>, // TODO - use the same with Outer
    pub shorts: Vec<LitChar>,
    pub longs: Vec<LitStr>,
    pub envs: Vec<Expr>,
    pub is_hidden: bool,
    pub is_default: bool,
    pub skip: bool,
}

impl Inner {
    fn make(inner_ty: &Ident, attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = Inner {
            command: None,
            help: Vec::new(),
            shorts: Vec::new(),
            longs: Vec::new(),
            envs: Vec::new(),
            is_hidden: false,
            is_default: false,
            skip: false,
        };
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Some(doc) = doc_comment(&attr) {
                    res.help.push(doc);
                }
            } else if attr.path().is_ident("bpaf") {
                attr.parse_args_with(|input: ParseStream| loop {
                    if input.is_empty() {
                        break Ok(());
                    }
                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;

                    if keyword == "command" {
                        let name = parse_opt_arg::<LitStr>(input)?.unwrap_or_else(|| {
                            let n = to_snake_case(&inner_ty.to_string());
                            LitStr::new(&n, inner_ty.span())
                        });
                        res.command = Some(CommandAttr {
                            name,
                            shorts: Vec::new(),
                            longs: Vec::new(),
                            usage: None,
                        });
                    } else if keyword == "short" {
                        let lit = parse_opt_arg::<LitChar>(input)?.unwrap_or_else(|| {
                            let n = to_snake_case(&inner_ty.to_string()).chars().next().unwrap();
                            LitChar::new(n, inner_ty.span())
                        });
                        res.shorts.push(lit);
                    } else if keyword == "long" {
                        let lit = parse_opt_arg::<LitStr>(input)?.unwrap_or_else(|| {
                            let n = to_snake_case(&inner_ty.to_string());
                            LitStr::new(&n, inner_ty.span())
                        });
                        res.longs.push(lit);
                    } else if keyword == "env" {
                        let lit = parse_expr(input)?;
                        res.envs.push(*lit);
                    } else if keyword == "hide" {
                        res.is_hidden = true;
                    } else if keyword == "default" {
                        res.is_default = true;
                    } else if keyword == "skip" {
                        res.skip = true;
                    } else if keyword == "usage" {
                        if let Some(cmd) = &mut res.command {
                            let content;
                            let _ = parenthesized!(content in input);
                            cmd.usage = Some(content.parse::<LitStr>()?);
                        } else {
                            return Err(input_copy
                                .error("In this context `usage` requires `command` annotation"));
                        }
                    } else {
                        return Err(input_copy.error("Not a valid inner attribute"));
                    }

                    if !input.is_empty() {
                        input.parse::<token::Comma>()?;
                    }
                })?;
            }
        }
        if let Some(cmd) = &mut res.command {
            cmd.shorts.append(&mut res.shorts);
            cmd.longs.append(&mut res.longs);
        }
        Ok(res)
    }
}

#[derive(Debug)]
struct Outer {
    kind: Option<OuterKind>,
    version: Option<Box<Expr>>,
    vis: Visibility,
    comp_style: Option<Box<Expr>>,
    generate: Option<Ident>,
    decor: Decor,
    longs: Vec<LitStr>,
    shorts: Vec<LitChar>,
    fallback: Option<Box<Expr>>,
    adjacent: bool,
    boxed: bool,
}

impl Outer {
    fn make(outer_ty: &Ident, vis: Visibility, attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = Outer {
            kind: None,
            version: None,
            vis,
            comp_style: None,
            generate: None,
            decor: Decor::default(),
            longs: Vec::new(),
            shorts: Vec::new(),
            fallback: None,
            adjacent: false,
            boxed: false,
        };

        let mut usage = None;
        let mut help = Vec::new();
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Some(doc) = doc_comment(&attr) {
                    help.push(doc);
                }
            } else if attr.path().is_ident("bpaf") {
                attr.parse_args_with(|input: ParseStream| loop {
                    if input.is_empty() {
                        break Ok(());
                    }

                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;

                    if keyword == "generate" {
                        res.generate = Some(parse_ident(input)?);
                    } else if keyword == "options" {
                        let lit = if input.peek(token::Paren) {
                            let content;
                            let _ = parenthesized!(content in input);
                            Some(content.parse::<LitStr>()?)
                        } else {
                            None
                        };
                        res.kind = Some(OuterKind::Options(lit));
                    } else if keyword == "complete_style" {
                        let style = parse_expr(input)?;
                        res.comp_style = Some(style);
                    } else if keyword == "construct" {
                        res.kind = Some(OuterKind::Construct);
                    } else if keyword == "version" {
                        let ver = parse_opt_arg::<Expr>(input)?
                            .unwrap_or_else(|| parse_quote!(env!("CARGO_PKG_VERSION")));
                        res.version = Some(Box::new(ver));
                    } else if keyword == "adjacent" {
                        res.adjacent = true;
                    } else if keyword == "boxed" {
                        res.boxed = true;
                    } else if keyword == "command" {
                        let name = parse_opt_arg::<LitStr>(input)?.unwrap_or_else(|| {
                            let n = to_snake_case(&outer_ty.to_string());
                            LitStr::new(&n, outer_ty.span())
                        });
                        res.kind = Some(OuterKind::Command(CommandAttr {
                            name,
                            shorts: Vec::new(),
                            longs: Vec::new(),
                            usage: None,
                        }));
                    } else if keyword == "usage" {
                        let content;
                        let _ = parenthesized!(content in input);
                        let lit = Some(content.parse::<LitStr>()?);
                        if let Some(OuterKind::Command(cmd)) = &mut res.kind {
                            cmd.usage = lit;
                        } else {
                            usage = lit;
                        }
                    } else if keyword == "short" {
                        // those are aliaes, no fancy name figuring out logic
                        let lit = parse_lit_char(input)?;
                        res.shorts.push(lit);
                    } else if keyword == "long" {
                        // those are aliaes, no fancy name figuring out logic
                        let lit = parse_lit_str(input)?;
                        res.longs.push(lit);
                    } else if keyword == "private" {
                        res.vis = Visibility::Inherited;
                    } else if keyword == "fallback" {
                        let fallback = parse_expr(input)?;
                        res.fallback = Some(fallback);
                    } else {
                        return Err(input_copy.error("Unexpected attribute"));
                    }
                    if !input.is_empty() {
                        input.parse::<token::Comma>()?;
                    }
                })?;
            }
        }
        if let Some(OuterKind::Command(cmd)) = &mut res.kind {
            cmd.shorts.append(&mut res.shorts);
            cmd.longs.append(&mut res.longs);
        }

        res.decor = Decor::new(&help, res.version.take(), usage);

        Ok(res)
    }
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

fn decorate_with_kind(outer: Outer, inner: BParser) -> ParserKind {
    let inner = if let Some(comp_style) = outer.comp_style {
        BParser::CompStyle(comp_style, Box::new(inner))
    } else {
        inner
    };

    match outer.kind.unwrap_or(OuterKind::Construct) {
        OuterKind::Construct => ParserKind::BParser(inner, outer.decor.descr),
        OuterKind::Options(maybe_cargo) => {
            let inner = match maybe_cargo {
                Some(cargo) => BParser::CargoHelper(cargo, Box::new(inner)),
                None => inner,
            };
            ParserKind::OParser(OParser {
                decor: outer.decor,
                inner: Box::new(inner),
            })
        }
        OuterKind::Command(cmd_attr) => {
            let oparser = OParser {
                decor: outer.decor,
                inner: Box::new(inner),
            };

            let cmd = BParser::Command {
                attribute: cmd_attr,
                inner_parser: Box::new(oparser),
                is_adjacent: outer.adjacent,
                is_hidden: false,
            };
            ParserKind::BParser(cmd, None)
        }
    }
}

impl Top {
    fn parse_struct(attrs: Vec<Attribute>, vis: Visibility, input: ParseStream) -> Result<Self> {
        let outer_ty = input.parse::<Ident>()?;
        let outer = Outer::make(&outer_ty, vis, attrs)?;

        let fields = input.parse::<Fields>()?;

        if fields.struct_definition_followed_by_semi() {
            input.parse::<Token![;]>()?;
        }

        let constr = ConstrName {
            namespace: None,
            constr: outer_ty.clone(),
            fallback: outer.fallback.clone(),
        };
        let inner = BParser::Constructor {
            name: constr,
            fields,
            adjacent: outer.adjacent,
            boxed: outer.boxed,
        };
        Ok(Top {
            name: outer
                .generate
                .clone()
                .unwrap_or_else(|| snake_case_ident(&outer_ty)),
            vis: outer.vis.clone(),
            kind: decorate_with_kind(outer, inner),
            outer_ty,
        })
    }

    fn parse_enum(attrs: Vec<Attribute>, vis: Visibility, input: ParseStream) -> Result<Self> {
        let outer_ty = input.parse::<Ident>()?;
        let outer = Outer::make(&outer_ty, vis, attrs)?;

        let mut branches: Vec<BParser> = Vec::new();

        let enum_contents;
        let _ = braced!(enum_contents in input);
        loop {
            if enum_contents.is_empty() {
                break;
            }
            let attrs = enum_contents.call(Attribute::parse_outer)?;
            let inner_ty = enum_contents.parse::<Ident>()?;
            let inner = Inner::make(&inner_ty, attrs.clone())?;

            let constr = ConstrName {
                namespace: Some(outer_ty.clone()),
                constr: inner_ty,
                fallback: None,
            };

            let branch = if enum_contents.peek(token::Paren) || enum_contents.peek(token::Brace) {
                let fields = Fields::parse(&enum_contents)?;
                BParser::Constructor {
                    name: constr,
                    fields,
                    adjacent: outer.adjacent,
                    boxed: outer.boxed,
                }
            } else if let Some(_cmd) = &inner.command {
                BParser::Constructor {
                    name: constr,
                    fields: Fields::NoFields,
                    adjacent: outer.adjacent,
                    boxed: outer.boxed,
                }
            } else {
                let req_flag = ReqFlag::make(constr, inner.clone());
                BParser::Singleton(Box::new(req_flag))
            };

            if !inner.skip {
                if let Some(cmd_arg) = inner.command {
                    let decor = Decor::new(&inner.help, None, None);
                    let oparser = OParser {
                        inner: Box::new(branch),
                        decor,
                    };
                    let branch = BParser::Command {
                        attribute: cmd_arg,
                        inner_parser: Box::new(oparser),
                        is_adjacent: outer.adjacent,
                        is_hidden: inner.is_hidden,
                    };
                    branches.push(branch);
                } else {
                    branches.push(branch);
                }
            }

            if !enum_contents.is_empty() {
                enum_contents.parse::<token::Comma>()?;
            }
        }

        let inner = match branches.len() {
            0 => {
                return Err(syn::Error::new(
                    outer_ty.span(),
                    "Can't construct a parser from empty enum",
                ))
            }
            1 => branches.remove(0),
            _ => BParser::Fold(branches, outer.fallback.clone()),
        };

        Ok(Top {
            name: outer
                .generate
                .clone()
                .unwrap_or_else(|| snake_case_ident(&outer_ty)),
            vis: outer.vis.clone(),
            kind: decorate_with_kind(outer, inner),
            outer_ty,
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
            ParserKind::BParser(_, _) => quote!(impl ::bpaf::Parser<#outer_ty>),
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
            ParserKind::BParser(bp, None) => bp.to_tokens(tokens),
            ParserKind::BParser(bp, Some(msg)) => {
                bp.to_tokens(tokens);
                quote!(
                    .group_help(#msg)
                )
                .to_tokens(tokens);
            }
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
            BParser::Command {
                attribute: cmd_attr,
                inner_parser: oparser,
                is_adjacent,
                is_hidden,
            } => {
                let cmd_name = &cmd_attr.name;
                let mut names = quote!();
                for short in &cmd_attr.shorts {
                    names = quote!(#names .short(#short));
                }
                for long in &cmd_attr.longs {
                    names = quote!(#names .long(#long));
                }

                let usage = match &cmd_attr.usage {
                    Some(usage) => quote!(.usage(#usage)),
                    None => quote!(),
                };
                if let Some(msg) = &oparser.decor.descr {
                    quote!( {
                        let inner_cmd = #oparser #usage;
                        ::bpaf::command(#cmd_name, inner_cmd).help(#msg)#names
                    })
                } else {
                    quote!({
                        let inner_cmd = #oparser #usage;
                        ::bpaf::command(#cmd_name, inner_cmd)#names
                    })
                }
                .to_tokens(tokens);

                if *is_adjacent {
                    quote!(.adjacent()).to_tokens(tokens);
                }
                if *is_hidden {
                    quote!(.hide()).to_tokens(tokens);
                }
            }
            BParser::CargoHelper(name, inner) => quote!({
                ::bpaf::cargo_helper(#name, #inner)
            })
            .to_tokens(tokens),
            BParser::Constructor {
                name,
                fields,
                adjacent,
                boxed,
            } => {
                if matches!(fields, Fields::NoFields) {
                    quote!(::bpaf::pure(#name)).to_tokens(tokens);
                } else {
                    let parse_decls = fields.parser_decls();
                    quote!({
                        #(#parse_decls)*
                        ::bpaf::construct!(#name #fields)
                    })
                    .to_tokens(tokens);
                }
                if *adjacent {
                    quote!(.adjacent()).to_tokens(tokens);
                }
                if let Some(fallback) = &name.fallback {
                    quote!(.fallback(#fallback)).to_tokens(tokens);
                }
                if *boxed {
                    quote!(.boxed()).to_tokens(tokens);
                }
            }
            BParser::Fold(xs, mfallback) => {
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
                if let Some(fallback) = mfallback {
                    quote!(.fallback(#fallback)).to_tokens(tokens);
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

    fn struct_definition_followed_by_semi(&self) -> bool {
        match self {
            Fields::Named(_) | Fields::NoFields => false,
            Fields::Unnamed(_) => true,
        }
    }
}

impl Decor {
    fn new(help: &[String], version: Option<Box<Expr>>, usage: Option<LitStr>) -> Self {
        let mut iter = LineIter::from(help);
        Decor {
            descr: iter.next(),
            header: iter.next(),
            footer: iter.next(),
            version,
            usage,
        }
    }
}
