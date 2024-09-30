use crate::executor::{Ctx, Error};
use std::pin::pin;
use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

impl<'ctx> Ctx<'ctx> {
    pub(crate) fn fork<T>(&self) -> (Rc<ExitHandle<T>>, JoinHandle<T>) {
        let result = Rc::new(Cell::new(None));
        let exit = ExitHandle {
            waker: Cell::new(None),

            result: result.clone(),
        };
        let exit = Rc::new(exit);
        let join = JoinHandle {
            result,
            task: Rc::downgrade(&exit),
        };
        (exit, join)
    }

    pub(crate) fn spark<T>(&self) -> (Rc<SparkExitHandle<T>>, SparkHandle<T>) {
        let exit = Rc::new(SparkExitHandle {
            result: Cell::new(None),
        });
        let join = SparkHandle {
            task: Rc::downgrade(&exit),
        };
        (exit, join)
    }
}

pub(crate) struct ExitHandle<T> {
    /// Id of child task
    ///
    /// used to kill the task when join handle is dropped
    // pub(crate) id: Cell<Option<Id>>,
    /// Waker for parent task
    ///
    /// used to wake parent task when result is written
    waker: Cell<Option<Waker>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
    // If we are running multiple tasks in parallel on the same bit of input
    // only task(s) that consume longest amount should succeed even if those
    // with shorter consumption can produce results.
    // poisoned: Rc<Cell<bool>>,
}

pub struct JoinHandle<T> {
    task: Weak<ExitHandle<T>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

pub(crate) struct SparkExitHandle<T> {
    result: Cell<Option<Result<T, Error>>>,
}

/// Join vs Spark
pub struct SparkHandle<T> {
    task: Weak<SparkExitHandle<T>>,
}

impl<T> Future for SparkHandle<T> {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.task.upgrade() {
            Some(task) => match task.result.take() {
                Some(r) => Poll::Ready(r),
                None => Poll::Pending,
            },
            None => Poll::Ready(Err(Error::fail("poisoned", usize::MAX))),
        }
    }
}

impl<T> SparkExitHandle<T> {
    pub(crate) fn exit_task(&self, result: Result<T, Error>) {
        self.result.set(Some(result));
    }
}

impl<T> ExitHandle<T> {
    pub(crate) fn exit_task(&self, result: Result<T, Error>) {
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl<T> Drop for ExitHandle<T> {
    fn drop(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.result.take() {
            Some(r) => Poll::Ready(r),
            None => {
                if let Some(task) = self.task.upgrade() {
                    task.waker.set(Some(cx.waker().clone()));
                    Poll::Pending
                } else {
                    Poll::Ready(Err(Error::fail("killed", usize::MAX)))
                }
            }
        }
    }
}

pub(crate) struct AltFuture<T> {
    pub(crate) handles: Vec<JoinHandle<T>>,
}

impl<T> Future for AltFuture<T> {
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
