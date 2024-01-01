use crate::{
    attrs::PostDecor,
    help::Help,
    utils::{parse_arg, parse_name_value, parse_opt_arg},
};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, token, Error, Expr, Ident, LitChar, LitStr, Result,
};

// 1. options[("name")] and command[("name")] must be first in line and change parsing mode
//  some flags are valid in different modes but other than that order matters and structure does
//  not

#[derive(Debug, Default)]
pub(crate) struct CommandCfg {
    pub(crate) name: Option<LitStr>,
    pub(crate) long: Vec<LitStr>,
    pub(crate) short: Vec<LitChar>,
    pub(crate) help: Option<Help>,
}

#[derive(Debug, Default)]
pub(crate) struct OptionsCfg {
    pub(crate) cargo_helper: Option<LitStr>,
    pub(crate) descr: Option<Help>,
    pub(crate) footer: Option<Help>,
    pub(crate) header: Option<Help>,
    pub(crate) usage: Option<Box<Expr>>,
    pub(crate) version: Option<Box<Expr>>,
}

#[derive(Debug, Default)]
pub(crate) struct ParserCfg {
    pub(crate) group_help: Option<Help>,
}

#[derive(Debug)]
pub(crate) enum Mode {
    Command {
        command: CommandCfg,
        options: OptionsCfg,
    },
    Options {
        options: OptionsCfg,
    },
    Parser {
        parser: ParserCfg,
    },
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
    /// don't convert rustdoc to group_help, help, etc.
    pub(crate) ignore_rustdoc: bool,

    pub(crate) adjacent: bool,
    pub(crate) mode: Mode,
    pub(crate) attrs: Vec<PostDecor>,

    /// Custom absolute path to the `bpaf` crate.
    pub(crate) bpaf_path: Option<syn::Path>,
}

impl Default for TopInfo {
    fn default() -> Self {
        Self {
            private: false,
            custom_name: None,
            boxed: false,
            adjacent: false,
            mode: Mode::Parser {
                parser: Default::default(),
            },
            attrs: Vec::new(),
            ignore_rustdoc: false,
            bpaf_path: None,
        }
    }
}

const TOP_NEED_OPTIONS: &str =
    "You need to add `options` annotation at the beginning to use this one";

const TOP_NEED_COMMAND: &str =
    "You need to add `command` annotation at the beginning to use this one";

const TOP_NEED_PARSER: &str = "This annotation can't be used with either `options` or `command`";

fn with_options(
    kw: &Ident,
    cfg: Option<&mut OptionsCfg>,
    f: impl FnOnce(&mut OptionsCfg),
) -> Result<()> {
    match cfg {
        Some(cfg) => {
            f(cfg);
            Ok(())
        }
        None => Err(Error::new_spanned(kw, TOP_NEED_OPTIONS)),
    }
}

fn with_command(
    kw: &Ident,
    cfg: Option<&mut CommandCfg>,
    f: impl FnOnce(&mut CommandCfg),
) -> Result<()> {
    match cfg {
        Some(cfg) => {
            f(cfg);
            Ok(())
        }
        None => Err(Error::new_spanned(kw, TOP_NEED_COMMAND)),
    }
}

fn with_parser(
    kw: &Ident,
    cfg: Option<&mut ParserCfg>,
    f: impl FnOnce(&mut ParserCfg),
) -> Result<()> {
    match cfg {
        Some(cfg) => {
            f(cfg);
            Ok(())
        }
        None => Err(Error::new_spanned(kw, TOP_NEED_PARSER)),
    }
}

