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
}

impl ReqFlag {
    pub fn new(value: ConstrName, names: Vec<OptNameAttr>, help: &[String]) -> Self {
        let naming = restrict_names(&value.constr, names);
        let help = LineIter::from(help).next();
        Self {
            value,
            naming,
            help,
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
        quote!(.req_flag(#value)).to_tokens(tokens);
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
pub enum OptNameAttr {
    Short(Option<LitChar>),
    Long(Option<LitStr>),
}
#[derive(Debug, Clone)]
pub enum StrictNameAttr {
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
}

#[derive(Debug, Clone)]
enum PostprAttr {
    FromStr(Box<Type>),
    Guard(Ident, LitStr),
    Many(Option<LitStr>),
    Map(Ident),
    Optional,
    Parse(Ident),
    Fallback(Box<Expr>),
    FallbackWith(Box<Expr>),
    Tokens(TokenStream),
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
            PostprAttr::Guard(_, _) | PostprAttr::Fallback(_) | PostprAttr::FallbackWith(_) => true,
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
        if let Ok(ext) = input.parse::<ExtAttr>() {
            external = Some(ext);
            comma(input)?;
        } else {
            while let Ok(nam) = input.parse() {
                naming.push(nam);
                comma(input)?;
            }
            if let Ok(cons) = input.parse() {
                consumer = Some(cons);
                comma(input)?;
            }
        }
        while let Ok(p) = input.parse() {
            postpr.push(p);
            comma(input)?;
        }
        if !input.is_empty() {
            return Err(input.error(format!("Can't parse remaining attributes: {}", input)));
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
        if input.peek(kw::external) {
            input.parse::<kw::external>()?;
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
            Err(input.error("Not a name attribute"))
        }
    }
}

impl Parse for OptNameAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(kw::long) {
            input.parse::<kw::long>()?;
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                Ok(Self::Long(Some(content.parse::<LitStr>()?)))
            } else {
                Ok(Self::Long(None))
            }
        } else if input.peek(kw::short) {
            let _ = input.parse::<kw::short>()?;
            if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                Ok(Self::Short(Some(content.parse::<LitChar>()?)))
            } else {
                Ok(Self::Short(None))
            }
        } else {
            Err(input.error("Not a name attribute"))
        }
    }
}

impl Parse for StrictNameAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(kw::long) {
            input.parse::<kw::long>()?;
            let content;
            let _ = parenthesized!(content in input);
            Ok(Self::Long(content.parse::<LitStr>()?))
        } else if input.peek(kw::short) {
            let _ = input.parse::<kw::short>()?;
            let content;
            let _ = parenthesized!(content in input);
            Ok(Self::Short(content.parse::<LitChar>()?))
        } else {
            Err(input.error("Not a name attribute"))
        }
    }
}

impl Parse for ConsumerAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let parse_arg = |input: &parse::ParseBuffer| {
            let content;
            if input.peek(syn::token::Paren) {
                let _ = parenthesized!(content in input);
                content.parse::<LitStr>()
            } else {
                Ok(LitStr::new("ARG", Span::call_site()))
            }
        };
        if input.peek(kw::argument) {
            input.parse::<kw::argument>()?;
            Ok(Self::Arg(parse_arg(input)?))
        } else if input.peek(kw::argument_os) {
            input.parse::<kw::argument_os>()?;
            Ok(Self::ArgOs(parse_arg(input)?))
        } else if input.peek(kw::positional) {
            input.parse::<kw::positional>()?;
            Ok(Self::Pos(parse_arg(input)?))
        } else if input.peek(kw::positional_os) {
            input.parse::<kw::positional_os>()?;
            Ok(Self::PosOs(parse_arg(input)?))
        } else if input.peek(kw::switch) {
            input.parse::<kw::switch>()?;
            Ok(Self::Switch)
        } else {
            Err(input.error("Not a consumer attribute"))
        }
    }
}

