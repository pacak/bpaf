//! This module contains the basic consumers for the arguments
//!
//! They know how to interact with executor, deal with early exit when necessary, etc.
//! Anything else gets built on top of those consumers
//!
//! only bits here know about consuming, cursor, and events.
//! Everything here is either fixed type or (in rare cases &dyn...)

use crate::*;

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

impl RawCtx {
    fn with_trigger(&self, change: TChange, items: impl IntoIterator<Item = TTarget>) {
        let (parent, id) = self.task_parent_and_id();
        let mut pending = self.pending_ops.borrow_mut();
        for target in items {
            pending.push_back(Op::Trigger {
                change,
                target,
                parent,
                id,
            });
        }
    }

    pub(crate) fn consume(&self, cnt: u32) {
        self.current_task.borrow_mut().consumed += cnt;
    }

    async fn wait_for(
        &self,
        items: impl IntoIterator<Item = TTarget> + Clone,
    ) -> Result<Option<Arg<'_>>, Error> {
        self.with_trigger(TChange::Add, items.clone());
        r#yield().await;
        self.with_trigger(TChange::Remove, items.clone());
        self.reason_to_arg(items)
    }

    pub(crate) async fn parse_pos(&self) -> Result<Option<OsString>, Error> {
        Ok(self.wait_for([TTarget::Pos]).await?.map(|arg| match arg {
            Arg::Named { .. } => unreachable!(),
            Arg::Pos { value, name: _ } => {
                self.consume(1);
                value.into_owned()
            }
        }))
    }

    pub(crate) async fn await_passing_check(
        &self,
        check: Rc<dyn Fn(&OsStr) -> bool>,
    ) -> Result<bool, Error> {
        let ok = self.wait_for([TTarget::Check(check)]).await?.is_some();
        if ok {
            self.consume(1);
        }
        Ok(ok)
    }

    /// Wake up on one of the literals, return it
    pub(crate) async fn parse_literal(
        &self,
        names: &[Lit<'static>],
    ) -> Result<Option<Lit<'static>>, Error> {
        let res = self
            .wait_for(names.iter().cloned().map(TTarget::Literal))
            .await?;

        Ok(match res {
            Some(Arg::Pos {
                value: _,
                name: Some(name),
            }) => Some(name.clone().into_owned()),
            _ => None,
        })
    }

    pub(crate) async fn parse_arg(
        &self,
        names: &[Name<'static>],
    ) -> Result<Option<OsString>, Error> {
        match self
            .wait_for(names.iter().cloned().map(TTarget::Arg))
            .await?
        {
            Some(arg) => self.parse_arg_consume(arg),
            None => Ok(None),
        }
    }

    pub(self) fn parse_arg_consume(&self, arg: Arg) -> Result<Option<OsString>, Error> {
        match arg {
            Arg::Pos { .. } => unreachable!(),
            Arg::Named { name, value: None } => {
                let cursor = self.cursor.get() + 1;
                let Some(next) = self.args.get(cursor) else {
                    return Err(Error::Problem(
                        cursor as u32 - 1,
                        Problem::WrongArgument {
                            name: name.into_owned(),
                            value: None,
                        },
                    ));
                };
                let pos = self.cursor.get() as u32;
                match lex_os_arg(next) {
                    Arg::Named { .. } => Err(Error::Problem(
                        pos,
                        Problem::WrongArgument {
                            name: name.into_owned(),
                            value: Some(next.clone()),
                        },
                    )),
                    Arg::Pos { value, name: _ } => {
                        self.consume(2);
                        if self.args.complete && cursor + 1 == self.args.len() {
                            let req = match value.to_str() {
                                Some(prefix) => CompleteReq::Literal {
                                    prefix: prefix.into(),
                                },
                                None => CompleteReq::Value(value.into_owned()),
                            };
                            return Err(Error::CompReq(req));
                        }
                        Ok(Some(value.clone().into_owned()))
                    }
                }
            }
            Arg::Named {
                name: _,
                value: Some((_adj, val)),
            } => {
                self.consume(1);
                Ok(Some(val.into_owned()))
            }
        }
    }

    /// Parse a flag by any one of the given names
    ///
    /// - `Ok(true)` when encounters a name
    /// - `Ok(false)` when it gets terminated by "no such item"
    /// - `Err(Error::Killed)` when it gets out-consumed by something else
    pub(crate) async fn parse_flag(&self, names: &[Name<'static>]) -> Result<bool, Error> {
        match self
            .wait_for(names.iter().cloned().map(TTarget::Flag))
            .await?
        {
            Some(arg) => self.parse_flag_consume(arg),
            None => Ok(false),
        }
    }

    /// Consume a present argument, advance
    #[inline(always)]
    pub(self) fn parse_flag_consume(&self, arg: Arg) -> Result<bool, Error> {
        match arg {
            Arg::Named { name, value } => match value {
                Some((adj, val)) => {
                    let problem = Problem::ExpectedFlag {
                        name: name.into_owned(),
                        adj,
                        value: val.into_owned(),
                    };
                    let pos = self.cursor.get() as u32;
                    Err(Error::Problem(pos, problem))
                }
                None => {
                    self.consume(1);
                    Ok(true)
                }
            },
            Arg::Pos { name: _, value: _ } => unreachable!(),
        }
    }

    /// All the `items` where killed as we parsed an item at current cursor position
    fn record_conflicts(&self, items: impl IntoIterator<Item = TTarget>) {
        // The idea for conflict tracking is to record that we could have consumed
        // a flag / literal, but instead consumed something else at a given cursor position
        //
        // We do this so when we encounter something we can't parse - we check if it was ever
        // possible to parse it before. This gives a position we parsed instead
        let pos = self.cursor.get() as u32;
        self.conflicts
            .borrow_mut()
            .extend(items.into_iter().filter_map(|t| t.into_conflict(pos)));
    }

    fn reason_to_arg(
        &self,
        items: impl IntoIterator<Item = TTarget> + Clone,
    ) -> Result<Option<Arg<'_>>, Error> {
        match &*self.wakeup_reason.borrow() {
            Reason::Arg(arg) => Ok(Some(arg.clone())),
            Reason::Kill(KillReason::Conflict) => {
                self.record_conflicts(items);
                Err(Error::Silent("Killed by conflict"))
            }
            Reason::Kill(KillReason::NoMatchingInput) => Ok(None),
            Reason::Kill(KillReason::TooShort)
            | Reason::Pass
            | Reason::Push
            | Reason::ChildProgress(_) => todo!(),

            Reason::Complete(complete) => Err(Error::CompReq(complete.clone())),
        }
    }
}
