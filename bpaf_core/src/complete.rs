//!  Shell completion
//!
//! 1. pick what parsers can possibly match the input
//! 2. set the wakeup reason accordingly - it should contain the
//!    name right away so we just use it
//! 3. wake up the tasks, they append the help to the name and produce
//!    the recovery reply error
//! 4. those errors are combined until the top level
//! 5. at the top level - something renders them to the console output

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fmt::Write as _,
    str::FromStr,
};

use crate::{
    Error, KillReason, Lit, Metavar, Name, ParseFailure, Reason, Scope, Triggers,
    arg::{Adjacency, Arg},
    error::CompValue,
    pecking::PeckingOrder,
};

fn append_help(buf: &mut OsString, rev: ShellRender, help: Option<&str>) {
    match rev {
        ShellRender::Dumb => {}
        ShellRender::Test | ShellRender::DumbTab => {
            if let Some(help) = help {
                _ = write!(buf, "\t{help}");
            }
        }
        ShellRender::Zsh => {}
    }
}

impl CompReply {
    #[inline(never)]
    pub(crate) fn literal(rev: ShellRender, name: &Lit, help: Option<&str>) -> Self {
        use std::fmt::Write;
        let mut buf = OsString::new();

        match rev {
            ShellRender::Test => {
                buf.push("lit: ");
                _ = write!(&mut buf, "{name}");
                if let Some(h) = help {
                    buf.push("\t");
                    buf.push(h);
                }
                buf.push("\n");
            }
            ShellRender::Zsh => {
                buf.push("local -a _bpaf_descr\n");
                buf.push("_bpaf_descr=('");
                zsh_push_single_quoted(&mut buf, &name.to_string());
                if let Some(h) = help {
                    buf.push("  -- ");
                    zsh_push_compadd_description(&mut buf, h);
                }
                buf.push("')\n");
                buf.push("compadd -l -d _bpaf_descr -- '");
                zsh_push_single_quoted(&mut buf, &name.to_string());
                buf.push("'\n");
            }
            ShellRender::Dumb => {
                _ = write!(&mut buf, "{name}");
                buf.push("\n");
            }
            ShellRender::DumbTab => {
                _ = write!(&mut buf, "{name}");
                if let Some(h) = help {
                    buf.push("\t");
                    buf.push(h);
                }
                buf.push("\n");
            }
        }
        Self(buf)
    }

    #[inline(never)]
    pub(crate) fn named(
        rev: ShellRender,
        name: &Name,
        meta: Option<Metavar>,
        help: Option<&str>,
    ) -> Self {
        use std::fmt::Write;
        let mut buf = OsString::new();

        if rev == ShellRender::Test {
            buf.push("named: ");
        }

        match rev {
            ShellRender::Zsh => {
                buf.push("local -a _bpaf_descr\n");
                buf.push("_bpaf_descr=('");
                zsh_push_single_quoted(&mut buf, &name.to_string());
                if let Some(h) = help {
                    buf.push("  -- ");
                    zsh_push_compadd_description(&mut buf, h);
                }
                buf.push("')\n");
                buf.push("compadd -l -d _bpaf_descr -- '");
                zsh_push_single_quoted(&mut buf, &name.to_string());
                if let Some(m) = meta {
                    buf.push("=");
                    zsh_push_single_quoted(&mut buf, &m.to_string());
                }
                buf.push("'\n");
            }
            ShellRender::Dumb => {
                if meta.is_some() {
                    _ = write!(&mut buf, "{name}=");
                } else {
                    _ = write!(&mut buf, "{name}");
                }
                buf.push("\n");
            }
            ShellRender::Test | ShellRender::DumbTab => {
                _ = write!(&mut buf, "{name}");
                if let Some(m) = meta {
                    buf.push("=");
                    _ = write!(&mut buf, "{m}");
                }
                append_help(&mut buf, rev, help);
                buf.push("\n");
            }
        }

        Self(buf)
    }
}

/// A generated shell completion reply
///
/// Can contain multiple lines,
#[derive(Clone, Default, Debug)]
pub(crate) struct CompReply(pub(crate) OsString);

