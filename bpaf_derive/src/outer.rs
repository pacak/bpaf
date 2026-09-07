//! Parsers for outer attributes on top of structs, enums and field variants

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, LitStr, Visibility, parse::ParseStream};

use crate::{
    Generate, Help, Help3, NAME_DECL, Name, OPTION_PARSER, body,
    decls::{COMMAND_DECL, OPTION_PARSER_TOP, OUTER_KIND, OuterKind},
    ident_to_long, lookup_op,
    postpr::Postpr,
    push, resolve_name, split_attrs,
    utils::snake_case_ident,
};

/// Handler for top level `#[bpaf(...)]`
pub(crate) fn parse_outer(input: ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse::<Visibility>()?;

    let (doc, bpaf) = split_attrs(attrs)?;

    let (oty, mut body) = body::parse_inner(input)?;
    let generate = Generate {
        vis: Some(vis),
        name: snake_case_ident(&oty),
    };
    use syn::parse::Parser;
    (|stream: ParseStream| match lookup_op(OUTER_KIND, stream)? {
        Some(OuterKind::Options { cargo_helper }) => {
            if let Some(ch) = cargo_helper {
                body = quote::quote!(::bpaf::cargo_helper(#ch, #body));
            };
            parse_option_parser(doc, generate, &oty, body, stream)
        }
        Some(OuterKind::Command(cmd)) => {
            parse_command(doc, generate, cmd, &oty, body, stream)?.render_fn()
        }
        Some(OuterKind::Nest) => {
            let trigger = Trigger::Named;
            parse_nested(doc, generate, &oty, trigger, body, stream)?.render_fn()
        }
        Some(OuterKind::Literal(name)) => {
            let trigger = Trigger::Literal(name);
            parse_nested(doc, generate, &oty, trigger, body, stream)?.render_fn()
        }
        Some(OuterKind::Construct) | None => {
            parse_construct(doc, generate, oty, body, stream)?.render_fn()
        }
    })
    .parse2(bpaf)
}

/// Render a mode's expression as a generated function, erroring out when the
/// output type could not be determined.
fn render_fn(generate: &Generate, post: &Postpr, expr: TokenStream) -> syn::Result<TokenStream> {
    let oty = post.oty()?;
    Ok(quote! {
        #[doc(hidden)]
        #generate -> impl ::bpaf::Parser<Output = #oty> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            #expr
        }
    })
}

/// Handler for `#[bpaf(options[("cargo")], ...)]:`
fn parse_option_parser(
    doc: Vec<String>,
    mut generate: Generate,
    ty: &Ident,
    body: proc_macro2::TokenStream,
    input: ParseStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut help = Help3::new(doc);
    let mut post = Postpr::new(ty);
    let mut to_opts = TokenStream::new();
    loop {
        if generate.lookup(input)? || help.lookup(input)? || post.lookup(input)? {
        } else if let Some(ts) = lookup_op(OPTION_PARSER, input)? {
            to_opts.extend(ts);
        } else if let Some(ts) = lookup_op(OPTION_PARSER_TOP, input)? {
            to_opts.extend(ts);
        } else {
            break;
        }
    }
    let postpr = post.body();
    let oty = post.oty()?;
    Ok(quote::quote! {
        #[doc(hidden)]
        #generate -> ::bpaf::OptionParser<#oty> {
            #[allow(unused_imports)]
            use ::bpaf::Parser;
            #body #postpr .to_options() #help #to_opts
        }
    })
}

