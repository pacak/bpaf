use crate::named::Name;

use super::{split_param, Arg, Ctx, Error, Id, Op};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::{pin, Pin},
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicU32, AtomicUsize},
        Arc, Mutex,
    },
    task::{Context, Poll, Wake, Waker},
    vec,
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

impl<T> Drop for ExitHandle<'_, T> {
    fn drop(&mut self) {
        // if waker is present - we must call it and mark the task as "killed"
        if let Some(waker) = self.waker.take() {
            let x = self.result.replace(Some(Err(Error::Killed)));
            assert!(x.is_none(), "Cloned ExitHandle?");
            println!("Dropped handle with waker!");
            waker.wake();
        };
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
                child.ctx.shared.borrow_mut().push_back(Op::KillTask { id });
            }
        }
    }
}

impl<T: std::fmt::Debug> ExitHandle<'_, T> {
    pub(crate) fn exit_task(self, result: Result<T, Error>) {
        println!("Setting result to {result:?}");
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
            Some(s) if s.starts_with('-') => Err(Error::Invalid),
            Some(s) => {
                self.ctx.advance(1);
                Ok(s.as_str())
            }
            None => Err(Error::Missing),
        })
    }
}

pub struct NamedFut<'a> {
    pub(crate) name: &'a [Name<'static>],
    pub(crate) ctx: Ctx<'a>,
    pub(crate) task_id: Option<Id>,
}

impl Drop for NamedFut<'_> {
    fn drop(&mut self) {
        println!("dropped {:?}", self.name);
        if let Some(id) = self.task_id {
            self.ctx
                .shared
                .borrow_mut()
                .push_back(Op::RemoveNamedListener {
                    names: self.name,
                    id,
                });
        }
    }
}

impl Future for NamedFut<'_> {
    type Output = Result<Name<'static>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.task_id.is_none() {
            self.task_id = self.ctx.current_task();
            self.ctx.named_wake(self.name, cx.waker().clone());
            return Poll::Pending;
        }

        self.task_id = None;
        let Some(front) = self.ctx.args.get(self.ctx.cur()) else {
            return Poll::Ready(Err(Error::Missing));
        };

        Poll::Ready(match split_param(front)? {
            Arg::Named { name, val } => {
                let r = self
                    .name
                    .iter()
                    .copied()
                    .find(|n| *n == name)
                    .ok_or(Error::Missing);
                if r.is_ok() {
                    self.ctx.advance(1);
                }

                r
            }
            Arg::ShortSet { .. } | Arg::Positional { .. } => Err(Error::Invalid),
        })
    }
}
