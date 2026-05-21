//!  Shell completion
//!
//! 1. pick what parsers can possibly match the input
//! 2. set the wakeup reason accordingly - it should contain the
//!    name right away so we just use it
//! 3. wake up the tasks, they append the help to the name and produce
//!    the recovery reply error
//! 4. those errors are combined until the top level
//! 5. at the top level - something renders them to the console output

use std::{collections::BTreeMap, ffi::OsStr, fmt::Write as _, str::FromStr};

use crate::{
    Error, Id, KillReason, Lit, Metavar, Name, ParseFailure, Reason, Scope, Triggers,
    arg::{Adjacency, Arg},
    error::CV,
    pecking::PeckingOrder,
};

pub(crate) struct CompItem<'a> {
    value: String,
    help: Option<&'a str>,
}

impl From<CV> for CompReply {
    fn from(value: CV) -> Self {
        let ci = CompItem {
            value: value.prefix_value,
            help: value.help,
        };
        let mut res = String::new();
        ci.render(value.shell, &mut res);
        CompReply(res)
    }
}

impl CompItem<'_> {
    fn render(&self, shell: ShellRender, buf: &mut String) {
        match shell {
            ShellRender::Test | ShellRender::DumbTab => self.render_simple(self.help, buf),
            ShellRender::Dumb => self.render_simple(None, buf),
            ShellRender::Zsh => self.render_zsh(buf),
        }
    }

    fn render_simple(&self, help: Option<&str>, buf: &mut String) {
        struct Esc<'a>(&'a str);
        impl std::fmt::Display for Esc<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for c in self.0.chars() {
                    if c == '\n' {
                        f.write_char('\\')?;
                    }
                    f.write_char(c)?;
                }
                Ok(())
            }
        }
        _ = match help {
            Some(help) => writeln!(buf, "{v}\t{h}", v = Esc(&self.value), h = Esc(help)),
            None => writeln!(buf, "{v}", v = Esc(&self.value)),
        }
    }

    fn render_zsh(&self, buf: &mut String) {
        /// There's two ways to pass descriptions to `compadd`. Both rely on
        /// passing an array via `-d` flag.
        /// You can pass either a variable name: `compadd -d descrs -- values`
        /// Or a literal array: `compadd -d '(descr1 descr2)'`.
        ///
        /// Both methods require some escaping. If you pass an array directly to `-d`
        /// `compadd` will use its internal parser: items are separated by space-ish
        /// symbols or comma. Quotes don't have special meaning.
        ///
        /// `Esc` escapes all the things `compadd` considers separators.
        struct Esc<'a>(&'a str);

        impl std::fmt::Display for Esc<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for c in self.0.chars() {
                    if c.is_whitespace() || c == ',' || c == ')' || c == '\'' {
                        f.write_char('\\')?;
                    }
                    f.write_char(c)?;
                }
                Ok(())
            }
        }

        _ = match self.help {
            Some(help) => writeln!(
                buf,
                "compadd -l -d '({val}\\ \\ --\\ {help})' -- {val}",
                help = Esc(help),
                val = Esc(&self.value)
            ),
            None => writeln!(buf, "compadd -- {val}", val = Esc(&self.value)),
        }
    }
}

impl CompReply {
    #[inline(never)]
    pub(crate) fn literal(rev: ShellRender, name: &Lit, help: Option<&'static str>) -> Self {
        let mut buf = String::new();
        let ci = CompItem {
            value: name.to_string(),
            help,
        };
        ci.render(rev, &mut buf);
        Self(buf)
    }

    #[inline(never)]
    pub(crate) fn named(
        rev: ShellRender,
        name: &Name,
        meta: Option<Metavar>,
        help: Option<&'static str>,
    ) -> Self {
        let mut buf = String::new();

        let ci = CompItem {
            value: name.to_string(),
            help,
        };
        ci.render(rev, &mut buf);
        Self(buf)
    }
}

