use crate::{
    attrs::PostDecor,
    help::Help,
    utils::{parse_arg, parse_opt_arg},
};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, token, Error, Expr, Ident, LitChar, LitStr, Result,
};

// 1. options[("name")] and command[("name")] must be first in line and change parsing mode
//  some flags are valid in different modes but other than that order matters and structure does
//  not

#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    Command,
    Options,
    Parser,
}

#[derive(Debug)]
pub(crate) enum HelpMsg {
    Lit(String),
    Custom(Box<Expr>),
}

impl From<String> for HelpMsg {
    fn from(value: String) -> Self {
        Self::Lit(value)
    }
}

impl From<Box<Expr>> for HelpMsg {
    fn from(value: Box<Expr>) -> Self {
        Self::Custom(value)
    }
}

impl ToTokens for HelpMsg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            HelpMsg::Lit(l) => l.to_tokens(tokens),
            HelpMsg::Custom(l) => l.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TopInfo {
    /// Should visibility for generated function to be inherited?
    pub(crate) private: bool,
    /// Should parser be generated with a custom name?
    pub(crate) custom_name: Option<Ident>,
    /// add .boxed() at the end
    pub(crate) boxed: bool,

    pub(crate) mode: Mode,
    pub(crate) attrs: Vec<TopAttr>,
}

impl Default for TopInfo {
    fn default() -> Self {
        Self {
            private: false,
            custom_name: None,
            boxed: false,
            mode: Mode::Parser,
            attrs: Vec::new(),
        }
    }
}

const TOP_NEED_OPTIONS: &str =
    "You need to add `options` annotation at the beginning to use this one";

const TOP_NEED_COMMAND: &str =
    "You need to add `command` annotation at the beginning to use this one";

#[derive(Debug)]
pub(crate) enum TopAttr {
    CargoHelper(LitStr),      // <- parsing
    Version(Box<Expr>),       // <- top only
    Adjacent,                 // generic
    NamedCommand(LitStr),     // generic
    UnnamedCommand,           // <- parsing
    CommandShort(LitChar),    //
    CommandLong(LitStr),      // <- command
    CompleteStyle(Box<Expr>), // decor
    Usage(Box<Expr>),         // command or top
    ToOptions,                // options
    Descr(Help),              // options
    Header(Help),             // options
    Footer(Help),             // options
    PostDecor(PostDecor),
}

