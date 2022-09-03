use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    parenthesized, parse, parse_quote, token, Attribute, Expr, Ident, LitChar, LitStr,
    PathArguments, Result, Token, Type, Visibility,
};

use crate::kw;
use crate::utils::to_kebab_case;

#[derive(Debug)]
pub struct ConstrName {
    pub namespace: Option<Ident>,
    pub constr: Ident,
}

mod named_field;
mod req_flag;

pub use self::named_field::Field;
pub use req_flag::ReqFlag;

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct FieldAttrs<T> {
    external: Option<ExtAttr>,
    name: Option<Ident>,
    naming: Vec<T>,
    consumer: Option<ConsumerAttr>,
    postpr: Vec<PostprAttr>,
    help: Option<String>,
}

impl<T> Default for FieldAttrs<T> {
    fn default() -> Self {
        Self {
            external: None,
            name: None,
            naming: Vec::new(),
            consumer: None,
            postpr: Vec::new(),
            help: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OptNameAttr {
    Short(Option<LitChar>),
    Long(Option<LitStr>),
    Env(Box<Expr>),
}
#[derive(Debug, Clone)]
pub enum StrictNameAttr {
    Short(LitChar),
    Long(LitStr),
    Env(Box<Expr>),
}

impl StrictNameAttr {
    fn is_name(&self) -> bool {
        match self {
            StrictNameAttr::Short(_) | StrictNameAttr::Long(_) => true,
            StrictNameAttr::Env(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
enum ConsumerAttr {
    Arg(LitStr),
    ArgOs(LitStr),
    Pos(LitStr),
    PosOs(LitStr),
    Switch,
    Flag(Box<Expr>, Box<Expr>), // incomplete
}

#[derive(Debug, Clone)]
enum PostprAttr {
    FromStr(Box<Type>),
    Guard(Ident, Box<Expr>),
    Many(Option<LitStr>),
    Map(Ident),
    Optional,
    Parse(Ident),
    Fallback(Box<Expr>),
    FallbackWith(Box<Expr>),
    Complete(Ident),
    // used for deriving stuff to express map to convert
    // from OsString to PathBuf... I wonder.
    Tokens(TokenStream),
    Hide,
    GroupHelp(Box<Expr>),
}

impl PostprAttr {
    const fn can_derive(&self) -> bool {
        match self {
            PostprAttr::Many(_)
            | PostprAttr::FromStr(_)
            | PostprAttr::Map(_)
            | PostprAttr::Tokens(_)
            | PostprAttr::Optional
            | PostprAttr::Parse(_) => false,
            PostprAttr::Guard(_, _)
            | PostprAttr::Fallback(_)
            | PostprAttr::FallbackWith(_)
            | PostprAttr::Complete(_)
            | PostprAttr::Hide
            | PostprAttr::GroupHelp(_) => true,
        }
    }
}

fn comma(input: parse::ParseStream) -> Result<()> {
    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

impl<T: Parse> Parse for FieldAttrs<T> {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut naming = Vec::new();
        let mut consumer = None;
        let mut postpr = Vec::new();
        let mut external = None;
        if input.peek(kw::external) {
            external = Some(input.parse::<ExtAttr>()?);
            comma(input)?;
        } else {
            // we are parsing arguments twice here, syn docs explicitly asks us not to
            // This is fine since field attributes should be only a few tokens at most
            while input.fork().parse::<T>().is_ok() {
                naming.push(input.parse::<T>()?);
                comma(input)?;
            }
            if input.fork().parse::<ConsumerAttr>().is_ok() {
                consumer = Some(input.parse()?);
                comma(input)?;
            }
        }
        while !input.is_empty() {
            postpr.push(input.parse::<PostprAttr>()?);
            if !input.is_empty() {
                comma(input)?;
            }
        }

        Ok(FieldAttrs {
            external,
            naming,
            consumer,
            postpr,
            // those two are filled in during postprocessing
            name: None,
            help: None,
        })
    }
}

#[derive(Debug, Clone)]
struct ExtAttr {
    ident: Option<Ident>,
}

impl Parse for ExtAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let keyword = input.parse::<Ident>()?;
        if keyword == "external" {
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                Ok(Self {
                    ident: Some(content.parse::<Ident>()?),
                })
            } else {
                Ok(Self { ident: None })
            }
        } else {
            Err(input_copy.error("Not a name attribute"))
        }
    }
}

impl Parse for OptNameAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let keyword = input.parse::<Ident>()?;
        let content;
        if keyword == "long" {
            if input.peek(token::Paren) {
                let _ = parenthesized!(content in input);
                Ok(Self::Long(Some(content.parse::<LitStr>()?)))
            } else {
                Ok(Self::Long(None))
            }
        } else if keyword == "short" {
            if input.peek(token::Paren) {
                let _ = parenthesized!(content in input);
                Ok(Self::Short(Some(content.parse::<LitChar>()?)))
            } else {
                Ok(Self::Short(None))
            }
        } else if keyword == "env" {
            let _ = parenthesized!(content in input);
            Ok(Self::Env(Box::new(content.parse::<Expr>()?)))
        } else {
            Err(input_copy.error("Not a name attribute"))
        }
    }
}

impl Parse for StrictNameAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let keyword = input.parse::<Ident>()?;
        let content;
        if keyword == "long" {
            let _ = parenthesized!(content in input);
            Ok(Self::Long(content.parse::<LitStr>()?))
        } else if keyword == "short" {
            let _ = parenthesized!(content in input);
            Ok(Self::Short(content.parse::<LitChar>()?))
        } else {
            Err(input_copy.error("Not a name attribute"))
        }
    }
}

