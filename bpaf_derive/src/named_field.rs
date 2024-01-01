use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::ParseStream, parse_quote, spanned::Spanned, token, Attribute, Error, Ident, LitStr,
    Result, Type, Visibility,
};

use crate::{
    attrs::{
        parse_bpaf_doc_attrs, Consumer, FieldAttrs, HelpPlacement, Name, Post, PostParse,
        StrictName, TurboFish,
    },
    field::{split_type, Shape},
    help::Help,
    utils::to_snake_case,
};

#[derive(Debug, Clone)]
pub(crate) struct StructField {
    pub name: Option<Ident>,
    pub env: Vec<StrictName>,
    pub naming: Vec<StrictName>,
    pub cons: Consumer,
    pub postpr: Vec<Post>,
    pub help: Option<Help>,
}

fn derive_consumer(name_present: bool, ty: &Type) -> Result<Consumer> {
    let span = ty.span();
    Ok(match split_type(ty) {
        Shape::Bool => {
            if name_present {
                Consumer::Switch { span }
            } else {
                let msg = "Refusing to derive a positional item for bool, you can fix this by either adding a short/long name or making it positional explicitly";
                return Err(Error::new(ty.span(), msg));
            }
        }
        Shape::Unit => {
            if name_present {
                Consumer::ReqFlag {
                    present: parse_quote!(()),
                    span,
                }
            } else {
                let msg = "Refusing to derive a positional item for (), you can fix this by either adding a short/long name or making it positional explicitly";
                return Err(Error::new(ty.span(), msg));
            }
        }
        Shape::Optional(t) | Shape::Multiple(t) | Shape::Direct(t) => {
            let ty = Some(t);
            let metavar = None;
            if name_present {
                Consumer::Argument { metavar, ty, span }
            } else {
                Consumer::Positional { metavar, ty, span }
            }
        }
    })
}

struct MMetavar<'a>(Option<&'a LitStr>);
impl ToTokens for MMetavar<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.0 {
            Some(mv) => mv.to_tokens(tokens),
            None => quote!("ARG").to_tokens(tokens),
        }
    }
}

