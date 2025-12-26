//! This module contains the basic consumers for the arguments
//!
//! They know how to interact with executor, deal with early exit when necessary, etc.
//! Anything else gets built on top of those consumers
//!
//! only bits here know about consuming, cursor, and events.
//! Everything here is either fixed type or (in rare cases &dyn...)

pub(crate) type AnyCheck = Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>;
pub(crate) type AnyResult = Option<Box<dyn std::any::Any>>;
use crate::*;

fn to_conflict(t: TTarget, pos: u32) -> Option<Conflict> {
    match t {
        TTarget::Arg(name) | TTarget::Flag(name) => Some(Conflict::Named { pos, name }),
        TTarget::Pos => Some(Conflict::Pos { pos }),
        TTarget::Any => None,
        TTarget::Literal(name) => Some(Conflict::Lit { pos, name }),
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

    async fn wait_for(&self, items: impl IntoIterator<Item = TTarget> + Clone) {
        self.with_trigger(TChange::Add, items.clone());
        r#yield().await;
        self.with_trigger(TChange::Remove, items.clone());

        // The idea for conflict tracking is to record that we could have consumed
        // a flag / literal, but instead consumed something else at a given cursor position
        //
        // We do this so when we encounter something we can't parse - we check if it was ever
        // possible to parse it before. This gives a position we parsed instead
        if matches!(&*self.wakeup_reason.borrow(), Reason::Arg(None)) {
            let pos = self.cursor.get() as u32;
            self.conflicts
                .borrow_mut()
                .extend(items.into_iter().filter_map(|t| to_conflict(t, pos)));
        }
    }

    pub(crate) async fn parse_pos(&self) -> Result<Option<OsString>, Error> {
        self.wait_for([TTarget::Pos]).await;
        Ok(self.arg_to_parse()?.map(|arg| match arg {
            Arg::Named { .. } => unreachable!(),
            Arg::Pos { value } => {
                self.consume(1);
                value.into_owned()
            }
        }))
    }

    pub(crate) async fn parse_any(&self, check: AnyCheck) -> Result<AnyResult, Error> {
        self.with_trigger(TChange::Add, [TTarget::Any]);
        r#yield().await;
        let res = loop {
            let res = self.try_to_parse_arg(None, |_arg| self.parse_any_consume(&check));
            if !matches!(res, Ok(None)) {
                break res;
            } else {
                self.pass.set(true);
                r#yield().await;
            }
        };
        self.with_trigger(TChange::Remove, [TTarget::Any]);
        res
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
    ) -> Result<Option<u32>, Error> {
        self.wait_for(names.iter().cloned().map(TTarget::Flag))
            .await;
        if self.arg_to_parse()?.is_some() {
            self.parse_nested(1, populate, parser)
        } else {
            Ok(None)
        }
    }

    /// Wake up on one of the literals, return it
    pub(crate) async fn parse_literal(
        &self,
        names: &[Cow<'static, str>],
    ) -> Result<Option<String>, Error> {
        self.wait_for(names.iter().cloned().map(TTarget::Literal))
            .await;
        Ok(match self.arg_to_parse()? {
            Some(Arg::Pos { value }) => Some(value.to_str().unwrap().to_owned()),
            _ => None,
        })
    }

    fn parse_nested(
        &self,
        skip: u32,
        populate: &dyn Fn(Ctx),
        parser: &dyn Visited,
    ) -> Result<Option<u32>, Error> {
        self.consume(skip);
        let ctx = self.fork(None);
        ctx.cursor.update(|c| c + skip as usize);
        let to_parse = ctx.args.len() - ctx.cursor.get();
        (populate)(ctx.clone());
        ctx.execute(parser, None)?;
        let consumed = ctx.cursor.get() - self.cursor.get() - 1;
        self.consume(consumed as u32);
        Ok(Some(to_parse as u32))
    }

    pub(crate) async fn parse_arg(
        &self,
        names: &[Name<'static>],
    ) -> Result<Option<OsString>, Error> {
        self.wait_for(names.iter().cloned().map(TTarget::Arg)).await;
        match self.arg_to_parse()? {
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
        self.wait_for(names.iter().cloned().map(TTarget::Flag))
            .await;
        match self.arg_to_parse()? {
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
            Arg::Pos { value: _ } => unreachable!(),
        }
    }

    /// Handle early exit conditions
    #[inline(always)]
    pub(self) fn try_to_parse_arg<T>(
        &self,
        fallback: T,
        act: impl Fn(&Arg) -> Result<T, Error>,
    ) -> Result<T, Error> {
        match &*self.wakeup_reason.borrow_mut() {
            Reason::NoPass | Reason::Pass | Reason::ChildProgress(_) | Reason::Push => {
                Err(Error::Silent("Unexpected reason in try_to_parse_arg"))
            }
            Reason::Arg(None) => Ok(fallback),
            Reason::Arg(Some(arg)) => act(arg),
            Reason::Complete(complete) => Err(Error::CompReq(complete.clone())),
        }
    }

    fn arg_to_parse(&self) -> Result<Option<Arg<'_>>, Error> {
        match &*self.wakeup_reason.borrow_mut() {
            Reason::NoPass | Reason::Pass | Reason::ChildProgress(_) | Reason::Push => {
                Err(Error::Silent("Unexpected reason in arg_to_parse"))
            }
            Reason::Arg(arg) => Ok(arg.clone()),
            Reason::Complete(complete) => Err(Error::CompReq(complete.clone())),
        }
    }
}
