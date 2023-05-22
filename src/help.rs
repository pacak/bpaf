//! Improve non-parse cases
//!
//! covers `--help`, `--version` etc.
//!
//! # Notation rules:
//! - Items in backticks refer to valid flag names known to bpaf:
//!       expected `--foo`, etc
//! - Items in quotes refer to user input which might or might not be a valid flag:
//!       "--foo" cannot be used at the same time as "--bar"

use crate::{
    args::{Arg, State},
    error::{Message, MissingItem},
    info::Info,
    inner_buffer::{Buffer, Color, Style},
    item::ShortLong,
    meta_help::render_help,
    meta_youmean::{Suggestion, Variant},
    short, Error, Meta, ParseFailure, Parser,
};

struct ParseExtraParams {
    version: Option<Buffer>,
}

impl Parser<ExtraParams> for ParseExtraParams {
    fn eval(&self, args: &mut State) -> Result<ExtraParams, Error> {
        if let Ok(ok) = ParseExtraParams::help().eval(args) {
            return Ok(ok);
        }

        match &self.version {
            Some(ver) => Self::ver(ver).eval(args),
            None => Err(Error::Missing(Vec::new())),
        }
    }

    fn meta(&self) -> Meta {
        match &self.version {
            Some(ver) => Meta::And(vec![Self::help().meta(), Self::ver(ver).meta()]),
            None => Self::help().meta(),
        }
    }
}

impl ParseExtraParams {
    #[inline(never)]
    fn help() -> impl Parser<ExtraParams> {
        short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(ExtraParams::Help)
    }
    #[inline(never)]
    fn ver(version: &Buffer) -> impl Parser<ExtraParams> {
        short('V')
            .long("version")
            .help("Prints version information")
            .req_flag(ExtraParams::Version(version.clone()))
    }
}

#[derive(Clone, Debug)]
enum ExtraParams {
    Help,
    Version(Buffer),
}

impl Info {
    fn help_parser(&self) -> impl Parser<ExtraParams> {
        ParseExtraParams {
            version: self.version.clone(),
        }
    }
}

fn check_conflicts(args: &State) -> Option<Message> {
    let (loser, winner) = args.conflict()?;
    Some(Message::Conflict(winner, loser))
}

pub(crate) fn improve_error(
    args: &mut State,
    info: &Info,
    inner: &Meta,
    err: Error,
) -> ParseFailure {
    // handle --help and --version messages
    match info.help_parser().eval(args) {
        Ok(ExtraParams::Help) => {
            let path = &args.path;
            let msg = render_help(path, info, inner, &info.help_parser().meta())
                .render(false, crate::inner_buffer::Color::Monochrome);
            return ParseFailure::Stdout(msg);
        }
        Ok(ExtraParams::Version(v)) => {
            return ParseFailure::Stdout(format!("Version: {}\n", v.monochrome(false)));
        }
        Err(_) => {}
    }

    // at this point the input user gave us is invalid and we need to propose a step towards
    // improving it. Improving steps can be:
    // 1. adding something that is required but missing
    //    + works best if there's no unexpected items left
    //
    // 2. suggesting to replace something that was typed wrongly: --asmm instead of --asm
    //    + works best if there's something close enough to current item
    //
    // 3. suggesting to remove something that is totally not expected in this context
    //    + safest fallback if earlier approaches failed

    ParseFailure::Stderr(match err {
        // parse succeeded, need to explain an unused argument
        Error::ParseFailure(f) => return f,
        Error::Message(mut msg) => {
            if let Message::Unconsumed(ix) = &msg {
                if let Some(conflict) = check_conflicts(args) {
                    msg = conflict;
                } else if let Some((ix, suggestion)) = crate::meta_youmean::suggest(args, inner) {
                    msg = Message::Suggestion(ix, suggestion);
                } else {
                    msg = Message::Unconsumed(*ix);
                }
            }
            msg.render(args)
        }
        Error::Missing(xs) => summarize_missing(&xs, inner, args),
    })
}

fn textual_part(args: &State, ix: Option<usize>) -> Option<std::borrow::Cow<str>> {
    match args.items.get(ix?)? {
        Arg::Short(_, _, _) | Arg::Long(_, _, _) => None,
        Arg::Word(s) | Arg::PosWord(s) => Some(s.to_string_lossy()),
    }
}

