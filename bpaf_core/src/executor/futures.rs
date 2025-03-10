use crate::executor::{Arg, BranchId, Ctx, Error, Id, Op};
use crate::{
    error::{Message, Metavar, MissingItem},
    named::Name,
    split::OsOrStr,
};
use std::pin::pin;
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

impl<'ctx> Ctx<'ctx> {
    pub(crate) fn fork<T>(&self) -> (Rc<ExitHandle<'ctx, T>>, JoinHandle<'ctx, T>) {
        let success = Rc::new(Cell::new(None));
        let failure = Rc::new(Cell::new(None));
        let exit = ExitHandle {
            waker: Cell::new(None),

            id: Cell::new(None),
            ctx: self.clone(),
            failure: failure.clone(),
            success: success.clone(),
        };
        let exit = Rc::new(exit);
        let join = JoinHandle {
            task: Rc::downgrade(&exit),
            success,
            failure,
        };
        (exit, join)
    }
}

pub(crate) struct ExitHandle<'a, T> {
    /// Id of child task
    pub(crate) id: Cell<Option<Id>>,
    /// Waker for parent task
    waker: Cell<Option<Waker>>,

    /// Handle can hold both success and a failure if parser succeeded
    /// but was terminated due better (gredier) parser in a parallel branch
    /// Failre takes priority
    failure: Rc<Cell<Option<Error>>>,
    success: Rc<Cell<Option<T>>>,

    ctx: Ctx<'a>,
}

pub struct JoinHandle<'a, T> {
    task: Weak<ExitHandle<'a, T>>,
    failure: Rc<Cell<Option<Error>>>,
    success: Rc<Cell<Option<T>>>,
}

impl<T> Drop for JoinHandle<'_, T> {
    fn drop(&mut self) {
        if let Some(child) = self.task.upgrade() {
            if let Some(id) = child.id.take() {
                child
                    .ctx
                    .shared
                    .borrow_mut()
                    .push_back(Op::RemoveTask { id });
            }
        }
    }
}

pub(crate) type ErrorHandle = Rc<Cell<Option<Error>>>;

impl<T> ExitHandle<'_, T> {
    pub(crate) fn exit_task(&self, result: Result<T, Error>) -> ErrorHandle {
        match result {
            Ok(ok) => self.success.set(Some(ok)),
            Err(err) => self.failure.set(Some(err)),
        }

        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
        self.failure.clone()
    }
}
impl<T> Future for JoinHandle<'_, T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.task.upgrade() {
            Some(task) => {
                task.waker.set(Some(cx.waker().clone()));
                Poll::Pending
            }
            None => {
                println!("Getting result out!");
                if let Some(err) = self.failure.take() {
                    Poll::Ready(Err(err))
                } else if let Some(ok) = self.success.take() {
                    Poll::Ready(Ok(ok))
                } else {
                    unreachable!("Child task exited without setting either result or success");
                }
            }
        }
    }
}

impl<'a> Ctx<'a> {
    pub fn early_exit(self, cnt: u32) -> EarlyExitFut<'a> {
        let (_branch, id) = self.current_id();
        EarlyExitFut {
            id,
            ctx: self,
            cnt,
            registered: false,
            early_err: None,
        }
    }
}

pub struct EarlyExitFut<'a> {
    /// Look for failures for tasks with this parent
    id: Id,
    /// There are this many children left
    cnt: u32,
    ctx: Ctx<'a>,
    registered: bool,
    early_err: Option<Error>,
}

impl Future for EarlyExitFut<'_> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx.add_children_exit_listener(self.id);
            return Poll::Pending;
        }

        if let Some(err) = self.ctx.child_exit.take() {
            self.early_err = Some(match self.early_err.take() {
                Some(e) => {
                    assert!(e.offset <= err.offset);
                    err.combine_with(e)
                }
                None => err,
            });
        }

        self.cnt -= 1;
        if self.cnt == 0 {
            Poll::Ready(self.early_err.take().map_or(Ok(()), Err))
        } else {
            Poll::Pending
        }
    }
}