impl Parse for TopInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut private = false;
        let mut custom_name = None;
        let mut boxed = false;
        let mut ignore_rustdoc = false;
        let mut command = None;
        let mut options = None;
        let mut parser = Some(ParserCfg::default());
        let mut adjacent = false;
        let mut attrs = Vec::new();
        let mut first = true;
        let mut bpaf_path = None;
        loop {
            let kw = input.parse::<Ident>()?;

            if first && kw == "options" {
                let mut cfg = OptionsCfg::default();
                if let Some(helper) = parse_opt_arg(input)? {
                    cfg.cargo_helper = Some(helper);
                }
                options = Some(cfg);
                parser = None;
            } else if first && kw == "command" {
                let mut cfg = CommandCfg::default();
                if let Some(name) = parse_opt_arg(input)? {
                    cfg.name = Some(name);
                }
                options = Some(OptionsCfg::default());
                command = Some(cfg);
                parser = None;
            } else if kw == "private" {
                private = true;
            } else if kw == "generate" {
                custom_name = parse_arg(input)?;
            } else if kw == "options" {
                return Err(Error::new_spanned(
                    kw,
                    "This annotation must be first and used only once: try `#[bpaf(options, ...`",
                ));
            } else if kw == "command" {
                return Err(Error::new_spanned(
                    kw,
                    "This annotation must be first: try `#[bpaf(command, ...`",
                ));
            } else if kw == "version" {
                let version = parse_opt_arg(input)?
                    .unwrap_or_else(|| parse_quote!(env!("CARGO_PKG_VERSION")));
                with_options(&kw, options.as_mut(), |cfg| cfg.version = Some(version))?;
            } else if kw == "boxed" {
                boxed = true;
            } else if kw == "adjacent" {
                adjacent = true;
            } else if kw == "short" {
                let short = parse_arg(input)?;
                with_command(&kw, command.as_mut(), |cfg| cfg.short.push(short))?;
            } else if kw == "long" {
                let long = parse_arg(input)?;
                with_command(&kw, command.as_mut(), |cfg| cfg.long.push(long))?;
            } else if kw == "header" {
                let header = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |cfg| cfg.header = Some(header))?;
            } else if kw == "footer" {
                let footer = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |opt| opt.footer = Some(footer))?;
            } else if kw == "usage" {
                let usage = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |opt| opt.usage = Some(usage))?;
            } else if kw == "group_help" {
                let group_help = parse_arg(input)?;
                with_parser(&kw, parser.as_mut(), |opt| {
                    opt.group_help = Some(group_help)
                })?;
            } else if kw == "ignore_rustdoc" {
                ignore_rustdoc = true;
            } else if kw == "descr" {
                let descr = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |opt| opt.descr = Some(descr))?;
            } else if kw == "help" {
                let help = parse_arg(input)?;
                with_command(&kw, command.as_mut(), |cfg| cfg.help = Some(help))?;
            } else if kw == "bpaf_path" {
                bpaf_path.replace(parse_name_value::<syn::Path>(input)?);
            } else if let Some(pd) = PostDecor::parse(input, &kw)? {
                attrs.push(pd);
            } else {
                return Err(Error::new_spanned(
                    kw,
                    "Unexpected attribute for top level annotation",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<token::Comma>()?;
            if input.is_empty() {
                break;
            }
            first = false;
        }

        let mode = match (options, command) {
            (Some(options), Some(command)) => Mode::Command { command, options },
            (Some(options), None) => Mode::Options { options },
            _ => Mode::Parser {
                parser: parser.unwrap_or_default(),
            },
        };

        Ok(TopInfo {
            ignore_rustdoc,
            private,
            custom_name,
            boxed,
            adjacent,
            mode,
            attrs,
            bpaf_path,
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
            } else if kw == "header" {
                attrs.push(EAttr::Header(parse_arg(input)?));
            } else if kw == "footer" {
                attrs.push(EAttr::Footer(parse_arg(input)?));
            } else if kw == "env" {
                attrs.push(EAttr::Env(parse_arg(input)?));
            } else {
                return Err(Error::new_spanned(
                    kw,
                    "Unexpected attribute for enum variant annotation",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<token::Comma>()?;
            if input.is_empty() {
                break;
            }
        }

        Ok(Ed { skip, attrs })
    }
}

#[derive(Debug, Clone)]
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
    Header(Help),
    Footer(Help),
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
            Self::Header(d) => quote!(header(#d)),
            Self::Footer(d) => quote!(footer(#d)),
            Self::Usage(u) => quote!(usage(#u)),
            Self::Env(e) => quote!(env(#e)),
            Self::Hide => quote!(hide()),
            Self::UnnamedCommand | Self::UnitShort(_) | Self::UnitLong(_) => unreachable!(),
        }
        .to_tokens(tokens);
    }
}