impl ToTokens for TopAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::ToOptions => quote!(to_options()),
            Self::CargoHelper(_) | Self::UnnamedCommand => unreachable!(),
            Self::Version(v) => quote!(version(#v)),
            Self::Adjacent => quote!(adjacent()),
            Self::NamedCommand(n) => quote!(command(#n)),
            Self::CommandShort(n) => quote!(short(#n)),
            Self::CommandLong(n) => quote!(long(#n)),
            Self::CompleteStyle(c) => quote!(complete_style(#c)),
            Self::Usage(u) => quote!(usage(#u)),
            Self::Descr(d) => quote!(descr(#d)),
            Self::Header(d) => quote!(header(#d)),
            Self::Footer(d) => quote!(footer(#d)),
            Self::PostDecor(pd) => return pd.to_tokens(tokens),
        }
        .to_tokens(tokens);
    }
}

fn options(kw: &Ident, mode: Mode) -> Result<()> {
    if matches!(mode, Mode::Options) {
        Ok(())
    } else {
        Err(Error::new_spanned(kw, TOP_NEED_OPTIONS))
    }
}

fn command(kw: &Ident, mode: Mode) -> Result<()> {
    if matches!(mode, Mode::Command) {
        Ok(())
    } else {
        Err(Error::new_spanned(kw, TOP_NEED_COMMAND))
    }
}

impl Parse for TopInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut private = false;
        let mut custom_name = None;
        let mut boxed = false;
        let mode = {
            let first = input.fork().parse::<Ident>()?;
            if first == "options" {
                Mode::Options
            } else if first == "command" {
                Mode::Command
            } else {
                Mode::Parser
            }
        };

        let mut attrs = Vec::new();

        loop {
            let kw = input.parse::<Ident>()?;
            if kw == "private" {
                private = true;
            } else if kw == "generate" {
                custom_name = parse_arg(input)?;
            } else if kw == "options" {
                if !matches!(mode, Mode::Options) {
                    return Err(Error::new_spanned(
                        kw,
                        "This annotation must be first: try `#[bpaf(options, ...`",
                    ));
                }
                if let Some(helper) = parse_opt_arg(input)? {
                    attrs.push(TopAttr::CargoHelper(helper));
                }
                attrs.push(TopAttr::ToOptions);
            } else if kw == "command" {
                if !matches!(mode, Mode::Command) {
                    return Err(Error::new_spanned(
                        kw,
                        "This annotation must be first: try `#[bpaf(command, ...`",
                    ));
                }
                attrs.push(if let Some(name) = parse_opt_arg(input)? {
                    TopAttr::NamedCommand(name)
                } else {
                    TopAttr::UnnamedCommand
                });
            } else if kw == "version" {
                options(&kw, mode)?;
                attrs.push(TopAttr::Version(
                    parse_opt_arg(input)?
                        .unwrap_or_else(|| parse_quote!(env!("CARGO_PKG_VERSION"))),
                ));
            } else if kw == "boxed" {
                boxed = true;
            } else if kw == "adjacent" {
                attrs.push(TopAttr::Adjacent);
            } else if kw == "short" {
                command(&kw, mode)?;
                attrs.push(TopAttr::CommandShort(parse_arg(input)?));
            } else if kw == "long" {
                command(&kw, mode)?;
                attrs.push(TopAttr::CommandLong(parse_arg(input)?));
            } else if kw == "complete_style" {
                attrs.push(TopAttr::CompleteStyle(parse_arg(input)?));
            } else if kw == "usage" {
                options(&kw, mode).or_else(|_| command(&kw, mode))?;
                attrs.push(TopAttr::Usage(parse_arg(input)?));
            } else if let Some(pd) = PostDecor::parse(input, &kw)? {
                attrs.push(TopAttr::PostDecor(pd));
            } else {
                return Err(Error::new_spanned(
                    kw,
                    "Unepected attribute for top level annotation",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<token::Comma>()?;
        }

        Ok(TopInfo {
            private,
            custom_name,
            boxed,
            mode,
            attrs,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Ed {
    pub(crate) skip: bool,
    pub(crate) attrs: Vec<EAttr>,
}

pub(crate) enum VariantMode {
    Command,
    Parser,
}

impl Parse for Ed {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Vec::new();
        let mut skip = false;

        let mode = {
            let first = input.fork().parse::<Ident>()?;
            if first == "command" {
                VariantMode::Command
            } else {
                VariantMode::Parser
            }
        };

        loop {
            let kw = input.parse::<Ident>()?;

            if kw == "command" {
                attrs.push(if let Some(name) = parse_opt_arg(input)? {
                    EAttr::NamedCommand(name)
                } else {
                    EAttr::UnnamedCommand
                });
            } else if kw == "short" {
                if matches!(mode, VariantMode::Command) {
                    attrs.push(EAttr::CommandShort(parse_arg(input)?));
                } else {
                    attrs.push(EAttr::UnitShort(parse_opt_arg(input)?));
                }
            } else if kw == "hide" {
                attrs.push(EAttr::Hide);
            } else if kw == "long" {
                if matches!(mode, VariantMode::Command) {
                    attrs.push(EAttr::CommandLong(parse_arg(input)?));
                } else {
                    attrs.push(EAttr::UnitLong(parse_opt_arg(input)?));
                }
            } else if kw == "skip" {
                skip = true;
            } else if kw == "adjacent" {
                attrs.push(EAttr::Adjacent);
            } else if kw == "usage" {
                attrs.push(EAttr::Usage(parse_arg(input)?));
            } else if kw == "env" {
                attrs.push(EAttr::Env(parse_arg(input)?));
            } else {
                return Err(Error::new_spanned(
                    kw,
                    "Unepected attribute for enum variant annotation",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<token::Comma>()?;
        }

        Ok(Ed { skip, attrs })
    }
}

#[derive(Debug)]
pub(crate) enum EAttr {
    NamedCommand(LitStr),
    UnnamedCommand,

    CommandShort(LitChar),
    CommandLong(LitStr),
    Adjacent,
    Hide,
    UnitShort(Option<LitChar>),
    UnitLong(Option<LitStr>),
    Descr(Help),
    Usage(Box<Expr>),
    Env(Box<Expr>),
    ToOptions,
}

impl ToTokens for EAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::ToOptions => quote!(to_options()),
            Self::NamedCommand(n) => quote!(command(#n)),
            Self::CommandShort(n) => quote!(short(#n)),
            Self::CommandLong(n) => quote!(long(#n)),
            Self::Adjacent => quote!(adjacent()),
            Self::Descr(d) => quote!(descr(#d)),
            Self::Usage(u) => quote!(usage(#u)),
            Self::Env(e) => quote!(env(#e)),
            Self::Hide => quote!(hide()),
            Self::UnnamedCommand | Self::UnitShort(_) | Self::UnitLong(_) => unreachable!(),
        }
        .to_tokens(tokens);
    }
}
