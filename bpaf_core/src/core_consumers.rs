//! This module contains the basic consumers for the arguments
//!
//! They know how to interact with executor, deal with early exit when necessary, etc.
//! Anything else gets built on top of those consumers

pub(crate) type AnyCheck = Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>;
pub(crate) type AnyResult = Option<Box<dyn std::any::Any>>;
use crate::*;

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

    pub(crate) async fn parse_pos(&self) -> Result<Option<OsString>, Error> {
        self.with_trigger(TChange::Add, [TTarget::Pos]);
        r#yield().await;
        let res = self
            .try_to_parse_arg(None, |arg| self.parse_pos_consume(arg))
            .await;
        self.with_trigger(TChange::Remove, [TTarget::Pos]);
        if self.is_task_terminated() {
            let pos = self.cursor.get() as u32;
            self.conflicts.borrow_mut().push(Conflict::Pos { pos });
            Err(Error::OUTCONSUMED)
        } else {
            res
        }
    }

    pub(self) fn parse_pos_consume(&self, arg: &Arg) -> Result<Option<OsString>, Error> {
        match arg {
            Arg::Named { .. } => unreachable!(),
            Arg::Pos { value } => {
                self.consume(1);
                Ok(Some(value.clone().into_owned()))
            }
        }
    }

    pub(crate) async fn parse_any(&self, check: AnyCheck) -> Result<AnyResult, Error> {
        self.with_trigger(TChange::Add, [TTarget::Any]);
        r#yield().await;
        let res = loop {
            println!("Trying to run Any");
            let res = self
                .try_to_parse_arg(None, |_arg| self.parse_any_consume(&check))
                .await;

            if !matches!(res, Ok(None)) {
                break res;
            }
        };
        self.with_trigger(TChange::Remove, [TTarget::Any]);
        if self.is_task_terminated() {
            let pos = self.cursor.get() as u32;
            self.conflicts.borrow_mut().push(Conflict::Pos { pos });
            Err(Error::OUTCONSUMED)
        } else {
            res
        }
    }

    pub(self) fn parse_any_consume(&self, check: &AnyCheck) -> Result<AnyResult, Error> {
        let cursor = self.cursor.get();
        let res = check(&self.args[cursor]);
        if res.is_some() {
            self.consume(1);
        }
        Ok(res)
    }

    pub(crate) async fn parse_flag_and(
        &self,
        names: &[Name<'static>],
        populate: &dyn Fn(Ctx),
        parser: &dyn Visited,
    ) -> Result<bool, Error> {
        self.with_trigger(TChange::Add, names.iter().cloned().map(TTarget::Flag));
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |_arg| self.parse_nested(1, populate, parser))
            .await;
        self.with_trigger(TChange::Remove, names.iter().cloned().map(TTarget::Flag));
        if self.is_task_terminated() {
            self.record_conflict_with_names(names);
            Err(Error::OUTCONSUMED)
        } else {
            res
        }
    }

    /// Run a nested parser prefixed by a literal
    ///
    /// Returns true if parser executed and false otherwise
    pub(crate) async fn parse_literal_and(
        &self,
        names: &[Cow<'static, str>],
        populate: &dyn Fn(Ctx),
        parser: &dyn Visited,
    ) -> Result<bool, Error> {
        self.with_trigger(TChange::Add, names.iter().cloned().map(TTarget::Literal));
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |_arg| self.parse_nested(1, populate, parser))
            .await;
        self.with_trigger(TChange::Remove, names.iter().cloned().map(TTarget::Literal));
        if self.is_task_terminated() {
            Err(Error::OUTCONSUMED)
        } else {
            res
        }
    }

    fn parse_nested(
        &self,
        skip: u32,
        populate: &dyn Fn(Ctx),
        parser: &dyn Visited,
    ) -> Result<bool, Error> {
        self.consume(skip);
        let ctx = self.fork();
        ctx.cursor.update(|c| c + skip as usize);
        (populate)(ctx.clone());
        ctx.execute(parser)?;
        let consumed = ctx.cursor.get() - self.cursor.get() - 1;
        self.consume(consumed as u32);
        Ok(true)
    }

    pub(crate) async fn parse_arg(
        &self,
        names: &[Name<'static>],
    ) -> Result<Option<OsString>, Error> {
        self.with_trigger(TChange::Add, names.iter().cloned().map(TTarget::Arg));
        r#yield().await;
        let res = self
            .try_to_parse_arg(None, |arg| self.parse_arg_consume(arg))
            .await;
        self.with_trigger(TChange::Remove, names.iter().cloned().map(TTarget::Arg));
        if self.is_task_in_conflict() {
            self.record_conflict_with_names(names);
        }
        if self.is_task_terminated() {
            Err(Error::OUTCONSUMED)
        } else {
            res
        }
    }

    pub(self) fn parse_arg_consume(&self, arg: &Arg) -> Result<Option<OsString>, Error> {
        match arg {
            Arg::Pos { .. } => unreachable!(),
            Arg::Named { name, value: None } => {
                let cursor = self.cursor.get() + 1;
                let Some(next) = self.args.get(cursor) else {
                    return Err(Error::Problem(Problem::WrongArgument {
                        name: name.clone().into_owned(),
                        value: None,
                    }));
                };
                match lex_os_arg(next) {
                    Arg::Named { .. } => Err(Error::Problem(Problem::WrongArgument {
                        name: name.clone().into_owned(),
                        value: Some(next.clone()),
                    })),
                    Arg::Pos { value } => {
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
                Ok(Some(val.clone().into_owned()))
            }
        }
    }

    fn record_conflict_with_names(&self, names: &[Name<'static>]) {
        let pos = self.cursor.get() as u32;
        self.conflicts.borrow_mut().extend(
            names
                .iter()
                .cloned()
                .map(|name| Conflict::Named { pos, name }),
        );
    }

    /// Parse a flag by any one of the given names
    ///
    /// - `Ok(true)` when encounters a name
    /// - `Ok(false)` when it gets terminated by "no such item"
    /// - `Err(Error::Killed)` when it gets out-consumed by something else
    pub(crate) async fn parse_flag(&self, names: &[Name<'static>]) -> Result<bool, Error> {
        self.with_trigger(TChange::Add, names.iter().cloned().map(TTarget::Flag));
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |arg| self.parse_flag_consume(arg))
            .await;
        self.with_trigger(TChange::Remove, names.iter().cloned().map(TTarget::Flag));
        if self.is_task_in_conflict() {
            self.record_conflict_with_names(names);
        }
        if self.is_task_terminated() {
            Err(Error::OUTCONSUMED)
        } else {
            res
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
                        value: val.clone().into_owned(),
                    };
                    Err(Error::Problem(problem))
                }
                None => {
                    self.consume(1);
                    Ok(true)
                }
            },
            Arg::Pos { value: _ } => unreachable!(),
        }
    }

    /// Handle early exit conditions
    #[inline(always)]
    pub(self) async fn try_to_parse_arg<T>(
        &self,
        fallback: T,
        act: impl Fn(&Arg) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let res = {
            let reason = self.wakeup_reason.borrow();
            let arg = match &*reason {
                Reason::NoPass | Reason::Pass | Reason::ChildProgress(_) | Reason::Push => {
                    return Err(Error::Silent("Unexpected reason in try_to_parse_arg"));
                }
                Reason::Arg(arg) => arg,
                Reason::Complete(complete) => {
                    return Err(Error::CompReq(complete.clone()));
                }
            };
            let Some(arg_ref) = arg.as_ref() else {
                return Ok(fallback);
            };
            act(arg_ref)
        };
        r#yield().await;
        res
    }

    fn is_task_terminated(&self) -> bool {
        matches!(*self.wakeup_reason.borrow(), Reason::NoPass)
    }

    // conflict can happen in two different ways - task gets out-consumed (rarely)
    // or some other branch makes progress so current one gets terminated
    fn is_task_in_conflict(&self) -> bool {
        matches!(
            *self.wakeup_reason.borrow(),
            Reason::Arg(None) | Reason::NoPass
        )
    }
}
