//! This module contains the basic consumers for the arguments
//!
//! They know how to interact with executor, deal with early exit when necessary, etc.
//! Anything else gets built on top of those consumers
//!
//! only bits here know about consuming, cursor, and events.
//! Everything here is either fixed type or (in rare cases &dyn...)

use crate::{
    Arg, CReq, Conflict, Error, KillReason, Lit, Literal, Metavar, Named, Op, Problem, RawCtx,
    Reason, Scope, TChange, TTarget, arg, error::CV, lex_os_arg, r#yield,
};
use std::{ffi::OsStr, rc::Rc};

/// Register triggers, yield for wakeup, then de-register triggers.
/// Used by all async parser methods to wait for a trigger event.
macro_rules! with_trigger_and_yield {
    ($self:ident, $targets:expr) => {{
        $self.with_trigger(TChange::Add, $targets);
        r#yield().await;
        $self.with_trigger(TChange::Remove, $targets);
    }};
}

impl TTarget {
    fn into_conflict(self, pos: u32) -> Option<Conflict> {
        match self {
            TTarget::Arg(name) | TTarget::Flag(name) => Some(Conflict::Named { pos, name }),
            TTarget::Pos => Some(Conflict::Pos { pos }),
            TTarget::Check(_) => None, // TODO ... technically I can store check inside of here and
            // invoke it later..
            TTarget::Literal(name) => Some(Conflict::Lit { pos, name }),
        }
    }
}

impl<'p> RawCtx<'p> {
    fn with_trigger(&self, change: TChange, items: impl IntoIterator<Item = TTarget>) {
        let cur = self.current_task.borrow();
        let mut pending = self.pending_ops.borrow_mut();
        for target in items {
            pending.push_back(Op::Trigger {
                change,
                target,
                parent: cur.parent_id,
                id: cur.id,
                kind: cur.parent_kind,
            });
        }
    }

    pub(crate) fn consume(&self, cnt: u32) {
        self.current_task.borrow_mut().consumed += cnt;
    }

