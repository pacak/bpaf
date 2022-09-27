use std::cell::RefCell;
use std::{collections::BTreeSet, rc::Rc};

use bpaf::*;

use crate::add::{parse_add, Add};
use crate::build::{parse_build, Build};
use crate::check::{parse_check, Check};
use crate::clean::{parse_clean, Clean};
use crate::metadata::{matching_targets, Exec, MatchKind};
use crate::run::{parse_run, Run};
use crate::test::{parse_test, Test};
use crate::unique_match;

#[derive(Debug, Clone, Bpaf)]
pub enum Cauwugo {
    Add(#[bpaf(external(parse_add))] Add),
    Build(#[bpaf(external(parse_build))] Build),
    Check(#[bpaf(external(parse_check))] Check),
    Run(#[bpaf(external(parse_run))] Run),
    Clean(#[bpaf(external(parse_clean))] Clean),
    Test(#[bpaf(external(parse_test))] Test),
    // bench ?
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
///       _~^~^~_
///   \) / >   < \ (/   run cargo commands with the power of dynamic completion
///     '_   ω   _'
///    /  '-----' /
pub struct CauwugoOpts {
    /// Cauwugo will print underlying cargo command before proceeding
    pub bpaf_verbose: bool,

    /// Cauwugo won't perform actions, only show the intended command line
    pub bpaf_dry: bool,

    #[bpaf(external)]
    pub cauwugo: Cauwugo,
}

/// Fill in completion suggestions for all the previous inputs and all the available items
///
/// `in_vec` considers last item of inputs as one being currently completed and matches
/// it as prefix, all other items - as already completed, so it uses an exact match
pub fn suggest_available<'a, I, S>(inputs: &[S], avail: I) -> Vec<(String, Option<String>)>
where
    I: Iterator<Item = &'a str>,
    S: AsRef<str>,
{
    if inputs.is_empty() {
        avail.map(|a| (a.to_owned(), None)).collect::<Vec<_>>()
    } else {
        let active = inputs.last();
        let complete = inputs[..inputs.len() - 1]
            .iter()
            .map(AsRef::as_ref)
            .collect::<BTreeSet<_>>();
        avail
            .filter_map(|av| {
                if complete.contains(av) {
                    None
                } else if active.map_or(true, |ac| av.starts_with(ac.as_ref())) {
                    Some((av.to_owned(), None))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

pub fn complete_target_kind<S: AsRef<str>>(
    input: &[S],
    package: Option<&'static str>,
    kinds: &'static [&'static str],
) -> Vec<(String, Option<String>)> {
    let targets =
        matching_targets(MatchKind::exact(package), MatchKind::Any, kinds).map(Exec::name);
    suggest_available(input, targets)
}

pub fn parse_runnable(package: Rc<RefCell<Option<&'static str>>>) -> impl Parser<Exec> {
    let bin = parse_bin(package.clone());
    let example = parse_example(package.clone());

    let package0 = package.clone();
    const RUNNABLE: &[&str] = &["bin", "example"];
    let pos = positional::<String>("EXE")
        .complete(move |i| complete_target_kind(&[i], *package.borrow(), RUNNABLE))
        .parse::<_, _, String>(move |name| {
            unique_match(
                matching_targets(
                    MatchKind::exact(*package0.borrow()),
                    MatchKind::Exact(&name),
                    RUNNABLE,
                ),
                &name,
            )
        })
        .complete_style(CompleteDecor::VisibleGroup("Binaries and examples"));

    construct!([pos, bin, example])
}

pub fn parse_testable(package: Rc<RefCell<Option<&'static str>>>) -> impl Parser<Exec> {
    let bin = parse_bin(package.clone());
    let example = parse_example(package.clone());
    let test = parse_exec_target('t', "test", "use this test", &["test"], package.clone());
    let bench = parse_exec_target('b', "bench", "use this benchmark", &["bench"], package);
    construct!([bin, example, test, bench])
}

fn parse_bin(package: Rc<RefCell<Option<&'static str>>>) -> impl Parser<Exec> {
    parse_exec_target('b', "bin", "use this binary", &["bin"], package)
}

fn parse_example(package: Rc<RefCell<Option<&'static str>>>) -> impl Parser<Exec> {
    parse_exec_target('e', "example", "use this example", &["example"], package)
}

fn parse_exec_target(
    short_name: char,
    long_name: &'static str,
    help: &'static str,
    kinds: &'static [&'static str],
    package: Rc<RefCell<Option<&'static str>>>,
) -> impl Parser<Exec> {
    let package0 = package.clone();
    long(long_name)
        .short(short_name)
        .help(help)
        .argument::<String>("NAME")
        .complete(move |i| complete_target_kind(&[i], *package.borrow(), kinds))
        .parse::<_, _, String>(move |name| {
            unique_match(
                matching_targets(
                    MatchKind::exact(*package0.borrow()),
                    MatchKind::Exact(&name),
                    kinds,
                ),
                &name,
            )
        })
}