impl Parse for PostprAttr {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let content;
        if input.peek(kw::guard) {
            input.parse::<kw::guard>()?;
            let _ = parenthesized!(content in input);
            let guard_fn = content.parse::<Ident>()?;
            let _ = content.parse::<Token![,]>()?;
            let msg = content.parse::<LitStr>()?;
            Ok(Self::Guard(guard_fn, msg))
        } else if input.peek(kw::fallback) {
            input.parse::<kw::fallback>()?;
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::Fallback(Box::new(expr)))
        } else if input.peek(kw::fallback_with) {
            input.parse::<kw::fallback_with>()?;
            let _ = parenthesized!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(Self::FallbackWith(Box::new(expr)))
        } else if input.peek(kw::parse) {
            input.parse::<kw::parse>()?;
            let _ = parenthesized!(content in input);
            let parse_fn = content.parse::<Ident>()?;
            Ok(Self::Parse(parse_fn))
        } else if input.peek(kw::map) {
            input.parse::<kw::map>()?;
            let _ = parenthesized!(content in input);
            let map_fn = content.parse::<Ident>()?;
            Ok(Self::Map(map_fn))
        } else if input.peek(kw::from_str) {
            input.parse::<kw::from_str>()?;
            let _ = parenthesized!(content in input);
            let ty = content.parse::<Type>()?;
            Ok(Self::FromStr(Box::new(ty)))
        } else if input.peek(kw::many) {
            input.parse::<kw::many>()?;
            Ok(Self::Many(None))
        } else if input.peek(kw::some) {
            input.parse::<kw::some>()?;
            let _ = parenthesized!(content in input);
            Ok(Self::Many(Some(content.parse::<LitStr>()?)))
        } else if input.peek(kw::optional) {
            input.parse::<kw::optional>()?;
            Ok(Self::Optional)
        } else {
            Err(input.error("Not a attribute"))
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
    if attrs.is_empty() {
        if name_str.chars().nth(1).is_some() {
            let l = LitStr::new(&name_str, base_name.span());
            res.push(StrictNameAttr::Long(l));
        } else {
            let c = LitChar::new(name_str.chars().next().unwrap(), base_name.span());
            res.push(StrictNameAttr::Short(c));
        }
    } else {
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
            });
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
            ConsumerAttr::Arg(_) | ConsumerAttr::ArgOs(_) | ConsumerAttr::Switch => true,
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
                // help only makes sense for named things
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
        }
        for postpr in &self.postpr {
            quote!(.).to_tokens(tokens);
            postpr.to_tokens(tokens);
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
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for StrictNameAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StrictNameAttr::Short(s) => quote!(short(#s)),
            StrictNameAttr::Long(l) => quote!(long(#l)),
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
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse2, parse_quote};

    #[derive(Debug, Clone)]
    struct UnnamedField {
        parser: FieldParser,
    }

    impl Parse for UnnamedField {
        fn parse(input: parse::ParseStream) -> Result<Self> {
            let parser = FieldParser::parse_unnamed(input)?;
            Ok(Self { parser })
        }
    }

    impl ToTokens for UnnamedField {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            self.parser.to_tokens(tokens)
        }
    }

    #[derive(Debug, Clone)]
    struct NamedField {
        parser: FieldParser,
    }

    impl Parse for NamedField {
        fn parse(input: parse::ParseStream) -> Result<Self> {
            let parser = FieldParser::parse_named(input)?;
            Ok(Self { parser })
        }
    }

    impl ToTokens for NamedField {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            self.parser.to_tokens(tokens)
        }
    }

    #[track_caller]
    fn field_trans_fail(input: TokenStream, expected_err: &str) {
        let err = syn::parse2::<NamedField>(input).unwrap_err().to_string();
        assert_eq!(err, expected_err)
    }

    #[test]
    fn implicit_parser() {
        let input: NamedField = parse_quote! {
            /// help
            number: usize
        };
        let output = quote! {
            ::bpaf::long("number").help("help").argument("ARG").from_str::<usize>()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn short_long() {
        let input: NamedField = parse_quote! {
            #[bpaf(short, long)]
            number: usize
        };
        let output = quote! {
            ::bpaf::short('n').long("number").argument("ARG").from_str::<usize>()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_fallback() {
        let input: NamedField = parse_quote! {
            #[bpaf(fallback(3.1415))]
            number: f64
        };
        let output = quote! {
            ::bpaf::long("number").argument("ARG").from_str::<f64>().fallback(3.1415)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_fallback_with() {
        let input: NamedField = parse_quote! {
            #[bpaf(fallback_with(external))]
            number: f64
        };
        let output = quote! {
            ::bpaf::long("number").argument("ARG").from_str::<f64>().fallback_with(external)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_external_help() {
        let input: NamedField = parse_quote! {
            /// help
            #[bpaf(external(level))]
            number: f64
        };
        let output = quote! {
            level()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_external_no_help() {
        let input: NamedField = parse_quote! {
            #[bpaf(external(level))]
            number: f64
        };
        let output = quote! {
            level()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_external_nohelp() {
        let input: NamedField = parse_quote! {
            /// help
            #[bpaf(external(level))]
            number: f64
        };
        let output = quote! {
            level()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_field_guard() {
        let input: NamedField = parse_quote! {
            #[bpaf(guard(positive, "msg"))]
            number: usize
        };
        let output = quote! {
            ::bpaf::long("number").argument("ARG").from_str::<usize>().guard(positive, "msg")
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn derive_help() {
        let input: NamedField = parse_quote! {
            /// multi
            ///
            /// vis
            ///
            ///
            /// hidden
            pub(crate) flag: bool
        };
        let output = quote! {
            ::bpaf::long("flag").help("multi\nvis").switch()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn map_requires_explicit_parser() {
        let input: NamedField = parse_quote! {
            #[bpaf(argument("NUM"), from_str(usize), map(double))]
            number: usize
        };
        let output = quote! {
            ::bpaf::long("number").argument("NUM").from_str::<usize>().map(double)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn map_requires_explicit_parser2() {
        let input = quote! {
            #[bpaf(map(double))]
            pub number: usize
        };
        let err = "Can't derive consumer for this element, try specifying `argument(\"arg\")` or `argument_os(\"arg\")`";
        field_trans_fail(input, err);
    }

    #[test]
    fn check_guard() {
        let input: UnnamedField = parse_quote! {
            #[bpaf(guard(odd, "must be odd"))]
            usize
        };

        let output = quote! {
            ::bpaf::positional("ARG").from_str::<usize>().guard(odd, "must be odd")
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn check_fallback() {
        let input: NamedField = parse_quote! {
            #[bpaf(argument("SPEED"), fallback(42.0))]
            speed: f64
        };
        let output = quote! {
            ::bpaf::long("speed").argument("SPEED").from_str::<f64>().fallback(42.0)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn check_many_files_implicit() {
        let input: NamedField = parse_quote! {
            files: Vec<std::path::PathBuf>
        };
        let output = quote! {
            ::bpaf::long("files").argument_os("ARG").map(std::path::PathBuf::from).many()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn check_option_file_implicit() {
        let input: NamedField = parse_quote! {
            files: Option<PathBuf>
        };
        let output = quote! {
            ::bpaf::long("files").argument_os("ARG").map(PathBuf::from).optional()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn check_guard_fallback() {
        let input: NamedField = parse_quote! {
            #[bpaf(guard(positive, "must be positive"), fallback(1))]
            num: u32
        };
        let output = quote! {
            ::bpaf::long("num").argument("ARG").from_str::<u32>().guard(positive, "must be positive").fallback(1)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn better_error_for_unnamed_argument() {
        let input = quote!(
            #[bpaf(argument("FILE"))]
            pub PathBuf
        );
        let err = parse2::<UnnamedField>(input).unwrap_err().to_string();
        assert_eq!(
            err,
            "This consumer needs a name, you can specify it with long(\"name\") or short('n')"
        );
    }

    #[test]
    fn postprocessing_after_external() {
        let input: NamedField = parse_quote! {
            #[bpaf(external(verbose), fallback(42))]
            verbose: usize
        };
        let output = quote! {
            verbose().fallback(42)
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_external() {
        let input: NamedField = parse_quote! {
            #[bpaf(external(verbose))]
            verbose: Option<String>
        };
        let output = quote! {
            verbose()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_external_shortcut() {
        let input: NamedField = parse_quote! {
            #[bpaf(external)]
            verbose: Option<String>
        };
        let output = quote! {
            verbose()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_external_unnamed() {
        let input: UnnamedField = parse_quote! {
            #[bpaf(external(verbose))]
            Option<String>
        };
        let output = quote! {
            verbose()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_field_is_sane() {
        let input: NamedField = parse_quote! {
            name: Option<String>
        };
        let output = quote! {
            ::bpaf::long("name").argument("ARG").optional()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn vec_field_is_sane() {
        let input: NamedField = parse_quote! {
            names: Vec<String>
        };
        let output = quote! {
            ::bpaf::long("names").argument("ARG").many()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn positional_named_fields() {
        let input: NamedField = parse_quote! {
            #[bpaf(positional("ARG"))]
            name: String
        };
        let output = quote! {
            ::bpaf::positional("ARG")
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_named_pathed() {
        let input: NamedField = parse_quote! {
            #[bpaf(long, short)]
            pub config: Option<aws::Location>
        };
        let output = quote! {
            ::bpaf::long("config")
                .short('c')
                .argument("ARG")
                .from_str::<aws::Location>()
                .optional()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_unnamed_pathed() {
        let input: UnnamedField = parse_quote! {
            #[bpaf(long("config"), short('c'))]
            Option<aws::Location>
        };
        let output = quote! {
            ::bpaf::long("config")
                .short('c')
                .argument("ARG")
                .from_str::<aws::Location>()
                .optional()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn optional_argument_with_name() {
        let input: NamedField = parse_quote! {
            #[bpaf(argument("N"))]
            config: Option<u64>
        };
        let output = quote! {
            ::bpaf::long("config")
                .argument("N")
                .from_str::<u64>()
                .optional()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn some_arguments() {
        let input: NamedField = parse_quote! {
            #[bpaf(argument("N"), from_str(u32), some("need params"))]
            config: Vec<u32>
        };
        let output = quote! {
            ::bpaf::long("config")
                .argument("N")
                .from_str::<u32>()
                .some("need params")
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn explicit_switch_argument() {
        let input: NamedField = parse_quote! {
            #[bpaf(switch)]
            item: bool
        };
        let output = quote! {
            ::bpaf::long("item").switch()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }

    #[test]
    fn implicit_switch_argument() {
        let input: NamedField = parse_quote! {
            #[bpaf(switch)]
            item: bool
        };
        let output = quote! {
            ::bpaf::long("item").switch()
        };
        assert_eq!(input.to_token_stream().to_string(), output.to_string());
    }
}
