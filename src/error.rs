use std::ops::Range;

use crate::{
    args::{Arg, State},
    buffer::{Block, Color, Doc, Style, Token},
    item::{Item, ShortLong},
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

    /// Tried to consume a non-strict positional argument, but the value was strict
    NonStrictPos(usize, Metavar),

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
            | Message::PureFailed(_)
            | Message::NonStrictPos(_, _) => true,
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
/// When [`OptionParser::run_inner`](crate::OptionParser::run_inner) produces `Err(ParseFailure)`
/// it means that the parser couldn't produce the value it supposed to produce and the program
/// should terminate.
///
/// If you are handling variants manually - `Stdout` contains formatted output and you can use any
/// logging framework to produce the output, `Completion` should be printed to stdout unchanged -
/// shell completion mechanism relies on that. In both cases application should exit with error
/// code of 0. `Stderr` variant indicates a genuinly parsing error which should be printed to
/// stderr or a logging framework of your choice as an error and the app should exit with error
/// code of 1. [`ParseFailure::exit_code`] is a helper method that performs printing and produces
/// the exit code to use.
///
/// For purposes of for unit testing for user parsers, you can consume it with
/// [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`] - both of which produce a
/// an unformatted `String` that parser might produce if failure type is correct or panics
/// otherwise.
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Print this to stdout and exit with success code
    Stdout(Doc, bool),
    /// This also goes to stdout with exit code of 0,
    /// this cannot be Doc because completion needs more control about rendering
    Completion(String),
    /// Print this to stderr and exit with failure code
    Stderr(Doc),
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
            Self::Stderr(err) => err.monochrome(true),
            Self::Completion(..) | Self::Stdout(..) => panic!("not an stderr: {:?}", self),
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
            Self::Stdout(err, full) => err.monochrome(full),
            Self::Completion(s) => s,
            Self::Stderr(..) => panic!("not an stdout: {:?}", self),
        }
    }

    /// Returns the exit code for the failure
    #[allow(clippy::must_use_candidate)]
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Stdout(..) | Self::Completion(..) => 0,
            Self::Stderr(..) => 1,
        }
    }

    #[doc(hidden)]
    #[deprecated = "Please use ParseFailure::print_message, with two s"]
    pub fn print_mesage(&self, max_width: usize) {
        self.print_message(max_width)
    }

    /// Prints a message to `stdout` or `stderr` appropriate to the failure.
    pub fn print_message(&self, max_width: usize) {
        let color = Color::default();
        match self {
            ParseFailure::Stdout(msg, full) => {
                println!("{}", msg.render_console(*full, color, max_width));
            }
            ParseFailure::Completion(s) => {
                print!("{}", s);
            }
            ParseFailure::Stderr(msg) => {
                #[allow(unused_mut)]
                let mut error;
                #[cfg(not(feature = "color"))]
                {
                    error = "Error: ";
                }

                #[cfg(feature = "color")]
                {
                    error = String::new();
                    color.push_str(Style::Invalid, &mut error, "Error: ");
                }

                eprintln!("{}{}", error, msg.render_console(true, color, max_width));
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
        Arg::ArgWord(s) | Arg::Word(s) | Arg::PosWord(s) => Some(s.to_string_lossy()),
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
        Arg::ArgWord(_) | Arg::Word(_) | Arg::PosWord(_) => None,
    };
    Some(cur - offset? - 1)
}

impl Message {
    #[allow(clippy::too_many_lines)] // it's a huge match with lots of simple cases
    pub(crate) fn render(mut self, args: &State, meta: &Meta) -> ParseFailure {
        // try to come up with a better error message for a few cases
        match self {
            Message::Unconsumed(ix) => {
                if let Some(conflict) = check_conflicts(args) {
                    self = conflict;
                } else if let Some(prev_ix) = only_once(args, ix) {
                    self = Message::OnlyOnce(prev_ix, ix);
                } else if let Some((ix, suggestion)) = crate::meta_youmean::suggest(args, meta) {
                    self = Message::Suggestion(ix, suggestion);
                }
            }
            Message::Missing(xs) => {
                self = summarize_missing(&xs, meta, args);
            }
            _ => {}
        }

        let mut doc = Doc::default();
        match self {
            // already rendered
            Message::ParseFailure(f) => return f,

            // this case is handled above
            Message::Missing(_) => {
                // this one is unreachable
            }

            // Error: --foo is not expected in this context
            Message::Unconsumed(ix) => {
                let item = &args.items[ix];
                doc.token(Token::BlockStart(Block::TermRef));
                doc.write(item, Style::Invalid);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" is not expected in this context");
            }

            // Error: environment variable FOO is not set
            Message::NoEnv(name) => {
                doc.text("environment variable ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.invalid(name);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" is not set");
            }

            // Error: FOO expected to be  in the right side of --
            Message::StrictPos(_ix, metavar) => {
                doc.text("expected ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.metavar(metavar);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" to be on the right side of ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.literal("--");
                doc.token(Token::BlockEnd(Block::TermRef));
            }

            // Error: FOO expected to be on the left side of --
            Message::NonStrictPos(_ix, metavar) => {
                doc.text("expected ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.metavar(metavar);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" to be on the left side of ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.literal("--");
                doc.token(Token::BlockEnd(Block::TermRef));
            }

            // Error: <message from some or fail>
            Message::ParseSome(s) | Message::ParseFail(s) => {
                doc.text(s);
            }

            // Error: couldn't parse FIELD: <FromStr message>
            Message::ParseFailed(mix, s) => {
                doc.text("couldn't parse");
                if let Some(field) = textual_part(args, mix) {
                    doc.text(" ");
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.invalid(&field);
                    doc.token(Token::BlockEnd(Block::TermRef));
                }
                doc.text(": ");
                doc.text(&s);
            }

            // Error: ( FIELD:  | check failed: ) <message from guard>
            Message::GuardFailed(mix, s) => {
                if let Some(field) = textual_part(args, mix) {
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.invalid(&field);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(": ");
                } else {
                    doc.text("check failed: ");
                }
                doc.text(s);
            }

            // Error: --foo requires an argument FOO, got a flag --bar, try --foo=-bar to use it as an argument
            // Error: --foo requires an argument FOO
            Message::NoArgument(x, mv) => match args.get(x + 1) {
                Some(Arg::Short(_, _, os) | Arg::Long(_, _, os)) => {
                    let arg = &args.items[x];
                    let os = &os.to_string_lossy();

                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.write(arg, Style::Literal);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(" requires an argument ");
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.metavar(mv);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(", got a flag ");
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.write(os, Style::Invalid);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(", try ");
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.write(arg, Style::Literal);
                    doc.literal("=");
                    doc.write(os, Style::Literal);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(" to use it as an argument");
                }
                // "Some" part of this branch is actually unreachable
                Some(Arg::ArgWord(_) | Arg::Word(_) | Arg::PosWord(_)) | None => {
                    let arg = &args.items[x];
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.write(arg, Style::Literal);
                    doc.token(Token::BlockEnd(Block::TermRef));
                    doc.text(" requires an argument ");
                    doc.token(Token::BlockStart(Block::TermRef));
                    doc.metavar(mv);
                    doc.token(Token::BlockEnd(Block::TermRef));
                }
            },
            // Error: <message from pure_with>
            Message::PureFailed(s) => {
                doc.text(&s);
            }
            // Error: app supports -f as both an option and an option-argument, try to split -foo
            // into invididual options (-f -o ..) or use -f=oo syntax to disambiguate
            Message::Ambiguity(ix, name) => {
                let mut chars = name.chars();
                let first = chars.next().unwrap();
                let rest = chars.as_str();
                let second = chars.next().unwrap();
                let s = args.items[ix].os_str().to_str().unwrap();

                if let Some(name) = args.path.first() {
                    doc.literal(name);
                    doc.text(" supports ");
                } else {
                    doc.text("app supports ");
                }

                doc.token(Token::BlockStart(Block::TermRef));
                doc.literal("-");
                doc.write_char(first, Style::Literal);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" as both an option and an option-argument, try to split ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.write(s, Style::Literal);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" into individual options (");
                doc.literal("-");
                doc.write_char(first, Style::Literal);
                doc.literal(" -");
                doc.write_char(second, Style::Literal);
                doc.literal(" ..");
                doc.text(") or use ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.literal("-");
                doc.write_char(first, Style::Literal);
                doc.literal("=");
                doc.literal(rest);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" syntax to disambiguate");
            }
            // Error: No such (flag|argument|command), did you mean  ...
            Message::Suggestion(ix, suggestion) => {
                let actual = &args.items[ix].to_string();
                match suggestion {
                    Suggestion::Variant(v) => {
                        let ty = match &args.items[ix] {
                            _ if actual.starts_with('-') => "flag",
                            Arg::Short(_, _, _) | Arg::Long(_, _, _) => "flag",
                            Arg::ArgWord(_) => "argument value",
                            Arg::Word(_) | Arg::PosWord(_) => "command or positional",
                        };

                        doc.text("no such ");
                        doc.text(ty);
                        doc.text(": ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.invalid(actual);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(", did you mean ");
                        doc.token(Token::BlockStart(Block::TermRef));

                        match v {
                            Variant::CommandLong(name) => doc.literal(name),
                            Variant::Flag(ShortLong::Long(l) | ShortLong::Both(_, l)) => {
                                doc.literal("--");
                                doc.literal(l);
                            }
                            Variant::Flag(ShortLong::Short(s)) => {
                                doc.literal("-");
                                doc.write_char(s, Style::Literal);
                            }
                        };

                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text("?");
                    }
                    Suggestion::MissingDash(name) => {
                        doc.text("no such flag: ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal("-");
                        doc.literal(name);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(" (with one dash), did you mean ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal("--");
                        doc.literal(name);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text("?");
                    }
                    Suggestion::ExtraDash(name) => {
                        doc.text("no such flag: ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal("--");
                        doc.write_char(name, Style::Literal);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(" (with two dashes), did you mean ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal("-");
                        doc.write_char(name, Style::Literal);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text("?");
                    }
                    Suggestion::Nested(x, v) => {
                        let ty = match v {
                            Variant::CommandLong(_) => "subcommand",
                            Variant::Flag(_) => "flag",
                        };
                        doc.text(ty);
                        doc.text(" ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal(actual);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(
                            " is not valid in this context, did you mean to pass it to command ",
                        );
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.literal(&x);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text("?");
                    }
                }
            }
            // Error: Expected (no arguments|--foo), got ..., pass --help
            Message::Expected(exp, actual) => {
                doc.text("expected ");
                match exp.len() {
                    0 => {
                        doc.text("no arguments");
                    }
                    1 => {
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write_item(&exp[0]);
                        doc.token(Token::BlockEnd(Block::TermRef));
                    }
                    2 => {
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write_item(&exp[0]);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(" or ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write_item(&exp[1]);
                        doc.token(Token::BlockEnd(Block::TermRef));
                    }
                    _ => {
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write_item(&exp[0]);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(", ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write_item(&exp[1]);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(", or more");
                    }
                }
                match actual {
                    Some(actual) => {
                        doc.text(", got ");
                        doc.token(Token::BlockStart(Block::TermRef));
                        doc.write(&args.items[actual], Style::Invalid);
                        doc.token(Token::BlockEnd(Block::TermRef));
                        doc.text(". Pass ");
                    }
                    None => {
                        doc.text(", pass ");
                    }
                }
                doc.token(Token::BlockStart(Block::TermRef));
                doc.literal("--help");
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" for usage information");
            }

            // Error: --intel cannot be used at the same time as --att
            Message::Conflict(winner, loser) => {
                doc.token(Token::BlockStart(Block::TermRef));
                doc.write(&args.items[loser], Style::Literal);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" cannot be used at the same time as ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.write(&args.items[winner], Style::Literal);
                doc.token(Token::BlockEnd(Block::TermRef));
            }

            // Error: argument FOO cannot be used multiple times in this context
            Message::OnlyOnce(_winner, loser) => {
                doc.text("argument ");
                doc.token(Token::BlockStart(Block::TermRef));
                doc.write(&args.items[loser], Style::Literal);
                doc.token(Token::BlockEnd(Block::TermRef));
                doc.text(" cannot be used multiple times in this context");
            }
        };

        ParseFailure::Stderr(doc)
    }
}

/// go over all the missing items, pick the left most scope
pub(crate) fn summarize_missing(items: &[MissingItem], inner: &Meta, args: &State) -> Message {
    // missing items can belong to different scopes, pick the best scope to work with
    let best_item = match items
        .iter()
        .max_by_key(|item| (item.position, item.scope.start))
    {
        Some(x) => x,
        None => return Message::ParseSome("parser requires an extra flag, argument or parameter, but its name is hidden by the author"),
    };

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
