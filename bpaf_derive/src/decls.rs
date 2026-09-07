use crate::{
    Name,
    attr::{Arg, Decl},
    fields::ConsumerKind,
    lookup_op,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Type, Visibility};

/// # Methods on `OptionParser`
pub(crate) enum Help3Op {
    Descr(TokenStream),
    Header(TokenStream),
    Footer(TokenStream),
}

pub(crate) const HELP3: &[Decl<Help3Op>] = &[
    Decl {
        name: "descr",
        descr: "short description of the parser, shown at the top of the help \
            message, see [`OptionParser::descr`](crate::OptionParser::descr)",
        op: &[Arg::Ts(|ts| Help3Op::Descr(quote!(.descr(#ts))))],
    },
    Decl {
        name: "header",
        descr: "extra text shown between the usage line and the options, \
            see [`OptionParser::header`](crate::OptionParser::header)",
        op: &[Arg::Ts(|ts| Help3Op::Header(quote!(.header(#ts))))],
    },
    Decl {
        name: "footer",
        descr: "extra text shown at the end of the help message, \
            see [`OptionParser::footer`](crate::OptionParser::footer)",
        op: &[Arg::Ts(|ts| Help3Op::Footer(quote!(.footer(#ts))))],
    },
];

pub(crate) const HELP_DECL: &[Decl<TokenStream>] = &[Decl {
    name: "help",
    descr: "help message for the parser, \
        see [`Named::help`](crate::api::primitives::Named::help); for commands \
        it sets the command help, \
        see [`Command::help`](crate::api::composite::Command::help)",
    op: &[Arg::Ts(|ts| ts)],
}];

pub(crate) const GROUP_HELP_DECL: &[Decl<TokenStream>] = &[Decl {
    name: "group_help",
    descr: "help message for the group, \
        see [`Parser::group_help`](crate::Parser::group_help)",
    op: &[Arg::Ts(|ts| ts)],
}];

/// Postprocessing for contexts where the output type is fixed: consumes a
/// non-type-changing [`POSTPR_DECL`] operation, rejects the rest.
pub(crate) fn postpr_keep_type(
    input: syn::parse::ParseStream,
    to_body: &mut TokenStream,
) -> syn::Result<bool> {
    let Some(pp) = lookup_op(POSTPR_DECL, input)? else {
        return Ok(false);
    };
    match pp {
        Post::KeepType(ts) => {
            to_body.extend(ts);
            Ok(true)
        }
        Post::ChangeTypeKnown { ts, .. } | Post::ChangeTypeUnknown(ts) => {
            let msg = "this annotation can change the type, it is not supported here";
            Err(syn::Error::new_spanned(ts, msg))
        }
    }
}

pub(crate) const OPTION_PARSER: &[Decl<TokenStream>] = &[
    Decl {
        name: "usage",
        descr: "replace the auto generated usage line, \
            see [`OptionParser::usage`](crate::OptionParser::usage)",
        op: &[Arg::Ts(|ts| quote!(.usage(#ts)))],
    },
    Decl {
        name: "fallback_to_usage",
        descr: "use the usage line as the help message when the parser fails, \
            see [`OptionParser::fallback_to_usage`](crate::OptionParser::fallback_to_usage)",
        op: &[Arg::Bare(|| quote!(.fallback_to_usage()))],
    },
];

/// Flags that configure the generated function itself, valid in every mode.
pub(crate) enum TopShared {
    SetOutName(Ident),
    Private,
}

pub(crate) struct Generate {
    pub(crate) vis: Option<Visibility>,
    pub(crate) name: Ident,
}

impl quote::ToTokens for Generate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Generate { vis, name } = self;
        tokens.extend(quote::quote! {#vis fn #name()});
    }
}

impl Generate {
    pub(crate) fn apply(&mut self, op: TopShared) {
        match op {
            TopShared::SetOutName(ident) => self.name = ident,
            TopShared::Private => self.vis = None,
        }
    }

    /// Consume a [`SHARED_DECL`] annotation, if the input starts with one.
    pub(crate) fn lookup(&mut self, input: syn::parse::ParseStream) -> syn::Result<bool> {
        if let Some(op) = lookup_op(SHARED_DECL, input)? {
            self.apply(op);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// keywords valid in every mode, consulted first
pub(crate) const SHARED_DECL: &[Decl<TopShared>] = &[
    Decl {
        name: "generate",
        descr: "produce a function with this name instead of deriving it from \
            the `struct`/`enum` name",
        op: &[Arg::Ident(TopShared::SetOutName)],
    },
    Decl {
        name: "private",
        descr: "make the generated function private to the module",
        op: &[Arg::Bare(|| TopShared::Private)],
    },
];

pub(crate) const IGNORE_RUSTDOC: Decl<()> = Decl {
    name: "ignore_rustdoc",
    descr: "ignore the doc comment when generating the help message",
    op: &[Arg::Bare(|| ())],
};

/// Unlike [`OPTION_PARSER`] those make sense at the top level only
pub(crate) const OPTION_PARSER_TOP: &[Decl<TokenStream>] = &[
    Decl {
        name: "max_width",
        descr: "maximum width of the help message, \
            see [`OptionParser::max_width`](crate::OptionParser::max_width)",
        op: &[Arg::Ts(|ts| quote!(.max_width(#ts)))],
    },
    Decl {
        name: "help_parser",
        descr: "custom parser for the `--help` flag, \
            see [`OptionParser::help_parser`](crate::OptionParser::help_parser)",
        op: &[Arg::Ts(|ts| quote::quote!(.help_parser(#ts)))],
    },
    Decl {
        name: "colorscheme",
        descr: "set the color scheme used by the help message, \
            see [`OptionParser::colorscheme`](crate::OptionParser::colorscheme)",
        op: &[Arg::Ts(|ts| quote::quote!(.colorscheme(#ts)))],
    },
    Decl {
        name: "version",
        descr: "version string, defaults to env!(\"CARGO_PKG_VERSION\"), \
            see [`OptionParser::version`](crate::OptionParser::version)",
        op: &[
            Arg::Bare(|| quote!(.version(env!("CARGO_PKG_VERSION")))),
            Arg::Ts(|ts| quote!(.version(#ts))),
        ],
    },
];

pub(crate) const SKIP: Decl<()> = Decl {
    name: "skip",
    descr: "",
    op: &[Arg::Bare(|| ())],
};

pub(crate) const CMD: Decl<Option<LitStr>> = Decl {
    name: "command",
    descr: "make this variant a command, name can be derived from the variant",
    op: &[Arg::Bare(|| None), Arg::LitStr(Some)],
};

pub(crate) const NEST: Decl<()> = Decl {
    name: "nest",
    descr: "wrap this variant into a nest, names act as nest names",
    op: &[Arg::Bare(|| ())],
};

pub(crate) const LITERAL: Decl<Option<LitStr>> = Decl {
    name: "literal",
    descr: "a literal matcher, name can be derived from the variant",
    op: &[Arg::Bare(|| None), Arg::LitStr(Some)],
};

/// Name annotations, shared by fields and the alias positions
/// (`literal`/`nest`/`command`).
pub(crate) const NAME_DECL: &[Decl<Name>] = &[
    Decl {
        name: "short",
        descr: "add a short name, `short('c')` or derive it from the field or type \
            name, see [`short`](crate::short)",
        op: &[Arg::Bare(|| Name::ShortDerive), Arg::Char(Name::Short)],
    },
    Decl {
        name: "long",
        descr: "add a long name, `long(\"name\")` or derive it from the field or type \
            name, see [`long`](crate::long)",
        op: &[Arg::Bare(|| Name::LongDerive), Arg::LitStr(Name::Long)],
    },
];

/// A parser prefix rather than a name: reads the value from an environment
/// variable.
pub(crate) const ENV_DECL: &[Decl<TokenStream>] = &[Decl {
    name: "env",
    descr: "read the value from an environment variable, see [`env`](crate::env)",
    op: &[Arg::Ts(|ts| quote::quote!(env(#ts)))],
}];

/// Consumers that require a name on the parser: `argument` and the flag*.
/// The tuple is the consumer kind and its parser producer.
pub(crate) const NAMED_DECL: &[Decl<(ConsumerKind, TokenStream)>] = &[
    Decl {
        name: "argument",
        descr: "parse an option argument, type and metavar can be skipped, \
            see [`Named::argument`](crate::api::primitives::Named::argument)",
        op: &[
            Arg::Bare(|| (ConsumerKind::Argument, quote::quote!(argument("ARG")))),
            Arg::LitStr(|m| (ConsumerKind::Argument, quote::quote!(argument(#m)))),
            Arg::Fish(|t| (ConsumerKind::Argument, quote::quote!(argument::<#t>("ARG")))),
            Arg::FishLitStr(|t, m| (ConsumerKind::Argument, quote::quote!(argument::<#t>(#m)))),
        ],
    },
    Decl {
        name: "switch",
        descr: "a boolean flag, present or absent, \
            see [`Named::switch`](crate::api::primitives::Named::switch)",
        op: &[Arg::Bare(|| {
            (ConsumerKind::Switch, quote::quote!(switch()))
        })],
    },
    Decl {
        name: "flag",
        descr: "a flag with a value for present and absent states, \
            see [`Named::flag`](crate::api::primitives::Named::flag)",
        op: &[Arg::TsTs(|present, absent| {
            (ConsumerKind::Flag, quote::quote!(flag(#present, #absent)))
        })],
    },
    Decl {
        name: "req_flag",
        descr: "a flag that has to be present, \
            see [`Named::req_flag`](crate::api::primitives::Named::req_flag)",
        op: &[Arg::Ts(|present| {
            (ConsumerKind::ReqFlag, quote::quote!(req_flag(#present)))
        })],
    },
];

pub(crate) const FREE_DECL: &[Decl<(ConsumerKind, TokenStream)>] = &[
    Decl {
        name: "positional",
        descr: "parse an operand, type and metavar can be skipped, \
            see [`positional`](crate::positional)",
        op: &[
            Arg::Bare(|| (ConsumerKind::Positional, quote::quote!(positional("ARG")))),
            Arg::LitStr(|m| (ConsumerKind::Positional, quote::quote!(positional(#m)))),
            Arg::Fish(|t| {
                (
                    ConsumerKind::Positional,
                    quote::quote!(positional::<#t>("ARG")),
                )
            }),
            Arg::FishLitStr(|t, m| {
                (
                    ConsumerKind::Positional,
                    quote::quote!(positional::<#t>(#m)),
                )
            }),
        ],
    },
    Decl {
        name: "any",
        descr: "parse an arbitrary item with a custom check, \
            see [`any`](crate::any)",
        op: &[
            Arg::LitStrTs(|m, check| (ConsumerKind::Any, quote::quote!(any(#m, #check)))),
            Arg::Fish2LitStrTs(|a, b, m, check| {
                (ConsumerKind::Any, quote::quote!(any::<#a, #b>(#m, #check)))
            }),
        ],
    },
    Decl {
        name: "any_from_str",
        descr: "parse an arbitrary item with `FromStr`, \
            see [`any_from_str`](crate::any_from_str)",
        op: &[
            Arg::LitStr(|m| (ConsumerKind::Any, quote::quote!(any_from_str(#m)))),
            Arg::FishLitStr(|f, m| (ConsumerKind::Any, quote::quote!(any_from_str<#f>(#m)))),
        ],
    },
    Decl {
        name: "pure",
        descr: "produce a fixed value, see [`pure`](crate::pure)",
        op: &[Arg::Ts(|expr| {
            (ConsumerKind::Pure, quote::quote!(pure(#expr)))
        })],
    },
    Decl {
        name: "pure_with",
        descr: "produce a value by calling a closure, \
            see [`pure_with`](crate::pure_with)",
        op: &[Arg::Ts(|expr| {
            (ConsumerKind::PureWith, quote::quote!(pure_with(#expr)))
        })],
    },
];

pub(crate) const EXTERNAL_DECL: &[Decl<Option<TokenStream>>] = &[Decl {
    name: "external",
    descr: "use a separately defined parser from a function or derive from the field name",
    op: &[
        // empty token stream for external = must derive name
        Arg::Bare(|| None),
        Arg::Ts(|ts| Some(quote::quote!(#ts()))),
    ],
}];

const COMPLETE: Decl<TokenStream> = Decl {
    name: "complete",
    descr: "enable shell completion for this parser, \
        see [`Argument::complete`](crate::api::primitives::Argument::complete), \
        [`Positional::complete`](crate::api::primitives::Positional::complete) and \
        [`Anything::complete`](crate::api::primitives::Anything::complete)",
    op: &[Arg::Ts(|f| quote::quote!(.complete(#f)))],
};

pub(crate) const ARG_MOD: &[Decl<TokenStream>] = &[
    Decl {
        name: "adjacent",
        descr: "require the value to be adjacent to the argument name, \
            see [`Argument::adjacent`](crate::api::primitives::Argument::adjacent)",
        op: &[Arg::Bare(|| quote::quote!(.adjacent()))],
    },
    Decl {
        name: "negative_lit",
        descr: "accept negative numbers as argument values, \
            see [`Argument::negative_lit`](crate::api::primitives::Argument::negative_lit)",
        op: &[Arg::Bare(|| quote::quote!(.negative_lit()))],
    },
    Decl {
        name: "on_missing_value",
        descr: "value to use when the argument is present without a value, \
            see [`Argument::on_missing_value`](crate::api::primitives::Argument::on_missing_value)",
        op: &[Arg::Ts(|f| quote::quote!(.on_missing_value(#f)))],
    },
    COMPLETE,
];

pub(crate) const FLAG_MOD: &[Decl<TokenStream>] = &[Decl {
    name: "default",
    descr: "use the present value as the default when the flag is absent, \
        see [`Flag::default`](crate::api::primitives::Flag::default)",
    op: &[Arg::Bare(|| quote::quote!(.default()))],
}];

pub(crate) const POS_MOD: &[Decl<TokenStream>] = &[
    Decl {
        name: "strict",
        descr: "stop parsing at the first positional, \
            see [`Positional::strict`](crate::api::primitives::Positional::strict)",
        op: &[Arg::Bare(|| quote::quote!(.strict()))],
    },
    Decl {
        name: "non_strict",
        descr: "continue parsing after the first positional, \
            see [`Positional::non_strict`](crate::api::primitives::Positional::non_strict)",
        op: &[Arg::Bare(|| quote::quote!(.non_strict()))],
    },
    Decl {
        name: "posix",
        descr: "use POSIX style parsing for this positional, \
            see [`Positional::posix`](crate::api::primitives::Positional::posix)",
        op: &[Arg::Bare(|| quote::quote!(.posix()))],
    },
    COMPLETE,
];

pub(crate) const ANY_MOD: &[Decl<TokenStream>] = &[COMPLETE];

pub(crate) const REPETITION_DECL: &[Decl<TokenStream>] = &[
    Decl {
        name: "many",
        descr: "collect zero or more values, \
            see [`Parser::many`](crate::Parser::many)",
        op: &[Arg::Bare(|| quote::quote!(.many()))],
    },
    Decl {
        name: "optional",
        descr: "make the value optional, \
            see [`Parser::optional`](crate::Parser::optional)",
        op: &[Arg::Bare(|| quote::quote!(.optional()))],
    },
    Decl {
        name: "some",
        descr: "collect one or more values, \
            see [`Parser::some`](crate::Parser::some)",
        op: &[Arg::LitStr(|m| quote::quote!(.some(#m)))],
    },
    Decl {
        name: "count",
        descr: "count how many times the parser was present, \
            see [`Parser::count`](crate::Parser::count)",
        op: &[Arg::Bare(|| quote::quote!(.count()))],
    },
    Decl {
        name: "last",
        descr: "keep only the last value, \
            see [`Parser::last`](crate::Parser::last)",
        op: &[Arg::Bare(|| quote::quote!(.last()))],
    },
    Decl {
        name: "collect",
        descr: "collect values into a collection, \
            see [`Parser::collect`](crate::Parser::collect)",
        op: &[Arg::Bare(|| quote::quote!(.collect()))],
    },
];

#[allow(clippy::large_enum_variant)]
pub(crate) enum Post {
    /// Doesn't change the output type: `guard`, `fallback`, `hide`, `group_help`,
    /// `or_else` and friends.
    KeepType(TokenStream),
    /// Changes the output type to a known one, given as a turbofish
    /// (`map::<T>(f)`): `ty` is the new output type. A later type changing post
    /// overrides it.
    ChangeTypeKnown { ts: TokenStream, ty: Type },
    /// Changes the output type, the new type has to be inferred by the compiler
    /// (`map(f)`). Like [`Self::ChangeTypeKnown`] it disables the implicit
    /// repetition derived from the field shape.
    ChangeTypeUnknown(TokenStream),
}

pub(crate) const POSTPR_DECL: &[Decl<Post>] = &[
    Decl {
        name: "map",
        descr: "apply a function to the parsed value, \
            see [`Parser::map`](crate::Parser::map)",
        op: &[
            Arg::Ts(|f| Post::ChangeTypeUnknown(quote::quote!(.map(#f)))),
            Arg::FishTs(|ty, f| Post::ChangeTypeKnown {
                ts: quote::quote!(.map::<_, #ty>(#f)),
                ty,
            }),
        ],
    },
    Decl {
        name: "parse",
        descr: "parse the value with a custom function, \
            see [`Parser::parse`](crate::Parser::parse)",
        op: &[
            Arg::Ts(|f| Post::ChangeTypeUnknown(quote::quote!(.parse(#f)))),
            Arg::FishTs(|ty, f| Post::ChangeTypeKnown {
                ts: quote::quote!(.parse::<_, #ty, _>(#f)),
                ty,
            }),
        ],
    },
    Decl {
        name: "guard",
        descr: "check the parsed value, the message is shown when the check \
            fails, see [`Parser::guard`](crate::Parser::guard)",
        op: &[Arg::TsTs(|check, msg| {
            Post::KeepType(quote::quote!(.guard(#check, #msg)))
        })],
    },
    Decl {
        name: "fallback",
        descr: "use this value when the parser fails, \
            see [`Parser::fallback`](crate::Parser::fallback)",
        op: &[Arg::Ts(|v| Post::KeepType(quote::quote!(.fallback(#v))))],
    },
    Decl {
        name: "fallback_with",
        descr: "call a closure to get the value when the parser fails, \
            see [`Parser::fallback_with`](crate::Parser::fallback_with)",
        op: &[Arg::Ts(|f| {
            Post::KeepType(quote::quote!(.fallback_with(#f)))
        })],
    },
    Decl {
        name: "fallback_str",
        descr: "use this string as the value when the parser fails, \
            see [`Parser::fallback_str`](crate::Parser::fallback_str)",
        op: &[Arg::LitStr(|v| {
            Post::KeepType(quote::quote!(.fallback_str(#v)))
        })],
    },
    Decl {
        name: "debug_fallback",
        descr: "use the `Debug` representation of the type as a fallback",
        op: &[Arg::Bare(|| {
            Post::KeepType(quote::quote!(.debug_fallback()))
        })],
    },
    Decl {
        name: "display_fallback",
        descr: "use the `Display` representation of the type as a fallback",
        op: &[Arg::Bare(|| {
            Post::KeepType(quote::quote!(.display_fallback()))
        })],
    },
    Decl {
        name: "format_fallback",
        descr: "use a custom formatting function as a fallback",
        op: &[Arg::Ts(|f| {
            Post::KeepType(quote::quote!(.format_fallback(#f)))
        })],
    },
    Decl {
        name: "hide",
        descr: "hide the parser from the help message, \
            see [`Parser::hide`](crate::Parser::hide)",
        op: &[Arg::Bare(|| Post::KeepType(quote::quote!(.hide())))],
    },
    Decl {
        name: "hide_usage",
        descr: "hide the parser from the usage line, \
            see [`Parser::hide_usage`](crate::Parser::hide_usage)",
        op: &[Arg::Bare(|| Post::KeepType(quote::quote!(.hide_usage())))],
    },
    Decl {
        name: "custom_usage",
        descr: "replace the usage line for this parser, \
            see [`Parser::custom_usage`](crate::Parser::custom_usage)",
        op: &[Arg::Ts(|u| {
            Post::KeepType(quote::quote!(.custom_usage(#u)))
        })],
    },
    // `group` is a legacy alias of `group_help`
    Decl {
        name: "group",
        descr: "alias for `group_help`, \
            see [`Parser::group_help`](crate::Parser::group_help)",
        op: &[Arg::LitStr(|h| {
            Post::KeepType(quote::quote!(.group_help(#h)))
        })],
    },
    Decl {
        name: "group_help",
        descr: "show the parser under a shared help message, \
            see [`Parser::group_help`](crate::Parser::group_help)",
        op: &[Arg::LitStr(|h| {
            Post::KeepType(quote::quote!(.group_help(#h)))
        })],
    },
    Decl {
        name: "or_else",
        descr: "use another parser when this one fails, \
            see [`Parser::or_else`](crate::Parser::or_else)",
        op: &[Arg::Ts(|f| Post::KeepType(quote::quote!(.or_else(#f()))))],
    },
    Decl {
        name: "boxed",
        descr: "box the parser to reduce the type size",
        op: &[Arg::Bare(|| Post::KeepType(quote::quote!(.into_box())))],
    },
];

/// Command help, chained onto `.command(...)`. Aliases are handled by
/// [`NAME_DECL`] so they can be derived from the type/variant name.
pub(crate) const COMMAND_DECL: &[Decl<TokenStream>] = &[Decl {
    name: "help",
    descr: "help message for the command, \
        see [`Command::help`](crate::api::composite::Command::help)",
    op: &[Arg::Ts(|ts| quote::quote!(.help(#ts)))],
}];

/// The first keyword of the annotation, selects the mode.
pub(crate) enum OuterKind {
    Options { cargo_helper: Option<LitStr> },
    Command(Option<LitStr>),
    Construct,
    Nest,
    Literal(Option<LitStr>),
}

pub(crate) const OUTER_KIND: &[Decl<OuterKind>] = &[
    Decl {
        name: "options",
        descr: "generate an `OptionParser` with the help message, \
            the whole command line is parsed by this type, \
            see [`Parser::to_options`](crate::Parser::to_options)",
        op: &[
            Arg::Bare(|| OuterKind::Options { cargo_helper: None }),
            Arg::LitStr(|l| OuterKind::Options {
                cargo_helper: Some(l),
            }),
        ],
    },
    Decl {
        name: "command",
        descr: "turn the type into a subcommand, can be used at the top level \
            and on enum variants, \
            see [`OptionParser::command`](crate::OptionParser::command)",
        op: &[
            Arg::Bare(|| OuterKind::Command(None)),
            Arg::LitStr(|l| OuterKind::Command(Some(l))),
        ],
    },
    Decl {
        name: "construct",
        descr: "explicit marker of the construct mode, combines the fields \
            into a parser without making it an `OptionParser`, \
            see [`construct!`](crate::construct)",
        op: &[Arg::Bare(|| OuterKind::Construct)],
    },
    Decl {
        name: "nest",
        descr: "use the type as a part of a bigger parser, \
            see [`Named::nest`](crate::api::primitives::Named::nest)",
        op: &[Arg::Bare(|| OuterKind::Nest)],
    },
    Decl {
        name: "literal",
        descr: "use the type as a part of a bigger parser, selected by a \
            literal name, see [`literal`](crate::literal)",
        op: &[
            Arg::Bare(|| OuterKind::Literal(None)),
            Arg::LitStr(|l| OuterKind::Literal(Some(l))),
        ],
    },
];
