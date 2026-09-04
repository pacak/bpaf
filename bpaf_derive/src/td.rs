use crate::{
    attrs::PostDecor,
    help::Help,
    top::{ident_to_long, ident_to_short},
    utils::{parse_arg, parse_opt_arg},
};
use quote::{ToTokens, quote};
use syn::{
    Error, Expr, Ident, LitChar, LitStr, Result,
    parse::{Parse, ParseStream},
    parse_quote, token,
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
    pub(crate) max_width: Option<Box<Expr>>,
    pub(crate) fallback_usage: bool,
    pub(crate) help_parser: Option<Box<Expr>>,
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

/// Raw parsed nest config before name resolution (names may be None for auto-derive)
#[derive(Debug, Default)]
pub(crate) struct NestCfgRaw {
    pub(crate) short: Vec<Option<LitChar>>,
    pub(crate) long: Vec<Option<LitStr>>,
    pub(crate) help: Option<Help>,
}

impl NestCfgRaw {
    pub(crate) fn into_cfg(self, name: &Ident, help: &mut Option<Help>) -> NestCfg {
        let mut cfg = NestCfg::default();
        for s in self.short {
            cfg.short.push(s.unwrap_or_else(|| ident_to_short(name)));
        }
        for l in self.long {
            cfg.long.push(l.unwrap_or_else(|| ident_to_long(name)));
        }
        if cfg.short.is_empty() && cfg.long.is_empty() {
            cfg.long.push(ident_to_long(name));
        }
        cfg.help = self.help.or_else(|| help.take());

        cfg
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

    /// Nest configuration for wrapping the parser (raw, before name resolution)
    pub(crate) nest: Option<NestCfgRaw>,
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
            nest: None,
        }
    }
}

const TOP_NEED_OPTIONS: &str =
    "You need to add `options` annotation at the beginning to use this one";

const TOP_NEED_NEST_OR_COMMAND: &str = "Use either `command` or `nest` before this attribute";

const TOP_NEED_PARSER: &str = "This annotation can't be used with either `options` or `command`";

