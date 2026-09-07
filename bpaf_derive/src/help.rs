//! Shared help and doc comment helpers.

use crate::decls::{GROUP_HELP_DECL, HELP_DECL, HELP3, Help3Op, IGNORE_RUSTDOC};
use crate::lookup_op;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// Doc comment split the same way the old parser does: first chunk is descr,
/// a non-empty second chunk is header, the rest is footer.
#[derive(Default)]
struct SplitHelp {
    pub(crate) descr: Option<String>,
    pub(crate) header: Option<String>,
    pub(crate) footer: Option<String>,
}

fn split_help(doc: &[String]) -> SplitHelp {
    if doc.is_empty() {
        SplitHelp::default()
    } else {
        let joined = doc.join("\n");
        let mut chunks = crate::utils::LineIter::from(joined.as_str());
        SplitHelp {
            descr: chunks.next(),
            header: chunks.next().filter(|s| !s.is_empty()),
            footer: chunks.rest(),
        }
    }
}

impl Help3 {
    pub(crate) fn new(help: Vec<String>) -> Self {
        Self {
            descr: None,
            header: None,
            footer: None,
            help,
        }
    }
    pub(crate) fn lookup(&mut self, input: syn::parse::ParseStream) -> syn::Result<bool> {
        if let Some(op) = lookup_op(HELP3, input)? {
            match op {
                Help3Op::Descr(ts) => self.descr = Some(ts),
                Help3Op::Header(ts) => self.header = Some(ts),
                Help3Op::Footer(ts) => self.footer = Some(ts),
            }
        } else if lookup_op(&[IGNORE_RUSTDOC], input)?.is_some() {
            self.help.clear();
        } else {
            return Ok(false);
        }
        Ok(true)
    }
}

pub(crate) struct Help3 {
    descr: Option<TokenStream>,
    header: Option<TokenStream>,
    footer: Option<TokenStream>,
    help: Vec<String>,
}

impl ToTokens for Help3 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SplitHelp {
            descr,
            header,
            footer,
        } = split_help(&self.help);

        tokens.extend(
            self.descr
                .clone()
                .or_else(|| descr.map(|descr| quote!(.descr(#descr)))),
        );

        tokens.extend(
            self.header
                .clone()
                .or_else(|| header.map(|header| quote!(.header(#header)))),
        );

        tokens.extend(
            self.footer
                .clone()
                .or_else(|| footer.map(|footer| quote!(.footer(#footer)))),
        );
    }
}

#[derive(Copy, Clone)]
pub(crate) enum HelpKind {
    /// `.help(..)`, the explicit override is `help(..)`
    Help,
    /// `.group_help(..)`, the explicit override is `group_help(..)`
    Group,
}

/// Single help slot for parsers that are not `OptionParser`s: same
/// lookup/done flow as [`Help3`], but rendered as a single method call.
pub(crate) struct Help {
    kind: HelpKind,
    custom: Option<TokenStream>,
    doc: Vec<String>,
}

impl Help {
    #[allow(clippy::self_named_constructors)] // there's two flavors of constructor
    pub(crate) fn help(doc: Vec<String>) -> Self {
        Self {
            kind: HelpKind::Help,
            custom: None,
            doc,
        }
    }

    pub(crate) fn group(doc: Vec<String>) -> Self {
        Self {
            kind: HelpKind::Group,
            custom: None,
            doc,
        }
    }

    pub(crate) fn lookup(&mut self, input: syn::parse::ParseStream) -> syn::Result<bool> {
        let decls = match self.kind {
            HelpKind::Help => HELP_DECL,
            HelpKind::Group => GROUP_HELP_DECL,
        };
        if let Some(ts) = lookup_op(decls, input)? {
            self.custom = Some(ts);
        } else if lookup_op(&[IGNORE_RUSTDOC], input)?.is_some() {
            self.doc.clear();
        } else {
            return Ok(false);
        }
        Ok(true)
    }
}

impl ToTokens for Help {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let j;
        let ts = if let Some(custom) = &self.custom {
            custom
        } else if !self.doc.is_empty() {
            let joined = self.doc.join("\n");
            j = quote!(#joined);
            &j
        } else {
            return;
        };

        match self.kind {
            HelpKind::Help => tokens.extend(quote!(.help(#ts))),
            HelpKind::Group => tokens.extend(quote!(.group_help(#ts))),
        }
    }
}
