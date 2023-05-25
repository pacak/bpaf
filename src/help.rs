use crate::{
    args::{Arg, State},
    error::{Message, MissingItem},
    inner_buffer::{Buffer, Color, Style},
    item::{Item, ShortLong},
    meta_youmean::{Suggestion, Variant},
    Meta, ParseFailure,
};

fn check_conflicts(args: &State) -> Option<Message> {
    let (loser, winner) = args.conflict()?;
    Some(Message::Conflict(winner, loser))
}

fn textual_part(args: &State, ix: Option<usize>) -> Option<std::borrow::Cow<str>> {
    match args.items.get(ix?)? {
        Arg::Short(_, _, _) | Arg::Long(_, _, _) => None,
        Arg::Word(s) | Arg::PosWord(s) => Some(s.to_string_lossy()),
    }
}

fn only_once(args: &State, cur: usize) -> Option<usize> {
    if cur == 0 {
        return None;
    }
    let mut iter = args.items[..cur].iter().rev();
    let offset = match args.items.get(cur)? {
        Arg::Short(s, _, _) => iter.position(|a| a.match_short(*s)),
        Arg::Long(l, _, _) => iter.position(|a| a.match_long(l)),
        Arg::Word(_) | Arg::PosWord(_) => None,
    };
    Some(cur - offset? - 1)
}

impl Message {
    pub(crate) fn render(mut self, args: &State, inner: &Meta) -> ParseFailure {
        // try to come up with a better error message for a few cases
        match self {
            Message::Unconsumed(ix) => {
                if let Some(conflict) = check_conflicts(args) {
                    self = conflict;
                } else if let Some((ix, suggestion)) = crate::meta_youmean::suggest(args, inner) {
                    self = Message::Suggestion(ix, suggestion);
                } else if let Some(prev_ix) = only_once(args, ix) {
                    self = Message::OnlyOnce(prev_ix, ix)
                }
            }
            Message::Missing(xs) => {
                self = summarize_missing(&xs, inner, args);
            }
            _ => {}
        }

        let mut buffer = Buffer::default();

        match self {
            // already rendered
            Message::ParseFailure(f) => return f,

            // it is possible to have both missing and unconsumed
            Message::Missing(_) => {
                // this one is unreachable
            }

            Message::Unconsumed(ix) => {
                let item = &args.items[ix];
                buffer.text("`");
                buffer.write(item, Style::Invalid);
                buffer.text("` is not expected in this context");
            }

            Message::NoEnv(name) => {
                buffer.text("Environment variable `");
                buffer.invalid(name);
                buffer.text("` is not set");
                buffer.monochrome(false);
            }
            Message::StrictPos(_ix, metavar) => {
                buffer.text("Expected `");
                buffer.metavar(metavar);
                buffer.text("` to be on the right side of `");
                buffer.literal("--");
                buffer.text("`");
            }
            Message::ParseSome(s) => {
                buffer.text(s);
            }
            Message::ParseFail(s) => {
                buffer.text(s);
            }
            Message::ParseFailed(mix, s) => {
                buffer.text("Couldn't parse");
                if let Some(field) = textual_part(args, mix) {
                    buffer.text(" `");
                    buffer.invalid(&field);
                    buffer.text("`: ");
                } else {
                    buffer.text(": ");
                }
                buffer.text(&s);
            }
            Message::GuardFailed(mix, s) => {
                if let Some(field) = textual_part(args, mix) {
                    buffer.text("`");
                    buffer.invalid(&field);
                    buffer.text("`: ");
                    buffer.text(s);
                } else {
                    buffer.text("Check failed: ");
                    buffer.text(s);
                }
            }
            Message::NoArgument(x, mv) => match args.get(x + 1) {
                Some(Arg::Short(_, _, os) | Arg::Long(_, _, os)) => {
                    let arg = &args.items[x];
                    let os = &os.to_string_lossy();

                    buffer.text("`");
                    buffer.write(arg, Style::Literal);
                    buffer.text("` requires an argument `");
                    buffer.metavar(mv);
                    buffer.text("`, got a flag `");
                    buffer.write(os, Style::Invalid);
                    buffer.text("`, try `");
                    buffer.write(arg, Style::Literal);
                    buffer.literal("=");
                    buffer.write(os, Style::Literal);
                    buffer.text("` to use it as an argument");
                }
                // "Some" part of this branch is actually unreachable
                Some(Arg::Word(_) | Arg::PosWord(_)) | None => {
                    let arg = &args.items[x];
                    buffer.text("`");
                    buffer.write(arg, Style::Literal);
                    buffer.text("` requires an argument `");
                    buffer.metavar(mv);
                    buffer.text("`");
                }
            },
            Message::PureFailed(s) => {
                buffer.text(&s);
            }
            Message::Ambiguity(ix, name) => {
                let mut chars = name.chars();
                let first = chars.next().unwrap();
                let rest = chars.as_str();
                let second = chars.next().unwrap();
                let s = args.items[ix].os_str().to_str().unwrap();

                match args.path.first() {
                    Some(name) => {
                        buffer.literal(name);
                        buffer.text("supports `");
                    }
                    None => buffer.text("App supports `"),
                }
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.text("` as both an option and an option-argument, try to split `");
                buffer.write(s, Style::Literal);
                buffer.text("` into individual options (");
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.literal(" -");
                buffer.write_char(second, Style::Literal);
                buffer.literal(" ..");
                buffer.text(") or use `");
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.literal("=");
                buffer.literal(rest);
                buffer.text("` syntax to disambiguate");
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

                        buffer.text("No such ");
                        buffer.text(ty);
                        buffer.text(": `");
                        buffer.invalid(actual);
                        buffer.text("`, did you mean `");

                        match v {
                            Variant::CommandLong(name) => buffer.literal(name),
                            Variant::Flag(ShortLong::Long(l) | ShortLong::ShortLong(_, l)) => {
                                buffer.literal("--");
                                buffer.literal(l);
                            }
                            Variant::Flag(ShortLong::Short(s)) => {
                                buffer.literal("-");
                                buffer.write_char(s, Style::Literal);
                            }
                        };

                        buffer.text("`?");
                    }
                    Suggestion::MissingDash(name) => {
                        buffer.text("No such flag: `");
                        buffer.literal("-");
                        buffer.literal(name);
                        buffer.text("` (with one dash), did you mean `");
                        buffer.literal("--");
                        buffer.literal(name);
                        buffer.text("`?");
                    }
                    Suggestion::ExtraDash(name) => {
                        buffer.text("No such flag: `");
                        buffer.literal("--");
                        buffer.write_char(name, Style::Literal);
                        buffer.text("` (with two dashes), did you mean `");
                        buffer.literal("-");
                        buffer.write_char(name, Style::Literal);
                        buffer.text("`?");
                    }
                    Suggestion::Nested(x, v) => {
                        let ty = match v {
                            Variant::CommandLong(_) => "Subcommand",
                            Variant::Flag(_) => "Flag",
                        };
                        buffer.text(ty);
                        buffer.text(" `");
                        buffer.literal(actual);
                        buffer.text(
                            "` is not valid in this context, did you mean to pass it to command `",
                        );
                        buffer.literal(&x);
                        buffer.text("`?");
                    }
                }
            }
            Message::Expected(exp, actual) => {
                buffer.text("Expected ");
                match exp.len() {
                    0 => {
                        buffer.text("Expected no arguments");
                    }
                    1 => {
                        buffer.text("`");
                        buffer.write_item(&exp[0]);
                        buffer.text("`");
                    }
                    2 => {
                        buffer.text("`");
                        buffer.write_item(&exp[0]);
                        buffer.text("` or `");
                        buffer.write_item(&exp[1]);
                        buffer.text("`");
                    }
                    _ => {
                        buffer.text("`");
                        buffer.write_item(&exp[0]);
                        buffer.text("`, `");
                        buffer.write_item(&exp[1]);
                        buffer.text("`, or more");
                    }
                }
                match actual {
                    Some(actual) => {
                        buffer.text(", got `");
                        buffer.write(&args.items[actual], Style::Invalid);
                        buffer.text("`. Pass `");
                    }
                    None => {
                        buffer.text(", pass `");
                    }
                }
                buffer.literal("--help");
                buffer.text("` for usage information");
            }
            Message::Conflict(winner, loser) => {
                buffer.text("`");
                buffer.write(&args.items[loser], Style::Literal);
                buffer.text("` cannot be used at the same time as `");
                buffer.write(&args.items[winner], Style::Literal);
                buffer.text("`");
            }
            Message::OnlyOnce(_winner, loser) => {
                buffer.text("Argument `");
                buffer.write(&args.items[loser], Style::Literal);
                buffer.text("` cannot be used multiple times in this context");
            }
        };

        ParseFailure::Stderr(buffer.render(true, Color::default()))
    }
}

