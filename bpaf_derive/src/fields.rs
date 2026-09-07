use proc_macro2::{Ident, Span, TokenStream};
use syn::{
    Attribute, Error, LitChar, LitStr, Type, Visibility,
    parse::{ParseStream, Parser},
    token,
};

use crate::{
    Decl, Help,
    decls::{
        ANY_MOD, ARG_MOD, ENV_DECL, EXTERNAL_DECL, FLAG_MOD, FREE_DECL, NAME_DECL, NAMED_DECL,
        POS_MOD, POSTPR_DECL, Post, REPETITION_DECL,
    },
    field::Shape,
    lookup_op, split_attrs,
    utils::to_kebab_case,
};

#[derive(Debug, Clone)]
pub(crate) enum Name {
    Short(LitChar),
    Long(LitStr),
    /// bare `short` - derive the name from the field ident
    ShortDerive,
    /// bare `long` - derive the name from the field ident
    LongDerive,
    /// derive the name size from the ident, short for single letter and long otherwise
    BestDerive,
}

/// Resolve a name immediately and return its dot-less tokens (e.g. `short('x')`).
pub(crate) fn resolve_name(
    name: &Name,
    field: Option<&Ident>,
    span: Span,
) -> syn::Result<TokenStream> {
    enum S {
        Short,
        Long,
        Best,
    }
    let s = match name {
        Name::Short(c) => return Ok(quote::quote!(short(#c))),
        Name::Long(s) => return Ok(quote::quote!(long(#s))),
        Name::ShortDerive => S::Short,
        Name::LongDerive => S::Long,
        Name::BestDerive => S::Best,
    };
    let Some(field) = field else {
        return Err(Error::new(
            span,
            "Can't derive an explicit name for unnamed struct, \
                try adding a name here like short('f') or long(\"name\")",
        ));
    };
    let kebabed = to_kebab_case(&field.to_string());

    match s {
        S::Short => {
            let f = kebabed.chars().next().expect("field with empty name?");
            Ok(quote::quote!(short(#f)))
        }
        S::Long => Ok(quote::quote!(long(#kebabed))),

        S::Best => {
            let mut c = kebabed.chars();
            let first = c.next().expect("field with empty name?");
            Ok(if c.next().is_some() {
                quote::quote!(long(#kebabed))
            } else {
                quote::quote!(short(#first))
            })
        }
    }
}

/// Append a dot-less fragment to a parser method chain: the first fragment is
/// prefixed with `::bpaf::`, every later one gets a leading dot.
pub(crate) fn push(chain: &mut TokenStream, frag: &TokenStream) {
    if chain.is_empty() {
        chain.extend(quote::quote!(::bpaf::#frag));
    } else {
        chain.extend(quote::quote!(.#frag));
    }
}

/// Consume a run of `short`/`long`/`env` prefixes, in any order. The boolean
/// reports whether an explicit name (`short`/`long`) was among them.
pub(crate) fn parse_names(
    field: Option<&Ident>,
    input: ParseStream,
) -> syn::Result<(TokenStream, bool)> {
    let mut body = TokenStream::new();
    let mut has_name = false;
    loop {
        let span = input.span();
        if let Some(name) = lookup_op(NAME_DECL, input)? {
            has_name = true;
            push(&mut body, &resolve_name(&name, field, span)?);
        } else if let Some(ts) = lookup_op(ENV_DECL, input)? {
            push(&mut body, &ts);
        } else {
            break;
        }
    }
    Ok((body, has_name))
}

/// Derive a name from `field` and append it unless `has_name` is set.
pub(crate) fn default_name(
    body: &mut TokenStream,
    has_name: bool,
    field: Option<&Ident>,
    span: Span,
) -> syn::Result<()> {
    if !has_name {
        push(body, &resolve_name(&Name::BestDerive, field, span)?);
    }
    Ok(())
}

fn preamble(input: ParseStream) -> syn::Result<(Vec<String>, TokenStream)> {
    let attrs = input.call(Attribute::parse_outer)?;
    let (doc, bpaf) = split_attrs(attrs)?;
    input.parse::<Visibility>()?;
    Ok((doc, bpaf))
}

pub(crate) fn parse_named(input: ParseStream) -> syn::Result<(Ident, TokenStream)> {
    let (doc, attr) = preamble(input)?;
    let name = input.parse::<Ident>()?;
    input.parse::<token::Colon>()?;
    let ty = input.parse::<Type>()?;
    input.parse::<Option<token::Comma>>()?;

    let ts = (|stream: ParseStream| parse_field(doc, Some(&name), &ty, stream)).parse2(attr)?;
    Ok((name, ts))
}

pub(crate) fn parse_unnamed(input: ParseStream) -> syn::Result<TokenStream> {
    let (doc, attr) = preamble(input)?;
    let ty = input.parse::<Type>()?;
    input.parse::<Option<token::Comma>>()?;
    let ts = (|stream: ParseStream| parse_field(doc, None, &ty, stream)).parse2(attr)?;
    Ok(ts)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ConsumerKind {
    Switch,
    Flag,
    ReqFlag,
    Argument,
    Positional,
    Any,
    External,
    Pure,
    PureWith,
}

impl ConsumerKind {
    /// Whether the derive attaches `.help(...)` to this consumer. `external`
    /// returns a parser whose shape we can't inspect; `pure`/`pure_with` are
    /// intentionally not shown in the help output.
    fn accepts_help(&self) -> bool {
        !matches!(
            self,
            ConsumerKind::External | ConsumerKind::Pure | ConsumerKind::PureWith
        )
    }

    /// Whether an `Option<_>`/`Vec<_>` field may be mapped to `.optional()`/
    /// `.many()`. `external` hides the parsed type, and repeating `pure`/
    /// `pure_with` is meaningless.
    fn accepts_implicit_repeat(&self) -> bool {
        !matches!(
            self,
            ConsumerKind::External | ConsumerKind::Pure | ConsumerKind::PureWith
        )
    }

    /// The modifier table valid right after this consumer
    fn modifiers(&self) -> &'static [Decl<TokenStream>] {
        match self {
            ConsumerKind::ReqFlag => FLAG_MOD,
            ConsumerKind::Argument => ARG_MOD,
            ConsumerKind::Positional => POS_MOD,
            ConsumerKind::Any => ANY_MOD,
            ConsumerKind::Switch
            | ConsumerKind::Flag
            | ConsumerKind::External
            | ConsumerKind::Pure
            | ConsumerKind::PureWith => &[],
        }
    }
}

/// The consumer implied by the field's type when none is given.
fn implied_consumer(sha: Shape) -> (ConsumerKind, TokenStream) {
    match sha {
        Shape::Bool => (ConsumerKind::Switch, quote::quote!(switch())),
        Shape::Unit => (ConsumerKind::ReqFlag, quote::quote!(req_flag(()))),
        Shape::Optional | Shape::Multiple | Shape::Plain => {
            (ConsumerKind::Argument, quote::quote!(argument("ARG")))
        }
    }
}

/// The consumer chosen for a field, together with the flags needed to decide
/// on modifiers and postprocessing.
struct SelectedConsumer {
    kind: ConsumerKind,
    /// The parser producer chain (`::bpaf::long("x").argument("ARG")`, `level()`, …).
    body: TokenStream,
    /// No consumer was written; it was derived from the field.
    implicit: bool,
}

/// Consume the consumer annotation: `short`/`long`/`env` names, an explicit
/// consumer, or one derived from the field name and type. Leaves the stream
/// positioned after the consumer.
fn resolve_consumer(
    sha: Shape,
    field: Option<&Ident>,
    input: ParseStream,
) -> syn::Result<SelectedConsumer> {
    let (default_kind, default_consumer) = implied_consumer(sha);
    let mut body = TokenStream::new();
    let mut has_name = false;
    let mut consumer = None;
    let mut kind = default_kind;
    let mut named = false;

    // first try to parse it as a named parser, can start with env/short/long,
    // can contain
    loop {
        let (ts, run_has_name) = parse_names(field, input)?;
        if !ts.is_empty() {
            named = true;
            has_name |= run_has_name;
            body.extend(ts);
        }
        let span = input.span();
        let Some((ckind, ts)) = lookup_op(NAMED_DECL, input)? else {
            break;
        };
        named = true;
        if consumer.is_some() {
            let msg = "Only one consumer per attribute is allowed!";
            return Err(Error::new(span, msg));
        }
        consumer = Some(ts);
        kind = ckind;
    }

    if named {
        default_name(&mut body, has_name, field, input.span())?;
        let implicit = consumer.is_none();
        push(&mut body, &consumer.unwrap_or(default_consumer));
        Ok(SelectedConsumer {
            kind,
            body,
            implicit,
        })
    } else if let Some((kind, stream)) = lookup_op(FREE_DECL, input)? {
        push(&mut body, &stream);
        Ok(SelectedConsumer {
            kind,
            body,
            implicit: false,
        })
    } else if let Some(ext) = lookup_op(EXTERNAL_DECL, input)? {
        let span = input.span();
        let body = match (ext, field) {
            (Some(ts), _) => ts,
            (None, Some(name)) => quote::quote!(#name()),
            (_, None) => {
                return Err(Error::new(
                    span,
                    "Can't derive name for this external, try specifying one",
                ));
            }
        };
        Ok(SelectedConsumer {
            kind: ConsumerKind::External,
            body,
            implicit: false,
        })
    } else {
        // no consumer annotation: derive one from the field name and its shape
        let span = input.span();
        let derived_name = resolve_name(&Name::BestDerive, field, span);
        let refuse = |what| {
            Error::new(
                span,
                format!(
                    "Refusing to derive a positional item for {what}, you can fix this \
                    by either adding a short/long name or making it positional explicitly"
                ),
            )
        };
        let body = match (derived_name, sha) {
            (Err(_), Shape::Bool) => return Err(refuse("bool")),
            (Err(_), Shape::Unit) => return Err(refuse("()")),
            (Err(_), _) => quote::quote!(::bpaf::positional("ARG")),
            (Ok(name), _) => quote::quote!(::bpaf::#name.#default_consumer),
        };
        Ok(SelectedConsumer {
            kind: default_kind,
            body,
            implicit: true,
        })
    }
}

/// Postprocessing after the consumer: repetitions and type-changing posts, plus
/// `ignore_rustdoc`/`help`, in the order written. Returns the postprocessing
/// tokens, the help tokens, and whether the implicit repetition still applies.
fn parse_postprocessing(
    kind: ConsumerKind,
    sha: Shape,
    doc: Vec<String>,
    input: ParseStream,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
    let mut postpr = TokenStream::new();
    let mut implicit_repeat =
        kind.accepts_implicit_repeat() && matches!(sha, Shape::Optional | Shape::Multiple);
    let mut help = Help::help(doc);

    while !input.is_empty() {
        let span = input.span();
        if let Some(ts) = lookup_op(REPETITION_DECL, input)? {
            postpr.extend(ts);
            implicit_repeat = false;
        } else if let Some(ts) = lookup_op(POSTPR_DECL, input)? {
            match ts {
                Post::KeepType(ts) => postpr.extend(ts),
                Post::ChangeTypeKnown { ts, ty: _ } | Post::ChangeTypeUnknown(ts) => {
                    implicit_repeat = false;
                    postpr.extend(ts);
                }
            }
        } else if !help.lookup(input)? {
            return Err(Error::new(span, "unexpected annotation"));
        }
    }

    let help = if kind.accepts_help() {
        quote::quote!(#help)
    } else {
        TokenStream::new()
    };

    Ok((postpr, help, implicit_repeat))
}

fn parse_field(
    doc: Vec<String>,
    field: Option<&Ident>,
    ty: &Type,
    input: ParseStream,
) -> syn::Result<TokenStream> {
    let sha = Shape::make(ty);
    let SelectedConsumer {
        kind,
        body,
        implicit,
    } = resolve_consumer(sha, field, input)?;

    // modifiers that belong to this consumer kind, in the order written
    let mut tail = TokenStream::new();
    if !implicit {
        while let Some(ts) = lookup_op(kind.modifiers(), input)? {
            tail.extend(ts);
        }
    }

    let (postpr, help, implicit_repeat) = parse_postprocessing(kind, sha, doc, input)?;
    if implicit_repeat {
        match sha {
            Shape::Optional => tail.extend(quote::quote!(.optional())),
            Shape::Multiple => tail.extend(quote::quote!(.many())),
            _ => {}
        }
    }

    Ok(quote::quote!( #body #help #tail #postpr))
}
