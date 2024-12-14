use crate::{error::MissingItem, named::Name};

use super::{Arg, Ctx, Error, Id, Op};
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

impl<'ctx> Ctx<'ctx> {
    pub(crate) fn fork<T>(&self) -> (Rc<ExitHandle<'ctx, T>>, JoinHandle<'ctx, T>) {
        let result = Rc::new(Cell::new(None));
        let exit = ExitHandle {
            waker: Cell::new(None),
            result: result.clone(),
            id: Cell::new(None),
            ctx: self.clone(),
        };
        let exit = Rc::new(exit);
        let join = JoinHandle {
            task: Rc::downgrade(&exit),
            result,
        };
        (exit, join)
    }
}

pub(crate) struct ExitHandle<'a, T> {
    /// Id of child task
    pub(crate) id: Cell<Option<Id>>,
    /// Waker for parent task
    waker: Cell<Option<Waker>>,
    /// A way to pass the result from ExitHandle side to JoinHandle
    result: Rc<Cell<Option<Result<T, Error>>>>,
    ctx: Ctx<'a>,
}

pub(crate) struct JoinHandle<'a, T> {
    task: Weak<ExitHandle<'a, T>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
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

impl<T> ExitHandle<'_, T> {
    pub(crate) fn exit_task(self, result: Result<T, Error>) {
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
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

                Poll::Ready(self.result.take().expect("Task exit sets result"))
            }
        }
    }
}

pub struct PositionalFut<'a> {
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
    type Output = Result<&'ctx str, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx.positional_wake(cx.waker().clone());
            return Poll::Pending;
        }

        Poll::Ready(match self.ctx.args.get(self.ctx.cur()) {
            // TODO - check if we are after --
            Some(s) if s.starts_with('-') => Error::unexpected(),
            Some(s) => {
                self.ctx.advance(1);
                Ok(s.as_str())
            }
            None => Err(Error::missing(MissingItem::Positional { meta: self.meta })),
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
    fn missing(&self) -> Poll<Result<String, Error>> {
        Poll::Ready(Err(Error::missing(MissingItem::Named {
            name: self.name.to_vec(),
            meta: Some(self.meta),
        })))
    }
}
impl Future for ArgFut<'_> {
    type Output = Result<String, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx
                .add_named_wake(false, self.name, cx.waker().clone());
            return Poll::Pending;
        }

        let front = self.ctx.front.borrow();
        let Some(Arg::Named { name, value }) = front.as_ref() else {
            return self.missing();
        };
        if !self.name.contains(name) {
            return self.missing();
        }
        if let Some(v) = value {
            self.ctx.advance(1);
            return Poll::Ready(Ok((*v).to_owned()));
        }
        let Some(v) = self.ctx.args.get(self.ctx.cur() + 1) else {
            // TODO - this lacks a value
            return self.missing();
        };
        if v.starts_with('-') {
            // TODO - this lacks a value
            self.missing()
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

        match dbg!(front) {
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