impl Message {
    fn render(self, args: &State) -> String {
        match self {
            Message::NoEnv(name) => {
                format!("env variable {} is not set", name)
            }
            Message::StrictPos(x) => {
                format!("Expected {} to be on the right side of --", x)
            }
            Message::ParseSome(s) => s.to_string(),
            Message::Guard(_) => todo!(),
            Message::ParseFail(s) => s.to_owned(),
            Message::ParseFailed(mix, s) => match textual_part(args, mix) {
                Some(field) => format!("Couldn't parse {:?}: {}", field, s),
                None => format!("Couldn't parse: {}", s),
            },
            Message::ValidateFailed(mix, s) => match textual_part(args, mix) {
                Some(field) => format!("{:?}: {}", field, s),
                None => format!("Couldn't parse: {}", s),
            },
            Message::NoArgument(x) => match args.items.get(x + 1) {
                Some(Arg::Short(_, _, os) | Arg::Long(_, _, os)) => {
                    let arg = &args.items[x];
                    if let (Arg::Short(s, _, fos), true) = (&arg, os.is_empty()) {
                        let fos = fos.to_string_lossy();
                        let repl = fos.strip_prefix('-').unwrap().strip_prefix(*s).unwrap();
                        format!(
                            "`{}` is not accepted, try using it as `-{}={}`",
                            fos, s, repl
                        )
                    } else {
                        let os = os.to_string_lossy();
                        format!( "`{}` requires an argument, got a flag-like `{}`, try `{}={}` to use it as an argument", arg, os, arg,os)
                    }
                }
                Some(Arg::Word(_)) => unreachable!("this is an argument!"),
                Some(Arg::PosWord(_)) => todo!(),
                None => format!("{} requires an argument", args.items[x]),
            },
            Message::PureFailed(s) => s,
            Message::Unconsumed(ix) => {
                let item = &args.items[ix];
                format!("`{}` is not expected in this context", item)
            }
            Message::Ambiguity(ix, name) => {
                let items = name.chars().collect::<Vec<_>>();

                let s = args.items[ix].os_str().to_str().unwrap();

                format!(
                    "Parser supports -{} as both option and option-argument, \
                                          try to split {} into individual options (-{} -{} ..) \
                                          or use -{}={} syntax to disambiguate",
                    items[0],
                    s,
                    items[0],
                    items[1],
                    items[0],
                    &s[1 + items[0].len_utf8()..]
                )
            }
            Message::Suggestion(ix, suggestion) => {
                let actual = &args.items[ix].to_string();
                match suggestion {
                    Suggestion::Variant(v) => {
                        let ty = match &args.items[ix] {
                            _ if actual.starts_with('-') => "flag",
                            Arg::Short(_, _, _) | Arg::Long(_, _, _) => "flag",
                            Arg::Word(_) | Arg::PosWord(_) => "command or positional",
                        };

                        // TODO - avoid allocating here
                        let suggested = match v {
                            Variant::CommandLong(name) => name.to_owned(),
                            Variant::Flag(ShortLong::Long(l) | ShortLong::ShortLong(_, l)) => {
                                format!("--{}", l)
                            }
                            Variant::Flag(ShortLong::Short(s)) => {
                                format!("-{}", s)
                            }
                        };
                        format!(
                            "No such {}: `{}`, did you mean `{}`?",
                            ty, actual, suggested
                        )
                    }
                    Suggestion::MissingDash(name) => format!(
                        "No such flag: `-{}` (with one dash), did you mean `--{}`?",
                        name, name
                    ),
                    Suggestion::ExtraDash(name) => format!(
                        "No such flag: `--{}` (with two dashes), did you mean `-{}`?",
                        name, name
                    ),
                    Suggestion::Nested(x, v) => {
                        let ty = match v {
                            Variant::CommandLong(_) => "Subcommand",
                            Variant::Flag(_) => "Flag",
                        };
                        format!("{} `{}` is not valid in this context, did you mean to pass it to command `{}`?", ty, actual, x)
                    }
                }

                /*
                let current = match &args.items[ix]
                {
                    Arg::Word(w) | Arg::PosWord(w) => {
                        let s = w.to_str().unwrap();
                        if s.starts_with('-') {
                            format!("flag: `{}`", s)
                        } else {
                            format!("command or positional: `{}`", s)
                        }
                    }
                    x => format!("{:?}", x),
                    Arg::Short(_, _, _) => todo!(),
                    Arg::Long(_, _, _) => todo!(),
                };
                let replacement = match suggestion {
                    Suggestion::MissingDash(x) => format!("`--{}` (with two dashes)", x),
                    x => format!("{:?}", x),
                    Suggestion::Variant(_) => todo!(),
                    Suggestion::Nested(_, _) => todo!(),
                };

                format!("No such {}, did you mean {}", current, replacement)*/
            }
            Message::Conflict(winner, loser) => {
                format!(
                    "\"{}\" cannot be used at the same time as \"{}\"",
                    args.items[loser], args.items[winner]
                )
            }
        }
    }
}

#[inline(never)]
pub(crate) fn summarize_missing(items: &[MissingItem], inner: &Meta, args: &State) -> String {
    // missing items can belong to different scopes, pick the best scope to work with
    let (best_pos, mut best_scope) = match items
        .iter()
        .max_by_key(|item| (item.position, item.scope.start))
    {
        Some(item) => (item.position, item.scope.clone()),
        None => return "Nothing expected, but parser somehow failed...".to_owned(),
    };

    let meta = Meta::Or(
        items
            .iter()
            .filter_map(|i| {
                if i.scope == best_scope {
                    Some(Meta::from(i.item.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    )
    .normalized(false);

    best_scope.start = best_scope.start.max(best_pos);
    let mut args = args.clone();
    args.set_scope(best_scope);
    if let Some(x) = args.peek() {
        if let Some((ix, sugg)) = crate::meta_youmean::suggest(&args, inner) {
            let msg = Message::Suggestion(ix, sugg);
            msg.render(&args)
        } else {
            let mut b = Buffer::default();
            b.write_str("Expected `", Style::Text);
            b.write_meta(&meta, false);
            b.write_str("`, got `", Style::Text);
            b.write_str(&x.to_string(), Style::Invalid);
            b.write_str("`. Pass `", Style::Text);
            b.write_str("--help", Style::Literal);
            b.write_str("` for usage information", Style::Text);
            b.render(true, Color::Monochrome)
        }
    } else {
        let mut b = Buffer::default();
        b.write_str("Expected `", Style::Text);
        b.write_meta(&meta, false);
        b.write_str("`, pass `", Style::Text);
        b.write_str("--help", Style::Literal);
        b.write_str("` for usage information", Style::Text);

        b.render(true, Color::Monochrome)
    }
}