/// go over all the missing items, pick the left most scope
pub(crate) fn summarize_missing(items: &[MissingItem], inner: &Meta, args: &State) -> Message {
    // missing items can belong to different scopes, pick the best scope to work with
    let best_item = items
        .iter()
        .max_by_key(|item| (item.position, item.scope.start))
        .unwrap();
    let mut best_scope = best_item.scope.clone();

    let mut saw_command = false;
    let expected = items
        .iter()
        .filter_map(|i| {
            let cmd = matches!(i.item, Item::Command { .. });
            if i.scope == best_scope && !(saw_command && cmd) {
                saw_command |= cmd;
                Some(i.item.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    best_scope.start = best_scope.start.max(best_item.position);
    let mut args = args.clone();
    args.set_scope(best_scope);
    if let Some((ix, _arg)) = args.items_iter().next() {
        if let Some((ix, sugg)) = crate::meta_youmean::suggest(&args, inner) {
            Message::Suggestion(ix, sugg)
        } else {
            Message::Expected(expected, Some(ix))
        }
    } else {
        Message::Expected(expected, None)
    }
}

/*
#[inline(never)]
/// the idea is to post some context for the error
fn snip(buffer: &mut Buffer, args: &State, items: &[usize]) {
    for ix in args.scope() {
        buffer.write(ix, Style::Text);
    }
}
*/
