use crate::executor::{Ctx, Error, Id, Op};
use std::pin::pin;
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

use super::PoisonHandle;

impl<'ctx> Ctx<'ctx> {
    pub(crate) fn fork<T>(&self) -> (Rc<ExitHandle<'ctx, T>>, JoinHandle<'ctx, T>) {
        let result = Rc::new(Cell::new(None));
        let poisoned = Rc::new(Cell::new(false));
        let exit = ExitHandle {
            waker: Cell::new(None),

            id: Cell::new(None),
            ctx: self.clone(),
            result: result.clone(),
            poisoned: poisoned.clone(),
        };
        let exit = Rc::new(exit);
        let join = JoinHandle {
            task: Rc::downgrade(&exit),
            result,
            poisoned,
        };
        (exit, join)
    }
}

pub(crate) struct ExitHandle<'a, T> {
    /// Id of child task
    pub(crate) id: Cell<Option<Id>>,
    /// Waker for parent task
    waker: Cell<Option<Waker>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
    /// If we are running multiple tasks in parallel on the same bit of input
    /// only task(s) that consume longest amount should succeed even if those
    /// with shorter consumption can produce results.
    poisoned: Rc<Cell<bool>>,

    /// dropping join handle needs to terminate all the children?
    /// TODO - we also track this in the executor
    ctx: Ctx<'a>,
}

pub struct JoinHandle<'a, T> {
    task: Weak<ExitHandle<'a, T>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
    /// See ExitHandle::poisoned
    poisoned: Rc<Cell<bool>>,
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

impl<T> ExitHandle<'_, T> {
    pub(crate) fn exit_task(&self, result: Result<T, Error>) -> PoisonHandle {
        self.result.set(Some(result));

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
        self.poisoned.clone()
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
                let res = self
                    .result
                    .take()
                    .expect("Child task exited without setting either result or success");

                Poll::Ready(if self.poisoned.get() {
                    Err(Error::fail("poisoned", usize::MAX))
                } else {
                    res
                })
            }
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