const NEST_MUST_BE_FIRST: &str = "This annotation must be first: try `#[bpaf(nest, ...`";

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
        let mut nest: Option<NestCfgRaw> = None;
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
            } else if kw == "fallback_to_usage" {
                if let Some(opts) = options.as_mut() {
                    opts.fallback_usage = true;
                } else {
                    return Err(Error::new_spanned(
                        kw,
                        "This annotation only makes sense in combination with `options` or `command`",
                    ));
                }
            } else if kw == "short" {
                if let Some(ref mut cfg) = nest {
                    cfg.short.push(parse_opt_arg(input)?);
                } else if let Some(cfg) = command.as_mut() {
                    cfg.short.push(parse_arg(input)?);
                } else {
                    return Err(Error::new_spanned(kw, TOP_NEED_NEST_OR_COMMAND));
                }
            } else if kw == "long" {
                if let Some(ref mut cfg) = nest {
                    cfg.long.push(parse_opt_arg(input)?);
                } else if let Some(cfg) = command.as_mut() {
                    cfg.long.push(parse_arg(input)?);
                } else {
                    return Err(Error::new_spanned(kw, TOP_NEED_NEST_OR_COMMAND));
                }
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
                if let Some(ref mut cfg) = nest {
                    cfg.help = Some(parse_arg(input)?);
                } else if let Some(cfg) = command.as_mut() {
                    cfg.help = Some(parse_arg(input)?);
                } else {
                    return Err(Error::new_spanned(kw, TOP_NEED_NEST_OR_COMMAND));
                }
            } else if first && kw == "nest" {
                nest = Some(NestCfgRaw::default());
            } else if kw == "nest" {
                return Err(Error::new_spanned(kw, NEST_MUST_BE_FIRST));
            } else if kw == "path" {
                bpaf_path.replace(parse_arg::<syn::Path>(input)?);
            } else if kw == "max_width" {
                let max_width = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |opt| opt.max_width = Some(max_width))?;
            } else if kw == "help_parser" {
                if command.is_some() {
                    return Err(Error::new_spanned(
                        kw,
                        "\"help_parser\" is only compatible with `options`",
                    ));
                }
                let help_parser = parse_arg(input)?;
                with_options(&kw, options.as_mut(), |opt| {
                    opt.help_parser = Some(help_parser)
                })?;
            } else if kw == "or_else" {
                if command.is_none() && parser.is_none() {
                    return Err(Error::new_spanned(
                        kw,
                        "This annotation is not compatible with `options`",
                    ));
                }
                let pd = PostDecor::parse(input, &kw)?.expect("or_else parses");
                attrs.push(pd);
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
            nest,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Ed {
    pub(crate) skip: bool,
    pub(crate) attrs: Vec<EAttr>,
    pub(crate) nest: Option<NestCfgRaw>,
}

pub(crate) enum VariantMode {
    Command,
    Parser,
    Nest,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NestCfg {
    pub(crate) short: Vec<LitChar>,
    pub(crate) long: Vec<LitStr>,
    pub(crate) help: Option<Help>,
}

impl Parse for Ed {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Vec::new();
        let mut skip = false;
        let mut nest = None;

        let mode = {
            let first = input.fork().parse::<Ident>()?;
            if first == "command" {
                VariantMode::Command
            } else if first == "nest" {
                VariantMode::Nest
            } else {
                VariantMode::Parser
            }
        };

        loop {
            let kw = input.parse::<Ident>()?;

            if kw == "command" {
                if matches!(mode, VariantMode::Nest) {
                    return Err(Error::new_spanned(
                        kw,
                        "\"command\" can't be used together with \"nest\"",
                    ));
                }
                attrs.push(if let Some(name) = parse_opt_arg(input)? {
                    EAttr::NamedCommand(name)
                } else {
                    EAttr::UnnamedCommand
                });
            } else if kw == "nest" {
                if matches!(mode, VariantMode::Command) {
                    return Err(Error::new_spanned(
                        kw,
                        "\"nest\" can't be used together with \"command\"",
                    ));
                }
                nest = Some(NestCfgRaw::default());
            } else if kw == "short" {
                if matches!(mode, VariantMode::Command) {
                    attrs.push(EAttr::CommandShort(parse_arg(input)?));
                } else if matches!(mode, VariantMode::Nest) {
                    if let Some(ref mut cfg) = nest {
                        cfg.short.push(parse_opt_arg(input)?);
                    }
                } else {
                    attrs.push(EAttr::UnitShort(parse_opt_arg(input)?));
                }
            } else if kw == "hide" {
                attrs.push(EAttr::Hide);
            } else if kw == "long" {
                if matches!(mode, VariantMode::Command) {
                    attrs.push(EAttr::CommandLong(parse_arg(input)?));
                } else if matches!(mode, VariantMode::Nest) {
                    if let Some(ref mut cfg) = nest {
                        cfg.long.push(parse_opt_arg(input)?);
                    }
                } else {
                    attrs.push(EAttr::UnitLong(parse_opt_arg(input)?));
                }
            } else if kw == "help" {
                if matches!(mode, VariantMode::Nest) {
                    if let Some(ref mut cfg) = nest {
                        cfg.help = Some(parse_arg(input)?);
                    }
                } else {
                    return Err(Error::new_spanned(
                        kw,
                        "\"help\" is not supported in this context",
                    ));
                }
            } else if kw == "fallback_to_usage" {
                if matches!(mode, VariantMode::Command) {
                    attrs.push(EAttr::FallbackUsage);
                } else {
                    return Err(Error::new_spanned(
                        kw,
                        "In this context this attribute requires \"command\" annotation",
                    ));
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

        Ok(Ed { skip, attrs, nest })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum EAttr {
    NamedCommand(LitStr),
    UnnamedCommand,

    FallbackUsage,
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
            Self::FallbackUsage => quote!(fallback_to_usage()),

            Self::UnnamedCommand | Self::UnitShort(_) | Self::UnitLong(_) => unreachable!(),
        }
        .to_tokens(tokens);
    }
}