/// A generated shell completion reply
///
/// Can contain multiple lines,
#[derive(Clone, Default, Debug)]
pub(crate) struct CompReply(pub(crate) String);

impl std::ops::Add for CompReply {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0.push_str(&rhs.0);
        self
    }
}

pub(crate) type StringCompleter = Box<dyn Fn(&str) -> Vec<(String, Option<String>)>>;

/// What is the current shell
///
/// Several shells can share completion rendering mechanisms, but `Shell` is what we get
/// from a user and what is used to produce a completion stub.
#[derive(Debug, Copy, Clone)]
pub enum Shell {
    Test,
    Bash,
    Zsh,
    Fish,
    Elvish,
}

/// How to render shell completion
///
/// Several shells can share the same rendering mechanism, gets created from [`Shell`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ShellRender {
    /// Prefixes each line with a short description, but behaves like a `DumbTab` otherwise
    Test,
    /// For shells that can separate completed value from a hint - `fish`
    DumbTab,
    /// Only render the completed value, never the help - used in shells like `bash`
    Dumb,
    /// Zsh specifically
    Zsh,
}

impl From<Shell> for ShellRender {
    fn from(value: Shell) -> Self {
        match value {
            Shell::Test => Self::Test,
            Shell::Bash => Self::Dumb,
            Shell::Zsh => Self::Zsh,
            Shell::Fish => Self::DumbTab,
            Shell::Elvish => Self::Dumb,
        }
    }
}

impl std::str::FromStr for Shell {
    type Err = ParseFailure;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "elvish" => Ok(Self::Elvish),
            _ => {
                let msg = format!("`{s}` is not a shell supported for completion"); // TODO - colors
                Err(ParseFailure::Stderr(msg.into()))
            }
        }
    }
}

impl Shell {
    pub fn render_for(self, name: &str) -> String {
        match self {
            Shell::Bash => format!(
                r#"_bpaf_{name}_dynamic_completion() {{
    mapfile -t COMPREPLY < <( BPAF_COMPLETE_REV=10 "${{COMP_WORDS[@]}}" 2>/dev/null )
}}
complete -o nosort -F _bpaf_{name}_dynamic_completion {name}
"#
            ),
            Shell::Zsh => format!(
                r#"#compdef {name}
eval "$( BPAF_COMPLETE_REV=7 "$words[@]" )"
"#
            ),
            Shell::Fish => format!(
                r#"function _bpaf_{name}_dynamic_completion
    set -l cmd (commandline -opc)
    set -l current (commandline -ct)
    env BPAF_COMPLETE_REV=9 $cmd $current 2>/dev/null
end

complete --command {name} --no-files --arguments '(_bpaf_{name}_dynamic_completion)'
"#
            ),
            Shell::Elvish => format!(
                "\
set edit:completion:arg-completer[{name}] = {{ |@args| var args = $args[1..];
     var @lines = ( {name} --bpaf-complete-rev=1 $@args );
     use str;
     for line $lines {{
         var @arg = (str:split \"\\t\" $line)
         try {{
             edit:complex-candidate $arg[0] &display=( printf \"%-19s %s\" $arg[0] $arg[1] )
         }} catch {{
             edit:complex-candidate $line
         }}
     }}
}}",
            ),
            Shell::Test => format!("dummy completion for app {name}"),
        }
    }

    pub fn from_rev(rev: usize) -> Result<Self, ParseFailure> {
        match rev {
            1 => Ok(Self::Elvish),
            7 => Ok(Self::Zsh),
            8 => Ok(Self::Bash),
            9 => Ok(Self::Fish),
            10 => Ok(Self::Bash),
            _ => {
                let msg = format!(
                    "Unsupported complete revision ({rev}), \
                    try to regenerate completion stub for your shell?"
                ); // TODO - colors
                Err(ParseFailure::Stderr(msg.into()))
            }
        }
    }
}

