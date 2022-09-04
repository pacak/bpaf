use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    parenthesized, parse, parse_quote, Attribute, Expr, Ident, LitChar, LitStr, PathArguments,
    Result, Token, Type, Visibility,
};

use crate::utils::to_kebab_case;

#[derive(Debug)]
pub struct ConstrName {
    pub namespace: Option<Ident>,
    pub constr: Ident,
}

mod named_field;
mod req_flag;

pub use self::named_field::Field;
pub use self::named_field::*;
pub use req_flag::ReqFlag;

#[derive(Debug, Clone)]
pub enum Name {
    Short(LitChar),
    Long(LitStr),
}

#[derive(Debug, Clone)]
enum ConsumerAttr {
    Arg(LitStr),
    ArgOs(LitStr),
    Pos(LitStr),
    PosOs(LitStr),
    Switch,
    Flag(Box<Expr>, Box<Expr>),
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

fn parse_optional_arg(input: parse::ParseStream) -> Result<LitStr> {
    let content;
    if input.peek(syn::token::Paren) {
        let _ = parenthesized!(content in input);
        content.parse::<LitStr>()
    } else {
        Ok(LitStr::new("ARG", Span::call_site()))
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

pub fn fill_in_name(value: &Ident, names: &mut Vec<Name>) {
    if names.is_empty() {
        names.push(if value.to_string().chars().nth(1).is_some() {
            Name::Long(as_long_name(value))
        } else {
            Name::Short(as_short_name(value))
        })
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

impl ToTokens for Name {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Name::Short(s) => quote!(short(#s)),
            Name::Long(l) => quote!(long(#l)),
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