/// Handler for `#[bpaf(command[("name")], ...)]`
pub(crate) fn parse_command(
    doc: Vec<String>,
    mut generate: Generate,
    cmd: Option<LitStr>,
    ty: &Ident,
    body: proc_macro2::TokenStream,
    input: ParseStream,
) -> syn::Result<Command> {
    let mut help = Help3::new(doc);
    let mut post = Postpr::new(ty);
    let mut to_opts = TokenStream::new();
    let cmd = cmd.unwrap_or_else(|| ident_to_long(ty));
    let mut command = quote::quote!(.command(#cmd));

    loop {
        let span = input.span();
        if generate.lookup(input)? || help.lookup(input)? || post.lookup(input)? {
            continue;
        } else if let Some(ts) = lookup_op(OPTION_PARSER, input)? {
            to_opts.extend(ts);
        } else if let Some(name) = lookup_op(NAME_DECL, input)? {
            let alias = resolve_name(&name, Some(ty), span)?;
            command.extend(quote!(.#alias));
        } else if let Some(ts) = lookup_op(COMMAND_DECL, input)? {
            command.extend(ts);
        } else {
            break;
        }
    }
    Ok(Command {
        generate,
        body,
        help,
        to_opts,
        command,
        post,
    })
}

pub(crate) struct Command {
    generate: Generate,
    body: TokenStream,
    help: Help3,
    to_opts: TokenStream,
    command: TokenStream,
    post: Postpr,
}

impl Command {
    /// Render as an expression, the output type can stay unknown
    pub(crate) fn expr(&self) -> TokenStream {
        let Command {
            body,
            help,
            to_opts,
            command,
            post,
            ..
        } = self;
        let postpr = post.body();
        quote!(#body .to_options() #help #to_opts #command #postpr)
    }

    /// Render as a function, the output type has to be known
    pub(crate) fn render_fn(&self) -> syn::Result<TokenStream> {
        render_fn(&self.generate, &self.post, self.expr())
    }
}

/// How the first name of a nested parser is produced
pub(crate) enum Trigger {
    /// `short`/`long` from the annotation, defaulting to a long name derived
    /// from the type
    Named,
    /// `literal(name)`, defaulting to a name derived from the type, followed
    /// by optional `short`/`long` aliases
    Literal(Option<LitStr>),
}

/// Handler for `#[bpaf(nest, ...)]` and `#[bpaf(literal[("name")], ...)]`
pub(crate) fn parse_nested(
    doc: Vec<String>,
    mut generate: Generate,
    ty: &Ident,
    trigger: Trigger,
    body: proc_macro2::TokenStream,
    input: ParseStream,
) -> syn::Result<Nested> {
    let mut names = TokenStream::new();
    let mut help = Help::help(doc);
    let mut post = Postpr::new(ty);

    if let Trigger::Literal(name) = trigger {
        let name = name.unwrap_or_else(|| ident_to_long(ty));
        push(&mut names, &quote!(literal(#name)));
    }

    loop {
        if generate.lookup(input)? || help.lookup(input)? || post.lookup(input)? {
            continue;
        } else if let Some(name) = lookup_op(NAME_DECL, input)? {
            push(&mut names, &resolve_name(&name, Some(ty), ty.span())?);
        } else {
            break;
        }
    }
    if names.is_empty() {
        push(
            &mut names,
            &resolve_name(&Name::LongDerive, Some(ty), ty.span())?,
        );
    }
    Ok(Nested {
        generate,
        names,
        help,
        body,
        post,
    })
}

pub(crate) struct Nested {
    generate: Generate,
    names: TokenStream,
    help: Help,
    body: TokenStream,
    post: Postpr,
}

impl Nested {
    /// Render as an expression, the output type can stay unknown
    pub(crate) fn expr(&self) -> TokenStream {
        let Nested {
            names,
            help,
            body,
            post,
            ..
        } = self;
        let postpr = post.body();
        quote!(#names #help .nest(#body) #postpr)
    }

    /// Render as a function, the output type has to be known
    pub(crate) fn render_fn(&self) -> syn::Result<TokenStream> {
        render_fn(&self.generate, &self.post, self.expr())
    }
}

/// Handler for either `#[bpaf(construct, ...)]` or no leading keyword (implicit
/// construct)
pub(crate) fn parse_construct(
    doc: Vec<String>,
    mut generate: Generate,
    ty: Ident,
    body: proc_macro2::TokenStream,
    input: ParseStream,
) -> syn::Result<Construct> {
    let mut group = Help::group(doc);
    let mut post = Postpr::new(&ty);
    while generate.lookup(input)? || group.lookup(input)? || post.lookup(input)? {}

    Ok(Construct {
        generate,
        body,
        group,
        post,
    })
}

pub(crate) struct Construct {
    generate: Generate,
    body: TokenStream,
    group: Help,
    post: Postpr,
}

impl Construct {
    /// Render as an expression, the output type can stay unknown
    pub(crate) fn expr(&self) -> TokenStream {
        let Construct {
            body, group, post, ..
        } = self;
        let postpr = post.body();
        quote!(#body #group #postpr)
    }

    /// Render as a function, the output type has to be known
    pub(crate) fn render_fn(&self) -> syn::Result<TokenStream> {
        render_fn(&self.generate, &self.post, self.expr())
    }
}
