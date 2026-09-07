//! Type-changing postprocessing annotations accumulated on top of a parser

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::ParseStream;

use crate::{
    decls::{POSTPR_DECL, Post},
    lookup_op,
};

/// Effect of a type-changing postprocessing operation on the output type
enum PostType {
    /// Doesn't change the output type
    Keep,
    /// Changes the output type to a known one, given as a turbofish
    Known(TokenStream),
    /// Changes the output type to one that has to be inferred. The tokens are
    /// used to point at the offending operation when a known type is required.
    Unknown(TokenStream),
}

fn untyped_postpr_error(ts: &TokenStream) -> syn::Error {
    let msg = "All transforming operations must specify the output type \
                       with turbo fish, try adding `::<Type>` before parens";
    syn::Error::new_spanned(ts, msg)
}

fn postpr_w_type(input: ParseStream, to_body: &mut TokenStream) -> syn::Result<Option<PostType>> {
    if let Some(pp) = lookup_op(POSTPR_DECL, input)? {
        let post = match pp {
            Post::KeepType(ts) => {
                to_body.extend(ts);
                PostType::Keep
            }
            Post::ChangeTypeKnown { ts, ty } => {
                to_body.extend(ts);
                PostType::Known(quote!(#ty))
            }
            Post::ChangeTypeUnknown(ts) => {
                to_body.extend(ts.clone());
                PostType::Unknown(ts)
            }
        };
        Ok(Some(post))
    } else {
        Ok(None)
    }
}

/// Accumulates type-changing postprocessing annotations together with the
/// output type they produce.
pub(crate) struct Postpr {
    /// Chained methods (`.map(f)`, `.fallback(1)`, …) in annotation order
    body: TokenStream,
    /// Current output type
    oty: TokenStream,
    /// First type changing operation without an explicit type; when set the
    /// parser can only be rendered as an expression
    untyped: Option<TokenStream>,
}

impl Postpr {
    pub(crate) fn new(ty: &Ident) -> Self {
        Self {
            body: TokenStream::new(),
            oty: quote!(#ty),
            untyped: None,
        }
    }

    /// Consume a postprocessing annotation, `false` if the input doesn't start
    /// with one. An untyped type change is recorded and reported by
    /// [`Self::oty`].
    pub(crate) fn lookup(&mut self, input: ParseStream) -> syn::Result<bool> {
        let Some(post) = postpr_w_type(input, &mut self.body)? else {
            return Ok(false);
        };
        match post {
            PostType::Keep => {}
            PostType::Known(t) => self.oty = t,
            PostType::Unknown(ts) => {
                self.untyped.get_or_insert(ts);
            }
        }
        Ok(true)
    }

    /// Chained postprocessing methods
    pub(crate) fn body(&self) -> &TokenStream {
        &self.body
    }

    /// Output type after postprocessing, errors out when a type changing
    /// operation didn't specify it with a turbofish
    pub(crate) fn oty(&self) -> syn::Result<&TokenStream> {
        if let Some(ts) = &self.untyped {
            return Err(untyped_postpr_error(ts));
        }
        Ok(&self.oty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::parse::Parser;

    fn postpr(ty: &str, tokens: TokenStream) -> syn::Result<Postpr> {
        let mut post = Postpr::new(&Ident::new(ty, Span::call_site()));
        (|input: ParseStream| {
            while post.lookup(input)? {}
            Ok(TokenStream::new())
        })
        .parse2(tokens)?;
        Ok(post)
    }

    #[test]
    fn untyped_type_change_errors_on_oty() {
        let post = postpr("Foo", quote!(map(f))).unwrap();
        assert!(post.oty().is_err());
        assert_eq!(post.body().to_string(), ". map (f)");
    }

    #[test]
    fn typed_type_change_updates_oty() {
        let post = postpr("Foo", quote!(map::<bool>(f))).unwrap();
        assert_eq!(post.oty().unwrap().to_string(), "bool");
    }

    #[test]
    fn keep_type_leaves_oty_alone() {
        let post = postpr("Foo", quote!(fallback(1))).unwrap();
        assert_eq!(post.oty().unwrap().to_string(), "Foo");
    }
}
