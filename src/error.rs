use std::ops::Range;

use crate::{
    args::{Arg, State},
    inner_buffer::{Block, Buffer, Color, Style, Token},
    item::Item,
    item::ShortLong,
    meta_help::Metavar,
    meta_youmean::{Suggestion, Variant},
    Meta,
};

/// Unsuccessful command line parsing outcome, internal representation
#[derive(Debug)]
pub struct Error(pub(crate) Message);

impl Error {
    pub(crate) fn combine_with(self, other: Self) -> Self {
        Error(self.0.combine_with(other.0))
    }
}

#[derive(Debug)]
pub(crate) enum Message {
    // those can be caught ---------------------------------------------------------------
    /// Tried to consume an env variable with no fallback, variable was not set
    NoEnv(&'static str),

    /// User specified an error message on some
    ParseSome(&'static str),

    /// User asked for parser to fail explicitly
    ParseFail(&'static str),

    /// pure_with failed to parse a value
    PureFailed(String),

    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<MissingItem>),

    // those cannot be caught-------------------------------------------------------------
    /// Parsing failed and this is the final output
    ParseFailure(ParseFailure),
    /// Tried to consume a strict positional argument, value was present but was not strictly
    /// positional
    StrictPos(usize, Metavar),

    /// Parser provided by user failed to parse a value
    ParseFailed(Option<usize>, String),

    /// Parser provided by user failed to validate a value
    GuardFailed(Option<usize>, &'static str),

    /// Argument requres a value but something else was passed,
    /// required: --foo <BAR>
    /// given: --foo --bar
    ///        --foo -- bar
    ///        --foo
    NoArgument(usize, Metavar),

    /// Parser is expected to consume all the things from the command line
    /// this item will contain an index of the unconsumed value
    Unconsumed(/* TODO - unused? */ usize),

    /// argument is ambigoups - parser can accept it as both a set of flags and a short flag with no =
    Ambiguity(usize, String),

    /// Suggested fixes for typos or missing input
    Suggestion(usize, Suggestion),

    /// Two arguments are mutually exclusive
    /// --release --dev
    Conflict(/* winner */ usize, usize),

    /// Expected one or more items in the scope, got someting else if any
    Expected(Vec<Item>, Option<usize>),

    /// Parameter is accepted but only once
    OnlyOnce(/* winner */ usize, usize),
}

impl Message {
    pub(crate) fn can_catch(&self) -> bool {
        match self {
            Message::NoEnv(_)
            | Message::ParseSome(_)
            | Message::ParseFail(_)
            | Message::Missing(_)
            | Message::PureFailed(_) => true,
            Message::StrictPos(_, _)
            | Message::ParseFailed(_, _)
            | Message::GuardFailed(_, _)
            | Message::Unconsumed(_)
            | Message::Ambiguity(_, _)
            | Message::Suggestion(_, _)
            | Message::Conflict(_, _)
            | Message::ParseFailure(_)
            | Message::Expected(_, _)
            | Message::OnlyOnce(_, _)
            | Message::NoArgument(_, _) => false,
        }
    }
}

/// Missing item in a context
#[derive(Debug, Clone)]
pub struct MissingItem {
    /// Item that is missing
    pub(crate) item: Item,
    /// Position it is missing from - exact for positionals, earliest possible for flags
    pub(crate) position: usize,
    /// Range where search was performed, important for combinators that narrow the search scope
    /// such as adjacent
    pub(crate) scope: Range<usize>,
}

impl Message {
    #[must_use]
    pub(crate) fn combine_with(self, other: Self) -> Self {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            // help output takes priority
            (a @ Message::ParseFailure(_), _) => a,
            (_, b @ Message::ParseFailure(_)) => b,

            // combine missing elements
            (Message::Missing(mut a), Message::Missing(mut b)) => {
                a.append(&mut b);
                Message::Missing(a)
            }

            // otherwise earliest wins
            (a, b) => {
                if a.can_catch() {
                    b
                } else {
                    a
                }
            }
        }
    }
}

/// Unsuccessful command line parsing outcome, use it for unit tests
///
/// Useful for unit testing for user parsers, consume it with
/// [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`]
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Print this to stdout and exit with success code
    Stdout(String),
    /// Print this to stderr and exit with failure code
    Stderr(String),
}

impl ParseFailure {
    /// Returns the contained `stderr` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stdout`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Self::Stderr(err) => err,
            Self::Stdout(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    /// Returns the contained `stdout` values - for unit tests
    ///
    /// # Panics
    ///
    /// Panics if failure contains `stderr`
    #[allow(clippy::must_use_candidate)]
    #[track_caller]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Self::Stdout(err) => err,
            Self::Stderr(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }

    /// Run an action appropriate to the failure and produce the exit code
    ///
    /// Prints a message to `stdout` or `stderr` and returns the exit code
    #[allow(clippy::must_use_candidate)]
    pub fn exit_code(self) -> i32 {
        match self {
            ParseFailure::Stdout(msg) => {
                print!("{}", msg); // completions are sad otherwise
                0
            }
            ParseFailure::Stderr(msg) => {
                eprintln!("{}", msg);
                1
            }
        }
    }
}

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
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.write(item, Style::Invalid);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" is not expected in this context");
            }

            Message::NoEnv(name) => {
                buffer.text("Environment variable ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.invalid(name);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" is not set");
                buffer.monochrome(false);
            }
            Message::StrictPos(_ix, metavar) => {
                buffer.text("Expected ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.metavar(metavar);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" to be on the right side of ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.literal("--");
                buffer.token(Token::BlockEnd(Block::TermRef));
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
                    buffer.text(" ");
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.invalid(&field);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(": ");
                } else {
                    buffer.text(": ");
                }
                buffer.text(&s);
            }
            Message::GuardFailed(mix, s) => {
                if let Some(field) = textual_part(args, mix) {
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.invalid(&field);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(": ");
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

                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.write(arg, Style::Literal);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(" requires an argument ");
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.metavar(mv);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(", got a flag ");
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.write(os, Style::Invalid);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(", try ");
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.write(arg, Style::Literal);
                    buffer.literal("=");
                    buffer.write(os, Style::Literal);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(" to use it as an argument");
                }
                // "Some" part of this branch is actually unreachable
                Some(Arg::Word(_) | Arg::PosWord(_)) | None => {
                    let arg = &args.items[x];
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.write(arg, Style::Literal);
                    buffer.token(Token::BlockEnd(Block::TermRef));
                    buffer.text(" requires an argument ");
                    buffer.token(Token::BlockStart(Block::TermRef));
                    buffer.metavar(mv);
                    buffer.token(Token::BlockEnd(Block::TermRef));
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
                        buffer.text("supports ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                    }
                    None => {
                        buffer.text("App supports ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                    }
                }
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" as both an option and an option-argument, try to split ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.write(s, Style::Literal);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" into individual options (");
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.literal(" -");
                buffer.write_char(second, Style::Literal);
                buffer.literal(" ..");
                buffer.text(") or use ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.literal("-");
                buffer.write_char(first, Style::Literal);
                buffer.literal("=");
                buffer.literal(rest);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" syntax to disambiguate");
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
                        buffer.text(": ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.invalid(actual);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(", did you mean ");
                        buffer.token(Token::BlockStart(Block::TermRef));

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

                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text("?");
                    }
                    Suggestion::MissingDash(name) => {
                        buffer.text("No such flag: ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal("-");
                        buffer.literal(name);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(" (with one dash), did you mean ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal("--");
                        buffer.literal(name);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text("?");
                    }
                    Suggestion::ExtraDash(name) => {
                        buffer.text("No such flag: ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal("--");
                        buffer.write_char(name, Style::Literal);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(" (with two dashes), did you mean ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal("-");
                        buffer.write_char(name, Style::Literal);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text("?");
                    }
                    Suggestion::Nested(x, v) => {
                        let ty = match v {
                            Variant::CommandLong(_) => "Subcommand",
                            Variant::Flag(_) => "Flag",
                        };
                        buffer.text(ty);
                        buffer.text(" ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal(actual);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(
                            " is not valid in this context, did you mean to pass it to command ",
                        );
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.literal(&x);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text("?");
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
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write_item(&exp[0]);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                    }
                    2 => {
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write_item(&exp[0]);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(" or ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write_item(&exp[1]);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                    }
                    _ => {
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write_item(&exp[0]);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(", ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write_item(&exp[1]);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(", or more");
                    }
                }
                match actual {
                    Some(actual) => {
                        buffer.text(", got ");
                        buffer.token(Token::BlockStart(Block::TermRef));
                        buffer.write(&args.items[actual], Style::Invalid);
                        buffer.token(Token::BlockEnd(Block::TermRef));
                        buffer.text(". Pass ");
                    }
                    None => {
                        buffer.text(", pass ");
                    }
                }
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.literal("--help");
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" for usage information");
            }
            Message::Conflict(winner, loser) => {
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.write(&args.items[loser], Style::Literal);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" cannot be used at the same time as ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.write(&args.items[winner], Style::Literal);
                buffer.token(Token::BlockEnd(Block::TermRef));
            }
            Message::OnlyOnce(_winner, loser) => {
                buffer.text("Argument ");
                buffer.token(Token::BlockStart(Block::TermRef));
                buffer.write(&args.items[loser], Style::Literal);
                buffer.token(Token::BlockEnd(Block::TermRef));
                buffer.text(" cannot be used multiple times in this context");
            }
        };

        ParseFailure::Stderr(buffer.render_console(true, Color::default()))
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