fn parse_optional_arg(input: parse::ParseStream) -> Result<LitStr> {
    let content;
    if input.peek(syn::token::Paren) {
        let _ = parenthesized!(content in input);
        content.parse::<LitStr>()
    } else {
        Ok(LitStr::new("ARG", Span::call_site()))
    }
}
impl Parse for ConsumerAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let keyword = input.parse::<Ident>()?;
        if keyword == "argument" {
            Ok(Self::Arg(parse_optional_arg(input)?))
        } else if keyword == "argument_os" {
            Ok(Self::ArgOs(parse_optional_arg(input)?))
        } else if keyword == "positional" {
            Ok(Self::Pos(parse_optional_arg(input)?))
        } else if keyword == "positional_os" {
            Ok(Self::PosOs(parse_optional_arg(input)?))
        } else if keyword == "switch" {
            Ok(Self::Switch)
        } else if keyword == "flag" {
            let content;
            let _ = parenthesized!(content in input);
            let a = content.parse()?;
            content.parse::<token::Comma>()?;
            let b = content.parse()?;
            Ok(Self::Flag(Box::new(a), Box::new(b)))
        } else {
            Err(input_copy.error("Not a consumer attribute"))
        }
    }
}

impl Parse for PostprAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let content;
        let keyword = input.parse::<Ident>()?;

        if keyword == "guard" {
            let _ = parenthesized!(content in input);
            let guard_fn = content.parse::<Ident>()?;
            let _ = content.parse::<Token![,]>()?;
            let msg = content.parse::<Expr>()?;
            Ok(Self::Guard(guard_fn, Box::new(msg)))
        } else if keyword == "fallback" {
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::Fallback(Box::new(expr)))
        } else if keyword == "fallback_with" {
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::FallbackWith(Box::new(expr)))
        } else if keyword == "parse" {
            let _ = parenthesized!(content in input);
            let parse_fn = content.parse::<Ident>()?;
            Ok(Self::Parse(parse_fn))
        } else if keyword == "map" {
            let _ = parenthesized!(content in input);
            let map_fn = content.parse::<Ident>()?;
            Ok(Self::Map(map_fn))
        } else if keyword == "from_str" {
            let _ = parenthesized!(content in input);
            let ty = content.parse::<Type>()?;
            Ok(Self::FromStr(Box::new(ty)))
        } else if keyword == "complete" {
            let _ = parenthesized!(content in input);
            let f = content.parse::<Ident>()?;
            Ok(Self::Complete(f))
        } else if keyword == "many" {
            Ok(Self::Many(None))
        } else if keyword == "some" {
            let _ = parenthesized!(content in input);
            Ok(Self::Many(Some(content.parse::<LitStr>()?)))
        } else if keyword == "optional" {
            Ok(Self::Optional)
        } else if keyword == "hide" {
            Ok(Self::Hide)
        } else if keyword == "group_help" {
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::GroupHelp(Box::new(expr)))
        } else {
            Err(input_copy.error("Not a valid postprocessing attribute"))
        }
    }
}