impl crate::args::Args {
    #[inline(never)]
    pub(crate) fn check_complete(&mut self) -> Result<(), ParseFailure> {
        if let Ok(rev) = std::env::var("BPAF_COMPLETE_REV") {
            self.complete = Some(Shell::from_rev(
                rev.parse().expect("Rev should be numeric"),
            )?);
            Ok(())
        } else if let Some(crate::arg::Arg::Named {
            name: Name::Long(name),
            value,
        }) = self.get(0).map(crate::arg::lex_os_arg)
            && let Some(shell) = name.strip_prefix("bpaf-complete-style-")
            && value.is_none()
            && self.items.len() == 1
        {
            let script = Shell::from_str(shell)?.render_for(&self.app);
            Err(ParseFailure::Console(script))
        } else {
            Ok(())
        }
    }
}

impl From<CompReply> for Error {
    fn from(value: CompReply) -> Self {
        Error::CompReply(value)
    }
}

/// Dump expand possible completions and dump them as CompReply
///
/// Can't keep intermediate representation in the error since it doesn't support
/// multiple of them. On the other hand, it's not needed.
pub(crate) fn complete_value(err: Error, completer: &StringCompleter) -> Error {
    let Error::CompValue(CV {
        mut prefix_value,
        has_value: true,
        prefix_len,
        help: _,
        shell,
        meta_only,
    }) = err
    else {
        return err;
    };
    let len = prefix_len as usize;
    if meta_only {
        prefix_value.clear();
    }

    let mut buf = String::new();
    let (name, vprefix) = prefix_value.split_at(len);

    for (mut value, help) in completer(vprefix) {
        if !name.is_empty() {
            value = format!("{name}{value}");
        }
        let ci = CompItem {
            value,
            help: help.as_deref(),
        };
        ci.render(shell, &mut buf);
    }

    Error::CompReply(CompReply(buf))
}

