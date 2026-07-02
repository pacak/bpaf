use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::DerefMut,
};

use crate::*;

pub struct Vault<I, F> {
    pub(crate) inner: I,
    pub(crate) op: F,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
enum Kt {
    Ty(TypeId),
}

#[derive(Default)]
pub struct Storage {
    m: HashMap<Kt, Box<dyn Any>>,
}

pub trait Key: 'static {
    type Value;
}

impl Storage {
    pub fn get<K: Key>(&self) -> Option<&K::Value> {
        let key = Kt::Ty(TypeId::of::<K>());
        let v = self.m.get(&key)?;
        // technically we should do `Some(v.downcast_ref().unwrap()`
        // but we know that the type is equal
        v.downcast_ref()
    }

    pub fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        let key = Kt::Ty(TypeId::of::<K>());
        let v = self.m.get_mut(&key)?;
        // technically we should do `Some(v.downcast_ref().unwrap()`
        // but we know that the type is equal
        v.downcast_mut()
    }

    pub fn set<K: Key>(&mut self, value: K::Value) -> Option<K::Value> {
        let key = Kt::Ty(TypeId::of::<K>());
        let old = self.m.insert(key, Box::new(value))?;
        Some(*old.downcast().ok()?)
    }

    pub fn remove<K: Key>(&mut self) -> Option<K::Value> {
        let key = Kt::Ty(TypeId::of::<K>());
        let v = self.m.remove(&key)?;
        Some(*v.downcast().ok()?)
    }
}

impl<I: Parser, R: 'static, E: ToString + 'static, F> Parser for Vault<I, F>
where
    F: for<'v> Fn(&'v mut Storage, I::Output) -> Result<R, E>,
{
    type Output = R;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let t = self.inner.eval(ctx.clone()).await?;
        match (self.op)(ctx.shared.vault.borrow_mut().deref_mut(), t) {
            Ok(r) => Ok(r),
            Err(error) => Err(Error::Problem(
                ctx.leaf_cursor(),
                Problem::Parse {
                    value: ctx
                        .current_value
                        .borrow()
                        .as_ref()
                        .map(|v| v.to_string_lossy().into_owned()),
                    error: error.to_string(),
                },
            )),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

#[macro_export]
macro_rules! key {
    ($name:ident : $ty:ty) => {
        pub struct $name;
        impl $crate::Key for $name {
            type Value = $ty;
        }
    };
}