impl std::ops::Add for CompReply {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0.push(&rhs.0);
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
            let script = OsString::from(Shell::from_str(shell)?.render_for(&self.app));
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

impl CompValue {
    pub(crate) fn into_os(self) -> OsString {
        self.into_reply().0
    }
    pub(crate) fn into_reply(self) -> CompReply {
        let CompValue {
            name,
            value,
            meta,
            shell,
            help,
        }: CompValue = self;

        let mut buf = OsString::new();

        match shell {
            ShellRender::Test => {
                buf.push("unh: ");
                if let Some(prefix) = &name {
                    buf.push(prefix.as_ref());
                }
                if value.is_empty() {
                    _ = write!(&mut buf, "{meta}");
                } else {
                    buf.push(&value);
                }
                if value.is_empty() {
                    if let Some(h) = help {
                        buf.push("\t");
                        buf.push(h);
                    }
                } else {
                    buf.push("\t");
                    _ = write!(&mut buf, "{meta}");
                }
                buf.push("\n");
            }
            ShellRender::Zsh => {
                if let Some(prefix) = &name {
                    buf.push("local -a _bpaf_descr\n");
                    buf.push("_bpaf_descr=('");
                    zsh_push_single_quoted(&mut buf, prefix.as_ref());
                    if value.is_empty() {
                        zsh_push_single_quoted(&mut buf, &meta.to_string());
                    } else {
                        zsh_push_single_quoted(&mut buf, value.to_str().unwrap_or(""));
                    }
                    if let Some(h) = help {
                        buf.push("  -- ");
                        zsh_push_compadd_description(&mut buf, h);
                    }
                    buf.push("')\n");
                    buf.push("compadd -l -d _bpaf_descr -- '");
                    zsh_push_single_quoted(&mut buf, prefix.as_ref());
                    if value.is_empty() {
                        zsh_push_single_quoted(&mut buf, &meta.to_string());
                    } else {
                        zsh_push_single_quoted(&mut buf, value.to_str().unwrap_or(""));
                    }
                    buf.push("'\n");
                } else {
                    // Positional argument - use compadd
                    let val_str = if value.is_empty() {
                        meta.to_string()
                    } else {
                        value.to_str().unwrap_or("").to_string()
                    };
                    zsh_push_compadd_value(&mut buf, &meta.to_string(), &val_str, help);
                }
            }
            ShellRender::Dumb => {
                if let Some(prefix) = &name {
                    buf.push(prefix.as_ref());
                }
                if value.is_empty() {
                    _ = write!(&mut buf, "{meta}");
                } else {
                    buf.push(&value);
                }
                buf.push("\n");
            }
            ShellRender::DumbTab => {
                if let Some(prefix) = &name {
                    buf.push(prefix.as_ref());
                }
                if value.is_empty() {
                    _ = write!(&mut buf, "{meta}");
                } else {
                    buf.push(&value);
                }
                if value.is_empty() {
                    if let Some(h) = help {
                        buf.push("\t");
                        buf.push(h);
                    }
                } else {
                    buf.push("\t");
                    _ = write!(&mut buf, "{meta}");
                }
                buf.push("\n");
            }
        }

        CompReply(buf)
    }
}

impl ShellRender {
    fn debug(&self, buf: &mut OsString, text: &str) {
        if *self == ShellRender::Test {
            buf.push(text)
        }
    }
}

/// Escape a string for safe inclusion inside a zsh single-quoted context.
/// In single quotes, only single quotes themselves need escaping,
/// using the '\'' pattern (end quote, escaped quote, start quote).
fn zsh_push_single_quoted(buf: &mut OsString, s: &str) {
    let mut utf8 = [0; 4];
    for c in s.chars() {
        if c == '\'' {
            buf.push("'\\''");
        } else {
            buf.push(&c.encode_utf8(&mut utf8));
        }
    }
}

/// Filter a string for use as a description in _arguments spec.
/// Removes characters that would break single-quoted parsing.
fn zsh_push_description(buf: &mut OsString, s: &str) {
    let mut utf8 = [0; 4];
    for c in s.chars() {
        // Skip characters that would break _arguments spec parsing inside single quotes
        if matches!(c, '\'' | '"' | '[' | ']') {
            continue;
        }
        buf.push(&c.encode_utf8(&mut utf8));
    }
}

/// Write a description for compadd display string.
/// Format: "VALUE  -- Description"
/// No special filtering needed beyond single-quote escaping.
fn zsh_push_compadd_description(buf: &mut OsString, s: &str) {
    zsh_push_single_quoted_part(buf, s);
}

/// Write a TAG name for _arguments, stripping angle brackets.
/// Angle brackets in TAG prevent description from displaying.
fn zsh_push_tag(buf: &mut OsString, s: &str) {
    let mut utf8 = [0; 4];
    for c in s.chars() {
        // Skip angle brackets - they break description display in TAG position
        if matches!(c, '<' | '>') {
            continue;
        }
        buf.push(&c.encode_utf8(&mut utf8));
    }
}

/// Write a compadd-based completion for a value with optional description.
/// Uses the format:
///   local -a _bpaf_descr
///   _bpaf_descr=('VALUE  -- HINT')
///   compadd -l -d _bpaf_descr -- 'VALUE'
fn zsh_push_compadd_value(buf: &mut OsString, _desc: &str, value: &str, hint: Option<&str>) {
    buf.push("local -a _bpaf_descr\n");
    buf.push("_bpaf_descr=('");
    zsh_push_single_quoted(buf, value);
    if let Some(h) = hint {
        buf.push("  -- ");
        zsh_push_compadd_description(buf, h);
    }
    buf.push("')\n");
    buf.push("compadd -l -d _bpaf_descr -- '");
    zsh_push_single_quoted(buf, value);
    buf.push("'\n");
}

/// Write content without single-quote escaping (for display strings).
fn zsh_push_single_quoted_part(buf: &mut OsString, s: &str) {
    let mut utf8 = [0; 4];
    for c in s.chars() {
        buf.push(&c.encode_utf8(&mut utf8));
    }
}

/// Dump
pub(crate) fn complete_value(err: Error, completer: &StringCompleter) -> Error {
    let Error::CompValue(CompValue {
        name,
        value,
        meta,
        shell,
        help: _,
    }) = err
    else {
        return err;
    };

    let value = value.to_string_lossy();

    let mut buf = OsString::new();

    // For zsh positional arguments, collect all values to generate compadd calls
    if shell == ShellRender::Zsh && name.is_none() {
        let values = completer(&value);
        if values.is_empty() {
            return Error::CompReply(CompReply(buf));
        }
        buf.push("local -a _bpaf_descr\n");
        for (v, hint) in &values {
            buf.push("_bpaf_descr+=('");
            zsh_push_single_quoted(&mut buf, v.as_str());
            if let Some(h) = hint {
                buf.push("  -- ");
                zsh_push_compadd_description(&mut buf, h.as_str());
            }
            buf.push("')\n");
        }
        buf.push("compadd -l -d _bpaf_descr -- ");
        for (v, _hint) in &values {
            buf.push("'");
            zsh_push_single_quoted(&mut buf, v.as_str());
            buf.push("' ");
        }
        buf.push("\n");
        return Error::CompReply(CompReply(buf));
    }

    for (value, hint) in completer(&value) {
        shell.debug(&mut buf, "val: ");
        match shell {
            ShellRender::Test | ShellRender::DumbTab | ShellRender::Dumb => {
                if let Some(name) = &name {
                    buf.push(name.as_ref());
                }
                buf.push(&value);
                if let Some(hint) = &hint {
                    _ = writeln!(&mut buf, "\t{hint}");
                } else {
                    buf.push("\n");
                }
            }
            ShellRender::Zsh => {
                if let Some(prefix) = &name {
                    buf.push("local -a _bpaf_descr\n");
                    buf.push("_bpaf_descr=('");
                    zsh_push_single_quoted(&mut buf, prefix.as_ref());
                    zsh_push_single_quoted(&mut buf, &value);
                    if let Some(h) = &hint {
                        buf.push("  -- ");
                        zsh_push_compadd_description(&mut buf, h);
                    }
                    buf.push("')\n");
                    buf.push("compadd -l -d _bpaf_descr -- '");
                    zsh_push_single_quoted(&mut buf, prefix.as_ref());
                    zsh_push_single_quoted(&mut buf, &value);
                    buf.push("'\n");
                }
            }
        }
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
        value: &'a OsStr,
    },

    /// An attempt to complete a positional value or a named value but the value is
    /// not adjacent to the name
    Value { value: &'a OsStr },

    /// An attempt to complete a literal string
    Literal { name: Lit<'a> },
}

impl CReq<'_> {
    /// Prefer long name where possible
    pub(crate) fn improve(&mut self, other: Self) {
        let CReq::Named { name: n1, .. } = self else {
            return;
        };
        let CReq::Named { name: n2, .. } = other else {
            return;
        };
        if matches!(n1, Name::Short(_)) && matches!(n2, Name::Long(_)) {
            *n1 = n2;
        }
    }
}

pub(crate) fn handle_subparser_complete(err: Error) -> Error {
    match err {
        Error::CompReply(CompReply(items)) => ParseFailure::Console(items).into(),
        Error::CompValue(cv) => ParseFailure::Console(cv.into_os()).into(),
        _ => err,
    }
}

/// For each possible set of triggers also return what
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
            if let Some(order) = triggers.args.get(&name) {
                let req = CReq::NamedValue { name, adj, value };
                out.push((req, order));
            }
        }
        Arg::Pos { value } if strict_pos => {
            let req = CReq::Value { value };
            out.push((req, &triggers.pos))
        }
        Arg::Pos { value: os_val } => {
            let Some(value) = os_val.to_str() else {
                let req = CReq::Value { value: os_val };
                out.push((req, &triggers.pos));
                todo!("Completing non-utf?");
            };

            if !value.starts_with("-") {
                let req = CReq::Value { value: os_val };
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

        let mut m = BTreeMap::default();
        // Normally we traverse each pecking order at once since only sum items can run
        // in parallel, but for autocomplete any item from a prod can run so we'll run
        // orders independent from each other
        for (req, order) in orders_by_prefix(arg, &self.triggers, self.ctx.strict_pos.get()) {
            self.mixer.push_peck(order);
            self.to_wake.extend(self.mixer.for_wake(&self.tasks));

            for id in self.to_wake.drain(..) {
                m.entry(id)
                    .and_modify(|prev: &mut CReq| prev.improve(req.clone()))
                    .or_insert(req.clone());
            }
        }

        // TODO - add all the active checks here?
        for id in self.triggers.checks.keys() {
            m.insert(*id, CReq::Value { value: arg_os });
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