impl ToTokens for Consumer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Consumer::Switch { .. } => quote!(switch()),
            Consumer::Flag {
                present, absent, ..
            } => quote!(flag(#present, #absent)),
            Consumer::ReqFlag { present, .. } => quote!(req_flag(#present)),
            Consumer::Any {
                metavar, ty, check, ..
            } => match ty {
                Some(ty) => quote!(::bpaf::any::<#ty, _, _>(#metavar, #check)),
                None => quote!(::bpaf::any(#metavar, #check)),
            },
            Consumer::Argument { metavar, ty, .. } => {
                let metavar = MMetavar(metavar.as_ref());
                let tf = ty.as_ref().map(TurboFish);
                quote!(argument #tf(#metavar))
            }
            Consumer::Positional { metavar, ty, .. } => {
                let metavar = MMetavar(metavar.as_ref());
                let tf = ty.as_ref().map(TurboFish);
                quote!(::bpaf::positional #tf(#metavar))
            }
            Consumer::External { ident, .. } => {
                quote!(#ident())
            }
            Consumer::Pure { expr, .. } => {
                quote!(::bpaf::pure(#expr))
            }
            Consumer::PureWith { expr, .. } => {
                quote!(::bpaf::pure_with(#expr))
            }
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StructField {
            name: _,
            env,
            naming,
            cons,
            postpr,
            help,
        } = self;

        let names = naming.iter().chain(env.iter());

        let prefix = if cons.needs_name() {
            quote!(::bpaf::)
        } else {
            quote!()
        };

        let help = help.iter();

        match cons.help_placement() {
            HelpPlacement::AtName => {
                quote!(#prefix #( #names .)* #(help(#help).)* #cons #(.#postpr)*)
            }
            HelpPlacement::AtConsumer => {
                quote!(#prefix #( #names .)* #cons #(.help(#help))* #(.#postpr)*)
            }
            HelpPlacement::NotAvailable => quote!(#prefix #(#names.)* #cons #(.#postpr)*),
        }
        .to_tokens(tokens);
    }
}

impl StructField {
    pub fn var_name(&self, ix: usize) -> Ident {
        let name = &self.name;
        match name {
            Some(name) => name.clone(),
            None => Ident::new(&format!("f{}", ix), Span::call_site()),
        }
    }

    pub fn parse_named(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<token::Colon>()?;
        let ty = input.parse::<Type>()?;
        Self::make(Some(name), ty, &attrs)
    }

    pub fn parse_unnamed(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _vis = input.parse::<Visibility>()?;
        let ty = input.parse::<Type>()?;
        Self::make(None, ty, &attrs)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn make(name: Option<Ident>, ty: Type, attrs: &[Attribute]) -> Result<Self> {
        let (field_attrs, mut help) = parse_bpaf_doc_attrs::<FieldAttrs>(attrs)?;

        let mut field_attrs = field_attrs.unwrap_or_default();

        if field_attrs.ignore_rustdoc {
            help = None;
        }

        let derived_consumer = field_attrs.consumer.is_empty();

        let mut cons = match field_attrs.consumer.pop() {
            Some(cons) => cons,
            None => derive_consumer(name.is_some() || !field_attrs.naming.is_empty(), &ty)?,
        };

        if let Consumer::External { span, ident: None } = &cons {
            let span = *span;
            match name.as_ref() {
                Some(n) => {
                    let ident = Ident::new(&to_snake_case(&n.to_string()), n.span());
                    cons = Consumer::External {
                        span,
                        ident: Some(ident.into()),
                    };
                }
                None => {
                    return Err(Error::new(
                        span,
                        "Can't derive name for this external, try specifying one",
                    ))
                }
            }
        }

        let mut env = Vec::new();
        let mut naming = Vec::new();
        for attr in field_attrs.naming {
            if let Name::Env { name, .. } = attr {
                env.push(StrictName::Env { name });
            } else {
                naming.push(StrictName::from_name(attr, &name)?);
            }
        }

        match (cons.needs_name(), !naming.is_empty()) {
            // need name, there are some. just need to resolve optional shorts and longs later
            (true, true) |
            // doesn't need a name, none are given. all good
            (false, false) => {}

            // needs a name, none specified, derive it from `name` field
            (true, false) => match &name {
                Some(n) => {
                    let span = n.span();
                    if n.to_string().chars().count() == 1 {
                        let short = Name::Short { name: None, span };
                        naming.push(StrictName::from_name(short, &name)?);
                    } else {
                        let long = Name::Long { name: None, span };
                        naming.push(StrictName::from_name(long, &name)?);
                    }
                }
                None => {
                    return Err(Error::new(
                        cons.span(),
                        "This consumer needs a name, you can specify it with long(\"name\") or short('n')",
                    ));
                }
            },

            // doesn't need a name, got some, must complain
            (false, true) => {
                return Err(Error::new_spanned(
                    ty,
                    "field doesn't take a name annotation",
                ));
            }

        };

        let mut postpr = std::mem::take(&mut field_attrs.postpr);

        let shape = split_type(&ty);

        if let Consumer::Argument { ty, .. }
        | Consumer::Positional { ty, .. }
        | Consumer::Any { ty, .. } = &mut cons
        {
            if ty.is_none() {
                match &shape {
                    Shape::Optional(t) | Shape::Multiple(t) | Shape::Direct(t) => {
                        *ty = Some(t.clone());
                    }
                    _ => {}
                }
            }
        }

        if derived_consumer {
            for pp in &postpr {
                if !pp.can_derive() {
                    let err = Error::new(
                        pp.span(),
                        "Can't derive implicit consumer with this annotation present",
                    );
                    return Err(err);
                }
            }
        }
        let span = ty.span();

        if !(postpr.iter().any(|p| matches!(p, Post::Parse(_)))
            || matches!(cons, Consumer::External { .. }))
        {
            match shape {
                Shape::Optional(_) => postpr.insert(0, Post::Parse(PostParse::Optional { span })),
                Shape::Multiple(_) => postpr.insert(0, Post::Parse(PostParse::Many { span })),
                Shape::Bool => {
                    if name.is_none()
                        && naming.is_empty()
                        && matches!(cons, Consumer::Switch { .. })
                    {
                        let msg = "Can't derive consumer for unnamed boolean field, try adding one of #[bpaf(positional)], #[bpaf(long(\"name\")] or #[bpaf(short('n'))] annotations to it";
                        let err = Error::new_spanned(ty, msg);
                        return Err(err);
                    }
                }
                Shape::Unit | Shape::Direct(_) => {}
            }
        }

        let help = match field_attrs.help.pop() {
            Some(h) => Some(Help::Custom(h.doc)),
            None => help,
        };

        Ok(StructField {
            name,
            env,
            naming,
            cons,
            postpr,
            help,
        })
    }
}