impl Drop for EarlyExitFut<'_> {
    fn drop(&mut self) {
        self.ctx.remove_children_exit_listener(self.id);
    }
}

pub(crate) struct PositionalFut<'a> {
    pub(crate) meta: Metavar,
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<(BranchId, Id)>,
}
impl Drop for PositionalFut<'_> {
    fn drop(&mut self) {
        println!("dropped positional");
        if let Some((branch, id)) = self.task_id {
            self.ctx
                .shared
                .borrow_mut()
                .push_back(Op::RemovePositionalListener { branch, id });
        }
    }
}

impl<'ctx> Future for PositionalFut<'ctx> {
    type Output = Result<OsOrStr<'ctx>, Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            let (branch, id) = self.ctx.current_task();
            self.task_id = Some((branch, id));
            self.ctx.positional_wake(branch, id);
            return Poll::Pending;
        }

        Poll::Ready(if self.ctx.is_term() {
            Err(Error::missing(
                MissingItem::Positional { meta: self.meta },
                self.ctx.cur(),
            ))
        } else {
            let ix = self.ctx.cur();
            self.ctx.advance(1);
            Ok(self.ctx.args[ix].as_ref())
        })
    }
}

// For named items we must be able to consume
// - flags: -a, --alice
// - arguments: -aval, -a val, -a=val, --alice=val --alice val
//
// Additionally flags can be mashed together: -abc is -a -b -c

pub(crate) struct FlagFut<'a> {
    pub(crate) name: &'a [Name<'static>],
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<(BranchId, Id)>,
}

pub(crate) struct ArgFut<'a> {
    pub(crate) name: &'a [Name<'static>],
    pub(crate) meta: Metavar,
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<(BranchId, Id)>,
}

impl Drop for ArgFut<'_> {
    fn drop(&mut self) {
        if let Some((branch, id)) = self.task_id {
            self.ctx.remove_named_listener(false, branch, id, self.name);
        }
    }
}

impl Drop for FlagFut<'_> {
    fn drop(&mut self) {
        if let Some((branch, id)) = self.task_id {
            self.ctx.remove_named_listener(true, branch, id, self.name);
        }
    }
}

impl ArgFut<'_> {
    fn missing(&self) -> Error {
        Error::missing(
            MissingItem::Named {
                name: self.name.to_vec(),
                meta: Some(self.meta),
            },
            self.ctx.cur(),
        )
    }
}
impl<'ctx> Future for ArgFut<'ctx> {
    type Output = Result<OsOrStr<'ctx>, Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // registration
        if self.task_id.is_none() {
            let (branch, id) = self.ctx.current_task();
            self.task_id = Some((branch, id));
            self.ctx.add_named_wake(false, self.name, branch, id);
            return Poll::Pending;
        }
        // on term - exit immediately
        if self.ctx.is_term() {
            return Poll::Ready(Err(self.missing()));
        }

        let front = self.ctx.front_value.borrow();
        if let Some(v) = front.as_ref() {
            self.ctx.advance(1);
            return Poll::Ready(Ok(v.clone()));
        }
        // TODO Pass the name here too?
        let name = || {
            let front = self
                .ctx
                .args
                .get(self.ctx.cur())
                .expect("Should be present");
            let m = Default::default();

            match super::split_param(front, &m, &m).unwrap() {
                Arg::Named { name, value: None } => name.to_owned(),
                _ => panic!("Expected to see a name "),
            }
        };
        Poll::Ready(match self.ctx.args.get(self.ctx.cur() + 1) {
            None => Err(Error::new(
                Message::ArgNeedsValue {
                    name: name(),
                    meta: self.meta,
                },
                self.ctx.cur(),
            )),
            Some(v) => {
                let val = v.to_owned();
                if val.is_named() {
                    Err(Error::new(
                        Message::ArgNeedsValueGotNamed {
                            name: name(),
                            meta: self.meta,
                            val,
                        },
                        self.ctx.cur(),
                    ))
                } else {
                    self.ctx.advance(2);
                    Ok(val)
                }
            }
        })
    }
}

