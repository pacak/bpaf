use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse, parse_quote, token, Attribute, Expr, Ident, LitChar, LitStr,
    PathArguments, Result, Token, Type, Visibility,
};

use crate::kw;
use crate::top::split_help_and;
use crate::utils::{to_kebab_case, LineIter};

#[derive(Debug)]
pub struct ConstrName {
    pub namespace: Option<Ident>,
    pub constr: Ident,
}

#[derive(Debug)]
pub struct ReqFlag {
    value: ConstrName,
    naming: Vec<StrictNameAttr>,
    help: Option<String>,
    is_hidden: bool,
    is_default: bool,
}

impl ReqFlag {
    pub fn new(value: ConstrName, attrs: Vec<EnumSingleton>, help: &[String]) -> Self {
        let mut is_hidden = false;
        let mut is_default = false;
        let mut names = Vec::new();
        for attr in attrs {
            match attr {
                EnumSingleton::IsDefault => is_default = true,
                EnumSingleton::Hidden => is_hidden = true,
                EnumSingleton::Short(short) => names.push(OptNameAttr::Short(short)),
                EnumSingleton::Long(long) => names.push(OptNameAttr::Long(long)),
                EnumSingleton::Env(env) => names.push(OptNameAttr::Env(env)),
            }
        }
        let naming = restrict_names(&value.constr, names);
        let help = LineIter::from(help).next();
        Self {
            value,
            naming,
            help,
            is_hidden,
            is_default,
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
pub enum EnumSingleton {
    Short(Option<LitChar>),
    Long(Option<LitStr>),
    Env(Box<Expr>),
    IsDefault,
    Hidden,
}

impl Parse for EnumSingleton {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_copy = input.fork();
        let keyword = input.parse::<Ident>()?;
        let content;
        if keyword == "hide" {
            Ok(Self::Hidden)
        } else if keyword == "default" {
            Ok(Self::IsDefault)
        } else if keyword == "long" {
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
            Err(input_copy.error("Not a valid enum singleton constructor attribute"))
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
            PostprAttr::FromStr(_)
            | PostprAttr::Many(_)
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

impl FieldParser {
    pub fn var_name(&self, ix: usize) -> Ident {
        let name = &self.name;
        match name {
            Some(name) => name.clone(),
            None => Ident::new(&format!("f{}", ix), Span::call_site()),
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

#[allow(clippy::module_name_repetitions)]
pub type FieldParser = FieldAttrs<StrictNameAttr>;

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

impl Shape {
    fn is_os_str(&self) -> bool {
        let ty = match self {
            Shape::Bool => return false,
            Shape::Optional(t) | Shape::Multiple(t) | Shape::Direct(t) => t,
        };
        ty == &parse_quote!(PathBuf)
            || ty == &parse_quote!(OsString)
            || ty == &parse_quote!(std::path::PathBuf)
            || ty == &parse_quote!(std::ffi::OsString)
    }
}

impl FieldParser {
    pub fn parse_unnamed(input: parse::ParseStream) -> Result<Self> {
        let i = input.fork();
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let ty = input.parse::<Type>()?;
        let (help, mut attrs) = split_help_and::<FieldAttrs<StrictNameAttr>>(&attrs)?;
        let mut parser = match attrs.len() {
            0 => FieldAttrs::<StrictNameAttr>::default(),
            1 => attrs.pop().unwrap(),
            _ => return Err(i.error("At most one bpaf annotation is expected")),
        };

        if let Some(err) = parser.implicit_consumer(&ty) {
            return Err(i.error(err));
        }

        if parser.naming.is_empty() && parser.consumer_needs_name() == Some(true) {
            return Err(i.error(
                "This consumer needs a name, you can specify it with long(\"name\") or short('n')",
            ));
        }
        if let Some(ext) = &parser.external {
            if ext.ident.is_none() {
                return Err(
                    i.error("Name shortcut for external attribute is only valid for named field")
                );
            }
        }

        parser.help = LineIter::from(&help[..]).next();
        Ok(parser)
    }

    pub fn parse_named(input: ParseStream) -> Result<Self> {
        let i = input.fork();
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        let (help, mut attrs) = split_help_and::<FieldAttrs<OptNameAttr>>(&attrs)?;
        let parser = match attrs.len() {
            0 => FieldAttrs::<OptNameAttr>::default(),
            1 => attrs.pop().unwrap(),
            _ => return Err(i.error("At most one bpaf annotation is expected")),
        };

        let strip_name = parser.naming.is_empty() && parser.consumer_needs_name() == Some(false);
        let mut parser = parser.implicit_name(name);
        if strip_name {
            parser.naming.clear();
        }
        if let Some(err) = parser.implicit_consumer(&ty) {
            return Err(i.error(err));
        }

        parser.help = LineIter::from(&help[..]).next();

        Ok(parser)
    }
}

impl FieldAttrs<OptNameAttr> {
    fn implicit_name(self, name: Ident) -> FieldAttrs<StrictNameAttr> {
        FieldAttrs {
            external: self.external,
            naming: restrict_names(&name, self.naming),
            consumer: self.consumer,
            postpr: self.postpr,
            help: self.help,
            name: Some(name),
        }
    }
}

fn restrict_names(base_name: &Ident, attrs: Vec<OptNameAttr>) -> Vec<StrictNameAttr> {
    let mut res = Vec::new();
    let name_str = {
        let s = base_name.to_string();
        if s.chars().next().unwrap().is_uppercase() {
            to_kebab_case(&s)
        } else {
            s.chars()
                .map(|c| if c == '_' { '-' } else { c })
                .collect::<String>()
        }
    };

    for name_attr in attrs {
        res.push(match name_attr {
            OptNameAttr::Short(Some(s)) => StrictNameAttr::Short(s),
            OptNameAttr::Long(Some(l)) => StrictNameAttr::Long(l),
            OptNameAttr::Short(None) => {
                let s = LitChar::new(name_str.chars().next().unwrap(), base_name.span());
                StrictNameAttr::Short(s)
            }
            OptNameAttr::Long(None) => {
                let l = LitStr::new(&name_str, base_name.span());
                StrictNameAttr::Long(l)
            }
            OptNameAttr::Env(e) => StrictNameAttr::Env(e),
        });
    }

    if !res.iter().any(StrictNameAttr::is_name) {
        if name_str.chars().nth(1).is_some() {
            let l = LitStr::new(&name_str, base_name.span());
            res.push(StrictNameAttr::Long(l));
        } else {
            let c = LitChar::new(name_str.chars().next().unwrap(), base_name.span());
            res.push(StrictNameAttr::Short(c));
        }
    }
    res
}

impl FieldAttrs<StrictNameAttr> {
    fn implicit_consumer(&mut self, ty: &Type) -> Option<&'static str> {
        let arg = LitStr::new("ARG", ty.span());
        let shape = split_type(ty);
        let can_derive_postpr =
            self.external.is_none() && self.postpr.iter().all(PostprAttr::can_derive);

        let os_str = shape.is_os_str();
        let inner_ty = match shape {
            Shape::Bool => {
                return if self.naming.is_empty() {
                    Some("Can't parse bool as a positional attribute")
                } else {
                    self.consumer = Some(ConsumerAttr::Switch);
                    None
                }
            }
            Shape::Direct(ty) => ty,
            Shape::Optional(ty) => {
                if can_derive_postpr {
                    self.postpr.insert(0, PostprAttr::Optional);
                }
                ty
            }
            Shape::Multiple(ty) => {
                if can_derive_postpr {
                    self.postpr.insert(0, PostprAttr::Many(None));
                }
                ty
            }
        };

        if self.consumer.is_none() && self.external.is_none() {
            if !can_derive_postpr {
                return Some(
                "Can't derive consumer for this element, try specifying `argument(\"arg\")` or `argument_os(\"arg\")`"
            );
            }
            self.consumer = Some(match (os_str, self.naming.is_empty()) {
                (true, true) => ConsumerAttr::PosOs(arg),
                (true, false) => ConsumerAttr::ArgOs(arg),
                (false, true) => ConsumerAttr::Pos(arg),
                (false, false) => ConsumerAttr::Arg(arg),
            });
        }

        if can_derive_postpr && self.external.is_none() {
            if os_str {
                let attr = PostprAttr::Tokens(quote!(map(#inner_ty::from)));
                self.postpr.insert(0, attr);
            } else if inner_ty != parse_quote!(String) {
                let attr = PostprAttr::FromStr(Box::new(inner_ty));
                self.postpr.insert(0, attr);
            }
        }

        None
    }
}
impl<T> FieldAttrs<T> {
    fn consumer_needs_name(&self) -> Option<bool> {
        Some(match self.consumer.as_ref()? {
            ConsumerAttr::Arg(_)
            | ConsumerAttr::ArgOs(_)
            | ConsumerAttr::Switch
            | ConsumerAttr::Flag(_, _) => true,
            ConsumerAttr::Pos(_) | ConsumerAttr::PosOs(_) => false,
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
