use crate::named::Name;

use super::{split_param, Arg, Ctx, Error};
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

pub(crate) fn fork<T>() -> (Rc<ExitHandle<T>>, JoinHandle<T>) {
    let result = Rc::new(Cell::new(None));
    let exit = ExitHandle {
        waker: Cell::new(None),
        result: result.clone(),
    };
    let exit = Rc::new(exit);
    let join = JoinHandle {
        task: Rc::downgrade(&exit),
        result,
    };
    (exit, join)
}

impl<T> Drop for ExitHandle<T> {
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

pub(crate) struct ExitHandle<T> {
    waker: Cell<Option<Waker>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

pub(crate) struct JoinHandle<T> {
    task: Weak<ExitHandle<T>>,
    result: Rc<Cell<Option<Result<T, Error>>>>,
}

impl<T: std::fmt::Debug> ExitHandle<T> {
    pub(crate) fn exit_task(self, result: Result<T, Error>) {
        println!("Setting result to {result:?}");
        self.result.set(Some(result));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}
impl<T> Future for JoinHandle<T> {
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
    pub(crate) registered: bool,
}

impl<'ctx> Future for PositionalFut<'ctx> {
    type Output = Result<&'ctx str, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
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
    pub(crate) registered: bool,
}

impl Drop for NamedFut<'_> {
    fn drop(&mut self) {
        println!("Should no longer accept {:?}", self.name);
    }
}

impl Future for NamedFut<'_> {
    type Output = Result<Name<'static>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            self.ctx.named_wake(self.name, cx.waker().clone());
            return Poll::Pending;
        }

        let Some(front) = self.ctx.args.get(self.ctx.cur()) else {
            return Poll::Ready(Err(Error::Missing));
        };

        Poll::Ready(match split_param(front) {
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
