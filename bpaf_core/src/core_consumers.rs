//! This module contains the basic consumers for the arguments
//!
//! They know how to interact with executor, deal with early exit when necessary, etc.
//! Anything else gets built on top of those consumers

pub(crate) type AnyCheck = Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>;
pub(crate) type AnyResult = Option<Box<dyn std::any::Any>>;
use crate::*;

impl RawCtx {
    #[inline(never)]
    pub(crate) fn add_names(
        &self,
        names: &[Name<'static>],
        selector: NamedPeckingOrderSelector,
    ) -> Result<(), Error> {
        let (parent, id) = self.task_parent_and_id();
        let (mut short, mut long) = RefMut::map_split(self.triggers.borrow_mut(), selector);
        for name in names {
            match name {
                Name::Short(c) => short.entry(*c).or_default().insert(parent, id),
                Name::Long(s) => long.entry(s.clone()).or_default().insert(parent, id),
            }
        }
        Ok(())
        //
    }
    pub(crate) fn add_literals(&self, names: &[Cow<'static, str>]) {
        let (parent, id) = self.task_parent_and_id();
        let mut triggers = self.triggers.borrow_mut();
        for name in names {
            triggers
                .literal
                .entry(name.clone())
                .or_default()
                .insert(parent, id);
        }
    }
    pub(crate) fn remove_literals(&self, names: &[Cow<'static, str>]) {
        use std::collections::hash_map::Entry;
        let (parent, id) = self.task_parent_and_id();
        let mut triggers = self.triggers.borrow_mut();
        for name in names {
            if let Entry::Occupied(mut e) = triggers.literal.entry(name.clone()) {
                if e.get_mut().remove(parent, id) {
                    e.remove();
                }
            } else {
                todo!();
            }
        }
    }

    #[inline(never)]
    pub(crate) fn remove_names(
        &self,
        names: &[Name<'static>],
        selector: NamedPeckingOrderSelector,
    ) -> Result<(), Error> {
        let (parent, id) = self.task_parent_and_id();
        let (mut short, mut long) = RefMut::map_split(self.triggers.borrow_mut(), selector);

        use std::collections::hash_map::Entry;
        for name in names {
            match name {
                Name::Short(c) => {
                    if let Entry::Occupied(mut e) = short.entry(*c) {
                        if e.get_mut().remove(parent, id) {
                            e.remove();
                        }
                    } else if cfg!(debug_assertions) {
                        panic!("Tried to remove missing {name:?}");
                    }
                }
                Name::Long(s) => {
                    if let Entry::Occupied(mut e) = long.entry(s.clone()) {
                        if e.get_mut().remove(parent, id) {
                            e.remove();
                        }
                    } else if cfg!(debug_assertions) {
                        panic!("Tried to remove missing {name:?}");
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn consume(&self, cnt: u32) {
        self.current_task.borrow_mut().consumed += cnt;
    }

    // pub(crate) async fn parse_any(&self, check: check: Box<dyn Fn(&OsStr) -> Option<Box<dyn std::any::Any>>>) -> Result<Option<Box<dyn std::any::Any>>, Error> {
    //
    // }

    fn add_to_po(&self, lens: fn(&mut Triggers) -> &mut PeckingOrder) {
        let (parent, id) = self.task_parent_and_id();
        lens(&mut self.triggers.borrow_mut()).insert(parent, id);
    }
    fn remove_from_po(&self, lens: fn(&mut Triggers) -> &mut PeckingOrder) {
        let (parent, id) = self.task_parent_and_id();
        lens(&mut self.triggers.borrow_mut()).remove(parent, id);
    }

    pub(crate) async fn parse_pos(&self) -> Result<Option<OsString>, Error> {
        self.add_to_po(|triggers| &mut triggers.pos);
        r#yield().await;
        let res = self
            .try_to_parse_arg(None, |arg| self.parse_pos_consume(arg))
            .await;
        self.remove_from_po(|triggers| &mut triggers.pos);
        self.managed_to_survive()?;
        res
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
        self.add_to_po(|triggers| &mut triggers.any);
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
        self.remove_from_po(|triggers| &mut triggers.any);
        self.managed_to_survive()?;
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
    ) -> Result<bool, Error> {
        self.add_names(names, view_reactor_flags)?;
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |_arg| self.parse_nested(1, populate))
            .await;
        self.remove_names(names, view_reactor_flags)?;
        self.managed_to_survive()?;
        res
    }

    pub(crate) async fn parse_literal_and(
        &self,
        names: &[Cow<'static, str>],
        populate: &dyn Fn(Ctx),
    ) -> Result<bool, Error> {
        self.add_literals(names);
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |_arg| self.parse_nested(1, populate))
            .await;
        self.remove_literals(names);
        self.managed_to_survive()?;
        res
    }

    fn parse_nested(&self, skip: u32, populate: &dyn Fn(Ctx)) -> Result<bool, Error> {
        self.consume(skip);
        let ctx = self.fork();
        ctx.cursor.update(|c| c + skip as usize);
        (populate)(ctx.clone());
        ctx.execute()?;
        let consumed = ctx.cursor.get() - self.cursor.get() - 1;
        self.consume(consumed as u32);
        Ok(true)
    }

    pub(crate) async fn parse_arg(
        &self,
        names: &[Name<'static>],
    ) -> Result<Option<OsString>, Error> {
        self.add_names(names, view_reactor_args)?;
        r#yield().await;
        let res = self
            .try_to_parse_arg(None, |arg| self.parse_arg_consume(arg))
            .await;
        self.remove_names(names, view_reactor_args)?;
        self.managed_to_survive()?;
        res
    }

    pub(self) fn parse_arg_consume(&self, arg: &Arg) -> Result<Option<OsString>, Error> {
        match arg {
            Arg::Pos { .. } => unreachable!(),
            Arg::Named { name, value: None } => {
                let cursor = self.cursor.get() + 1;
                let Some(arg) = self.args.get(cursor) else {
                    todo!("{name:?} expects a value");
                };
                match lex_os_arg(arg) {
                    Arg::Named { .. } => {
                        todo!("{name:?} got {arg:?}, try {name:?}={arg:?}")
                    }
                    Arg::Pos { value } => {
                        self.consume(2);
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

    /// Parse a flag by any one of the given names
    ///
    /// - `Ok(true)` when encounters a name
    /// - `Ok(false)` when it gets terminated by "no such item"
    /// - `Err(Error::Killed)` when it gets out-consumed by something else
    pub(crate) async fn parse_flag(&self, names: &[Name<'static>]) -> Result<bool, Error> {
        self.add_names(names, view_reactor_flags)?;
        r#yield().await;
        let res = self
            .try_to_parse_arg(false, |arg| self.parse_flag_consume(arg))
            .await;
        self.remove_names(names, view_reactor_flags)?;
        self.managed_to_survive()?;
        res
    }

    /// Consume a present argument, advance
    #[inline(always)]
    pub(self) fn parse_flag_consume(&self, arg: &Arg) -> Result<bool, Error> {
        match arg {
            Arg::Named { name, value } => match value {
                Some(val) => {
                    todo!("Expected flag {name:?}, got value {val:?}");
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
                Reason::NotConsumedEnough | Reason::Pass | Reason::ChildProgress(_) => {
                    return Err(Error::Killed);
                }
                Reason::Arg(arg) => arg,
                Reason::Complete(complete) => {
                    return Err(Error::Complete(Vec1::new(complete.clone())));
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

    fn managed_to_survive(&self) -> Result<(), Error> {
        if matches!(*self.wakeup_reason.borrow(), Reason::NotConsumedEnough) {
            Err(Error::Killed)
        } else {
            Ok(())
        }
    }
}