pub struct Doc(pub String);
impl Parse for Doc {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        input.parse::<Token![=]>()?;
        let mut s = input.parse::<LitStr>()?.value();
        if let Some(rest) = s.strip_prefix(' ') {
            s = rest.to_string();
        }
        Ok(Doc(s))
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Shape {
    Optional(Type),
    Multiple(Type),
    Bool,
    Direct(Type),
}

fn split_type(ty: &Type) -> Shape {
    fn single_arg(x: &PathArguments) -> Option<Type> {
        match x {
            PathArguments::AngleBracketed(arg) => {
                if arg.args.len() == 1 {
                    match &arg.args[0] {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn try_split_type(ty: &Type) -> Option<Shape> {
        let last = match ty {
            Type::Path(p) => p.path.segments.last()?,
            _ => return None,
        };
        if last.ident == "Vec" {
            Some(Shape::Multiple(single_arg(&last.arguments)?))
        } else if last.ident == "Option" {
            Some(Shape::Optional(single_arg(&last.arguments)?))
        } else if last.ident == "bool" {
            Some(Shape::Bool)
        } else {
            None
        }
    }
    try_split_type(ty).unwrap_or_else(|| Shape::Direct(ty.clone()))
}

fn is_os_str_ty(ty: &Type) -> bool {
    ty == &parse_quote!(PathBuf)
        || ty == &parse_quote!(OsString)
        || ty == &parse_quote!(std::path::PathBuf)
        || ty == &parse_quote!(std::ffi::OsString)
}

impl Field {
    pub fn parse_unnamed(input: parse::ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let ty = input.parse::<Type>()?;
        Field::make(ty, None, attrs)
    }

    pub fn parse_named(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        Field::make(ty, Some(name), attrs)
    }
}

pub fn as_short_name(value: &Ident) -> LitChar {
    let name_str = value.to_string();
    LitChar::new(
        name_str.chars().next().unwrap().to_ascii_lowercase(),
        value.span(),
    )
}

pub fn as_long_name(value: &Ident) -> LitStr {
    let kebabed_name = to_kebab_case(&value.to_string());
    LitStr::new(&kebabed_name, value.span())
}

pub fn fill_in_name(value: &Ident, names: &mut Vec<StrictNameAttr>) {
    if !names.iter().any(StrictNameAttr::is_name) {
        names.push(if value.to_string().chars().nth(1).is_some() {
            StrictNameAttr::Long(as_long_name(value))
        } else {
            StrictNameAttr::Short(as_short_name(value))
        })
    }
}

impl ToTokens for FieldAttrs<StrictNameAttr> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut first = true;
        if let Some(ext) = &self.external {
            let name = ext.ident.as_ref().or(self.name.as_ref()).unwrap();
            quote!(#name()).to_tokens(tokens);
        } else {
            if first {
                quote!(::bpaf::).to_tokens(tokens);
            }

            for naming in &self.naming {
                if !first {
                    quote!(.).to_tokens(tokens);
                }
                naming.to_tokens(tokens);
                first = false;
            }
            if let Some(help) = &self.help {
                // For named things help goes right after the name
                if !first {
                    quote!(.help(#help)).to_tokens(tokens);
                }
            }
            if let Some(cons) = &self.consumer {
                if !first {
                    quote!(.).to_tokens(tokens);
                }
                cons.to_tokens(tokens);
            }
            if let Some(help) = &self.help {
                // For positional things help goes right after the consumer
                if first {
                    quote!(.help(#help)).to_tokens(tokens);
                }
            }
        }
        for postpr in &self.postpr {
            quote!(.#postpr).to_tokens(tokens);
        }
    }
}

impl ToTokens for PostprAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PostprAttr::FromStr(ty) => quote!(from_str::<#ty>()),
            PostprAttr::Guard(f, m) => quote!(guard(#f, #m)),
            PostprAttr::Many(None) => quote!(many()),
            PostprAttr::Many(Some(m)) => quote!(some(#m)),
            PostprAttr::Map(f) => quote!(map(#f)),
            PostprAttr::Optional => quote!(optional()),
            PostprAttr::Parse(f) => quote!(parse(#f)),
            PostprAttr::Fallback(v) => quote!(fallback(#v)),
            PostprAttr::FallbackWith(v) => quote!(fallback_with(#v)),
            PostprAttr::Tokens(t) => quote!(#t),
            PostprAttr::Hide => quote!(hide()),
            PostprAttr::Complete(f) => quote!(complete(#f)),
            PostprAttr::GroupHelp(m) => quote!(group_help(#m)),
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for StrictNameAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StrictNameAttr::Short(s) => quote!(short(#s)),
            StrictNameAttr::Long(l) => quote!(long(#l)),
            StrictNameAttr::Env(e) => quote!(env(#e)),
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for ConsumerAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ConsumerAttr::Arg(arg) => quote!(argument(#arg)),
            ConsumerAttr::ArgOs(arg) => quote!(argument_os(#arg)),
            ConsumerAttr::Pos(arg) => quote!(positional(#arg)),
            ConsumerAttr::PosOs(arg) => quote!(positional_os(#arg)),
            ConsumerAttr::Switch => quote!(switch()),
            ConsumerAttr::Flag(a, b) => quote!(flag(#a, #b)),
        }
        .to_tokens(tokens);
    }
}