#[derive(Debug, Clone)]
/// Potential completion request
///
/// For every completion request we look at all the active parsers and for every potentially
/// matching one generate a guess.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CReq<'a> {
    /// An attempt to complete this name
    Named { name: Name<'a> },

    /// An attempt to complete a named argument with a value
    ///
    /// So user typed `--name=VALUE`, `-n=VALUE` or `-nVALUE`, value can be incomplete
    NamedValue {
        name: Name<'a>,
        adj: Adjacency,
        value: &'a str,
    },

    /// An attempt to complete a positional value or a named value but the value is
    /// not adjacent to the name
    Value { value: &'a str },

    /// An attempt to complete a literal string
    Literal { name: Lit<'a> },
}

pub(crate) fn handle_subparser_complete(err: Error) -> Error {
    match err {
        Error::CompReply(CompReply(items)) => ParseFailure::Console(items).into(),
        Error::CompValue(cv) => ParseFailure::Console(CompReply::from(cv).0).into(),
        _ => err,
    }
}

/// Collect all the possible triggers that can fire given a possibly incomplete input
pub(crate) fn orders_by_prefix<'a, 'b>(
    arg: Arg<'b>,
    triggers: &'a Triggers,
    strict_pos: bool,
) -> Vec<(CReq<'b>, &'a PeckingOrder)>
where
    'b: 'a,
{
    let mut out = Vec::new();

    // TODO - want to include all the "any" parsers here
    // self.pecking_push(Some(&triggers.any));
    match arg {
        Arg::Named {
            name: Name::Long(name_prefix),
            value: None,
        } => {
            for (name, order) in triggers.args.iter().chain(triggers.flags.iter()) {
                if let Name::Long(ln) = name
                    && ln.starts_with(name_prefix.as_ref())
                {
                    let req = CReq::Named { name: name.clone() };
                    out.push((req, order));
                }
            }
        }
        Arg::Named { name, value: None } => {
            let req = CReq::Named { name: name.clone() };
            if let Some(order) = triggers.args.get(&name) {
                out.push((req.clone(), order));
            }

            if let Some(order) = triggers.flags.get(&name) {
                out.push((req.clone(), order));
            }
        }
        Arg::Named {
            name,
            value: Some((adj, value)),
        } => {
            if let Some(order) = triggers.args.get(&name)
                && let Some(value) = value.to_str()
            {
                let req = CReq::NamedValue { name, adj, value };
                out.push((req, order));
            }
        }
        Arg::Pos { value } if strict_pos => {
            if let Some(value) = value.to_str() {
                let req = CReq::Value { value };
                out.push((req, &triggers.pos))
            }
        }
        Arg::Pos { value } => {
            let Some(value) = value.to_str() else {
                return out;
            };

            if !value.starts_with("-") {
                let req = CReq::Value { value };
                out.push((req, &triggers.pos))
            }

            if value.is_empty() || value == "-" {
                for (name, order) in triggers.args.iter().chain(triggers.flags.iter()) {
                    let req = CReq::Named { name: name.clone() };
                    out.push((req, order));
                }
            } else if value == "--" {
                for (name, order) in triggers.args.iter().chain(triggers.flags.iter()) {
                    if matches!(name, Name::Long(_)) {
                        let req = CReq::Named { name: name.clone() };
                        out.push((req, order));
                    }
                }
            }

            if value.is_empty() || !value.starts_with("-") {
                for (name, order) in triggers.literal.iter() {
                    if name.starts_with(value) {
                        let req = CReq::Literal { name: name.clone() };
                        out.push((req, order));
                    }
                }
            }
        }
    };

    out.sort_by(|v1, v2| v1.0.cmp(&v2.0));

    out
}

impl<'a, 'p> crate::Executor<'a, 'p> {
    pub(crate) fn check_autocomplete(&mut self, arg_os: &'p OsStr) -> Option<Result<(), Error>> {
        // complete should only do anything when requested and we are at the very last item
        let shell = self.ctx.args.complete?;
        if self.ctx.cursor.get() + 1 != self.ctx.args.len() {
            return None;
        }

        let arg = crate::arg::lex_os_arg(arg_os);

        let Some(value) = arg_os.to_str() else {
            // Explicitly ignoring non-utf8 items. Produce an empty completion result
            return Some(Err(Error::CompReply(CompReply::default()))); // TODO - this can be const
        };

        let mut m = BTreeMap::<Id, Vec<CReq<'p>>>::default();
        // Normally we traverse each pecking order at once since only sum items can run
        // in parallel, but for autocomplete any item from a prod can run so we'll run
        // orders independent from each other
        for (req, order) in orders_by_prefix(arg, &self.triggers, self.ctx.strict_pos.get()) {
            self.mixer.push_peck(order);
            self.to_wake.extend(self.mixer.for_wake(&self.tasks));

            for id in self.to_wake.drain(..) {
                m.entry(id).or_default().push(req.clone());
            }
        }

        for id in self.triggers.checks.keys() {
            m.entry(*id).or_default().push(CReq::Value { value });
        }

        // even if there's nothing to return - let's produce an empty set of results so we
        // get a completion instead of the result
        if m.is_empty() {
            return Some(Err(Error::CompReply(CompReply::default())));
        }

        for (id, reason) in m {
            *self.ctx.wakeup_reason.borrow_mut() = Reason::Complete(shell.into(), reason);

            let mut task = self.tasks.remove(&id).unwrap();
            let r = self.ctx.poll_in_context(&mut task);
            if task.info.parent_id.is_root() {
                continue;
            }
            self.to_propagate
                .push_back((task.info.id, task.info.parent_id, task.info.consumed));
            assert!(r);
        }
        self.kill_in_scope(Scope::ALL, KillReason::NoMatchingInput);
        self.propagate();
        Some(Ok(()))
    }
}