    pub(crate) async fn parse_pos(
        &self,
        help: Option<&'static str>,
        meta: Metavar,
    ) -> Result<Option<&'p OsStr>, Error> {
        with_trigger_and_yield!(self, [TTarget::Pos]);
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(Arg::Pos { value }) => {
                self.consume(1);
                *self.current_value.borrow_mut() = Some(value);
                Ok(Some(value))
            }
            Reason::Arg(_) => unreachable!(),
            Reason::Kill(KillReason::NoMatchingInput) => Ok(None),
            Reason::Kill(KillReason::Conflict) => Err(self.record_conflicts([TTarget::Pos])),
            r @ (Reason::Kill(KillReason::TooShort)
            | Reason::Pass
            | Reason::Push
            | Reason::ChildProgress(_)) => unreachable!("non-leaf wakeup: {r:?}"),
            Reason::Complete(shell, creqs) => match creqs.as_slice() {
                [CReq::Value { value }] => {
                    let prefix_value = if value.is_empty() {
                        meta.to_string()
                    } else {
                        (*value).to_owned()
                    };
                    let cv = CV {
                        has_value: true,
                        prefix_len: 0,
                        prefix_value,
                        meta_only: value.is_empty(),
                        help,
                        shell: *shell,
                    };
                    Err(Error::CompValue(cv))
                }
                _ => unreachable!(),
            },
        }
    }

    pub(crate) async fn await_passing_check(
        &self,
        meta: Metavar,
        check: Rc<dyn Fn(&OsStr) -> bool>,
    ) -> Result<bool, Error> {
        with_trigger_and_yield!(self, [TTarget::Check(check.clone())]);
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(_) => {
                self.consume(1);
                Ok(true)
            }
            Reason::Kill(KillReason::Conflict) => Err(Error::Silent("Killed by conflict")),
            Reason::Kill(KillReason::NoMatchingInput) => Ok(false),
            r @ (Reason::Kill(KillReason::TooShort)
            | Reason::Pass
            | Reason::Push
            | Reason::ChildProgress(_)) => unreachable!("non-leaf wakeup: {r:?}"),
            Reason::Complete(shell, _) => {
                let value = self.args[self.cursor.get()].as_os_str();
                let prefix_value = if value.is_empty() {
                    meta.to_string()
                } else {
                    value.to_string_lossy().into_owned()
                };
                let cv = CV {
                    prefix_value,
                    has_value: true,
                    prefix_len: 0,
                    help: None,
                    shell: *shell,
                    meta_only: value.is_empty(),
                };
                Err(Error::CompValue(cv))
            }
        }
    }

    /// Wake up on one of the literals, return it
    pub(crate) async fn parse_literal(
        &self,
        literal: &Literal,
    ) -> Result<Option<Lit<'static>>, Error> {
        with_trigger_and_yield!(self, literal.triggers());
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(Arg::Pos { value }) => {
                self.consume(1);
                Ok(Some(arg::as_name(value).unwrap().into_owned()))
            }
            Reason::Arg(_) => unreachable!(),
            Reason::Kill(KillReason::NoMatchingInput) => Ok(None),
            Reason::Kill(KillReason::Conflict) => Err(self.record_conflicts(literal.triggers())),
            r @ (Reason::Kill(_) | Reason::Pass | Reason::Push | Reason::ChildProgress(_)) => {
                unreachable!("non-leaf wakeup: {r:?}")
            }
            Reason::Complete(shell, creqs) => {
                let best = literal.names.iter().find(|n| {
                    creqs
                        .iter()
                        .any(|creq| matches!(creq, CReq::Literal { name } if *n == name))
                });
                let cv = CV {
                    prefix_value: best.unwrap().to_string(),
                    has_value: false,
                    prefix_len: 0,
                    help: literal.help,
                    shell: *shell,
                    meta_only: false,
                };
                Err(Error::CompValue(cv))
            }
        }
    }

    pub(crate) async fn parse_arg(
        &self,
        named: &Named,
        meta: Metavar,
    ) -> Result<Option<&'p OsStr>, Error> {
        with_trigger_and_yield!(self, named.arg_triggers());
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(arg) => self.parse_arg_consume(arg.clone(), meta, named.help),

            Reason::Kill(KillReason::Conflict) => Err(self.record_conflicts(named.arg_triggers())),
            Reason::Kill(KillReason::NoMatchingInput) => Ok(None),

            r @ (Reason::Kill(KillReason::TooShort)
            | Reason::Pass
            | Reason::Push
            | Reason::ChildProgress(_)) => unreachable!("non-leaf wakeup: {r:?}"),

            Reason::Complete(shell, creqs) => {
                let mut value_adj = None;
                let best = named.names.iter().find(|n| {
                    creqs.iter().any(|creq| match creq {
                        CReq::Named { name } => *n == name,
                        CReq::NamedValue { name, adj, value } if *n == name => {
                            value_adj = Some((*adj, *value));
                            true
                        }
                        _ => false,
                    })
                });
                let best = best.unwrap();

                let mut prefix_value = best.to_string();
                let mut value_len = 0;
                if let Some((a, v)) = value_adj {
                    use std::fmt::Write as _;
                    value_len = v.len();
                    _ = write!(&mut prefix_value, "{a}{v}");
                }
                let prefix_len = prefix_value.len() as u32 - value_len as u32;
                let cv = CV {
                    prefix_value,
                    has_value: value_adj.is_some(),
                    prefix_len,
                    help: named.help,
                    shell: *shell,
                    meta_only: false,
                };

                Err(Error::CompValue(cv))
            }
        }
    }

    pub(self) fn parse_arg_consume(
        &self,
        arg: Arg<'p>,
        meta: Metavar,
        help: Option<&'static str>,
    ) -> Result<Option<&'p OsStr>, Error> {
        match arg {
            Arg::Pos { .. } => unreachable!(),
            Arg::Named { name, value: None } => {
                let cursor = self.cursor.get() + 1;
                let Some(next) = self.args.get(cursor) else {
                    return Err(Error::Problem(
                        cursor - 1,
                        Problem::WrongArgument {
                            name: name.into_owned(),
                            value: None,
                        },
                    ));
                };
                let pos = self.cursor.get();
                match lex_os_arg(next) {
                    Arg::Named { .. } => Err(Error::Problem(
                        pos,
                        Problem::WrongArgument {
                            name: name.into_owned(),
                            value: Some(next.to_os_string()),
                        },
                    )),
                    Arg::Pos { value } => {
                        self.consume(2);
                        // Unlike most of the parsers, arguments can trigger completions
                        // in two possible ways: to complete the name and to complete the value.
                        // Second case becomes active when user types the name, a space and, then
                        // tries to complete the value
                        if cursor + 1 == self.args.len()
                            && let Some(shell) = self.args.complete
                        {
                            let prefix_value = if value.is_empty() {
                                meta.to_string()
                            } else {
                                value.to_string_lossy().into_owned()
                            };
                            let cv = CV {
                                prefix_value,
                                has_value: true,
                                prefix_len: 0,
                                help,
                                shell: shell.into(),
                                meta_only: value.is_empty(),
                            };
                            return Err(Error::CompValue(cv));
                        }
                        *self.current_value.borrow_mut() = Some(value);
                        Ok(Some(value))
                    }
                }
            }
            Arg::Named {
                name: _,
                value: Some((_adj, val)),
            } => {
                *self.current_value.borrow_mut() = Some(val);
                self.consume(1);
                Ok(Some(val))
            }
        }
    }

    /// Parse a flag by any one of the given names
    ///
    /// - `Ok(true)` when encounters a name
    /// - `Ok(false)` when it gets terminated by "no such item"
    /// - `Err(Error::Killed)` when it gets out-consumed by something else
    pub(crate) async fn parse_flag(&self, named: &Named) -> Result<bool, Error> {
        with_trigger_and_yield!(self, named.flag_triggers());
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(arg) => self.parse_flag_consume(arg),
            Reason::Kill(KillReason::Conflict) => Err(self.record_conflicts(named.flag_triggers())),
            Reason::Kill(KillReason::NoMatchingInput) => Ok(false),
            r @ (Reason::Kill(KillReason::TooShort)
            | Reason::Pass
            | Reason::Push
            | Reason::ChildProgress(_)) => unreachable!("non-leaf wakeup: {r:?}"),

            Reason::Complete(shell, creqs) => {
                let best = named.names.iter().find(|n| {
                    creqs
                        .iter()
                        .any(|creq| matches!(creq, CReq::Named { name } if *n == name))
                });
                let cv = CV {
                    prefix_value: best.unwrap().to_string(),
                    has_value: false,
                    prefix_len: 0,
                    help: named.help,
                    shell: *shell,
                    meta_only: false,
                };
                Err(Error::CompValue(cv))
            }
        }
    }

    /// Consume a present argument, advance
    #[inline(always)]
    pub(self) fn parse_flag_consume(&self, arg: &Arg) -> Result<bool, Error> {
        match arg {
            Arg::Named { name, value } => match value {
                Some((adj, val)) => {
                    let problem = Problem::ExpectedFlag {
                        name: name.clone().into_owned(),
                        adj: *adj,
                        value: val.to_os_string(),
                    };
                    let pos = self.cursor.get();
                    Err(Error::Problem(pos, problem))
                }
                None => {
                    self.consume(1);
                    Ok(true)
                }
            },
            Arg::Pos { value: _ } => unreachable!(),
        }
    }

    /// All the `items` where killed as we parsed an item at current cursor position
    fn record_conflicts(&self, items: impl IntoIterator<Item = TTarget>) -> Error {
        // The idea for conflict tracking is to record that we could have consumed
        // a flag / literal, but instead consumed something else at a given cursor position
        //
        // We do this so when we encounter something we can't parse - we check if it was ever
        // possible to parse it before. This gives a position we parsed instead
        let pos = self.cursor.get();
        self.conflicts
            .borrow_mut()
            .extend(items.into_iter().filter_map(|t| t.into_conflict(pos)));
        Error::Silent("Killed by conflict")
    }

    /// Keep track of children progress and trim under-consuming ones
    ///
    /// Since a proper parser must consume all the items from the input and only
    /// one child from a Sum can succeed - we must make sure that only children
    /// that consume at least as much as the best consuming one remains.
    ///
    /// Sum task will be woken up with a `ChildProgress` reason multiple times
    /// as children make progress.
    ///
    /// This method could live inside of an `impl Parser for Sum`. Having it here
    /// generates smaller code since the way to progress doesn't depend on
    /// the output type
    #[inline(never)]
    pub(crate) async fn all_children_finish(&self, mut scopes: Vec<Scope>) {
        loop {
            match *self.wakeup_reason.borrow() {
                Reason::ChildProgress(ref ids) => {
                    scopes.retain(|scope| {
                        let lives = ids.as_slice().iter().any(|id| scope.contains(*id));
                        if !lives {
                            let op = Op::KillScope {
                                scope: *scope,
                                cursor: self.cursor.get(),
                                reason: KillReason::Conflict,
                            };
                            self.pending_ops.borrow_mut().push_back(op);
                        }
                        lives
                    });
                    if scopes.len() == 1 {
                        // This is an optimization. Once there's only one branch
                        // left in a sum - it's not different from a regular parser.
                        // De-registering the sum avoids extra wakeups.
                        self.pending_ops.borrow_mut().push_back(Op::DeregisterSum {
                            id: self.current_task.borrow().id,
                        });
                    }
                }
                Reason::Push => {
                    if self.current_task.borrow().pending == 0 {
                        break;
                    }
                }
                _ => break,
            }
            r#yield().await; // end of stage 2
        }
    }
}

impl Named {
    fn flag_triggers(&self) -> impl Iterator<Item = TTarget> {
        self.names.iter().cloned().map(TTarget::Flag)
    }
    fn arg_triggers(&self) -> impl Iterator<Item = TTarget> {
        self.names.iter().cloned().map(TTarget::Arg)
    }
}

impl Literal {
    fn triggers(&self) -> impl Iterator<Item = TTarget> {
        self.names.iter().cloned().map(TTarget::Literal)
    }
}
