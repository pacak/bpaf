use crate::{error::MissingItem, named::Name, split::OsOrStr};

use super::{Arg, Ctx, Error, Id, Op};
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    sync::atomic::AtomicUsize,
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
                    .push_back(Op::ParentExited { id });
            }
        }
    }
}

pub(crate) type ErrorHandle = Rc<Cell<Option<Error>>>;

impl<T> ExitHandle<'_, T> {
    pub(crate) fn exit_task(self, result: Result<T, Error>) -> ErrorHandle {
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
        let id = self.current_id();
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
                Some(e) => e.combine_with(err),
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
    pub(crate) meta: &'static str,
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<Id>,
}
impl Drop for PositionalFut<'_> {
    fn drop(&mut self) {
        println!("dropped positional");
        if let Some(id) = self.task_id {
            self.ctx
                .shared
                .borrow_mut()
                .push_back(Op::RemovePositionalListener { id });
        }
    }
}

impl<'ctx> Future for PositionalFut<'ctx> {
    type Output = Result<OsOrStr<'ctx>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx.positional_wake(cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(match self.ctx.args.get(self.ctx.cur()) {
            Some(s) => {
                if s.is_named() {
                    Error::unexpected()
                } else {
                    self.ctx.advance(1);
                    Ok(s.to_owned())
                }
            }
            None => {
                if self.ctx.is_term() {
                    Err(Error::missing(MissingItem::Positional { meta: self.meta }))
                } else {
                    return Poll::Pending;
                }
            }
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
    pub(crate) task_id: Option<Id>,
}

pub(crate) struct ArgFut<'a> {
    pub(crate) name: &'a [Name<'static>],
    pub(crate) meta: &'static str,
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<Id>,
}

impl Drop for ArgFut<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.task_id {
            self.ctx.remove_named_listener(false, id, self.name);
        }
    }
}

impl Drop for FlagFut<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.task_id {
            self.ctx.remove_named_listener(true, id, self.name);
        }
    }
}

impl ArgFut<'_> {
    fn missing(&self) -> Error {
        Error::missing(MissingItem::Named {
            name: self.name.to_vec(),
            meta: Some(self.meta),
        })
    }
}
impl<'ctx> Future for ArgFut<'ctx> {
    type Output = Result<OsOrStr<'ctx>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx
                .add_named_wake(false, self.name, cx.waker().clone());
            return Poll::Pending;
        }
        if self.ctx.is_term() {
            return Poll::Ready(Err(self.missing()));
        }

        let front = self.ctx.front.borrow();
        let Some(Arg::Named { name, value }) = front.as_ref() else {
            return Poll::Pending;
        };
        if !self.name.contains(name) {
            return Poll::Pending;
        }
        if let Some(v) = value {
            self.ctx.advance(1);
            return Poll::Ready(Ok(v.to_owned()));
        }
        let Some(v) = self.ctx.args.get(self.ctx.cur() + 1) else {
            // TODO - this lacks a value
            return Poll::Ready(Err(self.missing()));
        };
        if v.is_named() {
            // TODO - this lacks a value
            Poll::Ready(Err(self.missing()))
        } else {
            self.ctx.advance(2);
            Poll::Ready(Ok(v.to_owned()))
        }
    }
}

impl FlagFut<'_> {
    fn missing(&self) -> Poll<Result<(), Error>> {
        Poll::Ready(Err(Error::missing(MissingItem::Named {
            name: self.name.to_vec(),
            meta: None,
        })))
    }
}
impl Future for FlagFut<'_> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx.add_named_wake(true, self.name, cx.waker().clone());
            return Poll::Pending;
        }

        let front = self.ctx.front.borrow();
        let Some(front) = front.as_ref() else {
            return self.missing();
        };

        match front {
            Arg::Named { name, value } => {
                if self.name.contains(name) {
                    if value.is_none() {
                        self.ctx.advance(1);
                        Poll::Ready(Ok(()))
                    } else {
                        todo!()
                    }
                } else {
                    self.missing()
                }
            }
            Arg::ShortSet { current, names } => todo!(),
            Arg::Positional { value } => todo!(),
        }
    }
}

pub(crate) struct FallbackFut<'ctx> {
    pub(crate) ctx: Ctx<'ctx>,
    pub(crate) task_id: Option<Id>,
}

impl Drop for FallbackFut<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.task_id {
            self.ctx.remove_fallback(id);
        }
    }
}

impl<'ctx> Future for FallbackFut<'ctx> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx.add_fallback(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(())
    }
}