impl FlagFut<'_> {
    fn missing(&self) -> Poll<Result<(), Error>> {
        Poll::Ready(Err(Error::missing(
            MissingItem::Named {
                name: self.name.to_vec(),
                meta: None,
            },
            self.ctx.cur(),
        )))
    }
}
impl Future for FlagFut<'_> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            let (branch, id) = self.ctx.current_task();
            self.task_id = Some((branch, id));
            self.ctx.add_named_wake(true, self.name, branch, id);
            return Poll::Pending;
        }

        if self.ctx.is_term() {
            self.missing()
        } else {
            assert!(self.ctx.front_value.borrow().is_none());
            self.ctx.advance(1);

            Poll::Ready(Ok(()))
        }
    }
}

/// Match literal positional value - doesn't start with - or --
pub(crate) struct LiteralFut<'a> {
    /// Name match values without prefix!
    /// TODO - do I really want to keep Name here?
    pub(crate) values: &'a [Name<'static>],
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<(BranchId, Id)>,
}

impl LiteralFut<'_> {
    fn missing(&self) -> Error {
        Error::missing(
            MissingItem::Command {
                name: self.values.to_vec(),
            },
            self.ctx.cur(),
        )
    }
}

impl Future for LiteralFut<'_> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            let (branch, id) = self.ctx.current_task();
            self.task_id = Some((branch, id));
            self.ctx.add_literal_wake(self.values, branch, id);
            return Poll::Pending;
        }

        if self.ctx.is_term() {
            return Poll::Ready(Err(self.missing()));
        }
        self.ctx.advance(1);
        Poll::Ready(Ok(()))
    }
}

impl Drop for LiteralFut<'_> {
    fn drop(&mut self) {
        if let Some((branch, id)) = self.task_id.take() {
            self.ctx.remove_literal(self.values, branch, id);
        }
    }
}

pub(crate) struct AnyFut<'a, T> {
    pub(crate) check: &'a dyn Fn(OsOrStr) -> Option<T>,
    pub(crate) ctx: Ctx<'a>,
    pub(crate) metavar: Metavar,
    pub(crate) task_id: Option<(BranchId, Id)>,
}

impl<T> Future for AnyFut<'_, T> {
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            let (branch, id) = self.ctx.current_task();
            self.task_id = Some((branch, id));
            self.ctx.add_any_wake(branch, id);
            return Poll::Pending;
        }

        if self.ctx.is_term() {
            let missing = MissingItem::Positional { meta: self.metavar };
            return Poll::Ready(Err(Error::missing(missing, self.ctx.cur())));
        }
        let Some(arg) = self.ctx.args.get(self.ctx.cur()) else {
            debug_assert!(false, "This should not be reachable");
            return Poll::Pending;
        };
        match (self.check)(arg.as_ref()) {
            Some(t) => {
                self.ctx.advance(1);
                Poll::Ready(Ok(t))
            }
            None => Poll::Pending,
        }
    }
}

impl<T> Drop for AnyFut<'_, T> {
    fn drop(&mut self) {
        if let Some((branch, id)) = self.task_id.take() {
            self.ctx.remove_any(branch, id);
        }
    }
}

pub(crate) struct AltFuture<'a, T> {
    pub(crate) handles: Vec<JoinHandle<'a, T>>,
}

impl<T> Future for AltFuture<'_, T> {
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(!self.as_ref().handles.is_empty());
        for (ix, mut h) in self.as_mut().handles.iter_mut().enumerate() {
            if let Poll::Ready(r) = pin!(h).poll(cx) {
                // This future can be called multiple times, as long as there
                // are handles to be consumed
                self.handles.remove(ix);
                return Poll::Ready(r);
            }
        }
        Poll::Pending
    }
}
