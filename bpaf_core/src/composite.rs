use crate::{BoxParser, Ctx, Error, Parser, VisitGroup, Visitor, ctx::Scope, tasks::Kind};

/// A product of a primary parser `P` and one more secondary parsers
///
/// Primary parser produces results, secondary parsers only used for effects
pub struct AndAlso<P> {
    pub(crate) inner: P,
    pub(crate) also: Vec<BoxParser<()>>,
}

impl<P> AndAlso<P> {
    pub fn and_also(mut self, other: impl Parser<Output = ()> + 'static) -> Self {
        self.also.push(other.into_box());
        self
    }
}

impl<P: Parser> Parser for AndAlso<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let inner = ctx.spawn(crate::tasks::Kind::Prod, &self.inner);
        let also: Vec<_> = self
            .also
            .iter()
            .map(|p| ctx.spawn(crate::tasks::Kind::Prod, p))
            .collect();

        ctx.wait_for_children().await;

        let mut err = None;
        let inner = inner.take().map_err(|e| e.append_to(&mut err));
        for h in also {
            // values are `()` so we care only about errors.
            if let Err(e) = h.take() {
                e.append_to(&mut err);
            }
        }

        err.map_or(inner, Err)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(VisitGroup::Prod);
        self.inner.visit(visitor);
        for p in &self.also {
            p.visit(visitor);
        }
        visitor.pop_group();
    }
}

/// A categorical sum of two or more parsers
///
/// This is a parser that is composed of two or more parsers. For `Sum<T>` to succeed
/// at least one member must succeed. If there are several succeeding variants - one that
/// consumes more input wins. If this is equal - one that goes earlier wins.
///
/// You can create it with [`construct!`](crate::construct)
///
/// TODO - a few dummy examples
pub struct Sum<T> {
    pub items: Vec<BoxParser<T>>,
}

impl<T: 'static> Sum<T> {
    pub fn or_else(&mut self, other: impl Parser<Output = T> + 'static) {
        self.items.push(other.into_box());
    }
}

impl<T: 'static> Parser for Sum<T> {
    type Output = T;
    fn or_else(mut self, other: impl Parser<Output = Self::Output> + 'static) -> Self {
        self.items.push(other.into_box());
        self
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(VisitGroup::Sum);
        for i in &self.items {
            i.visit(visitor);
        }
        visitor.pop_group();
    }

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<T, Error> {
        let id = ctx.shared.current_task.borrow().id;
        let mut scopes = Vec::with_capacity(self.items.len());
        let mut handles = Vec::with_capacity(self.items.len());
        for parser in &self.items {
            let (h, scope) = ctx.scoped_spawn(parser, Kind::Sum);
            scopes.push(scope);
            handles.push(h);
        }
        ctx.shared.sums.borrow_mut().insert(
            id,
            Scope {
                start: id,
                end: scopes.last().unwrap().end,
            },
        );
        ctx.wait_for_children().await; // give children a chance to start
        ctx.all_children_finish(scopes).await;
        ctx.shared.sums.borrow_mut().remove(&id);

        let mut acc = Error::Silent("Empty Sum?");

        let consumed = ctx.shared.current_task.borrow().consumed > 0;

        // prefer final error if present or the earliest result
        let mut val = None;
        for h in handles {
            match h.take() {
                Err(err) => acc = acc.combine(err, Kind::Sum),
                Ok(v) if consumed => return Ok(v),
                Ok(v) => val = val.or(Some(v)),
            }
        }

        // If the succeeding parser didn't consume a value - prefer to return an accumulated
        // error instead. We are dealing with a fallback-like case here. Such cases should
        // only handle "missing" style errors.
        //
        // Sample scenarios where this branch is taken:
        // - a sub-parser handles `--help`. It fails with `Error::Final` and we don't want
        //   to discard it if there's an alternative branch that succeeds without consuming
        //   anything: input that leads to the sub-parser is still there and needs to be
        //   consumed.
        // - in one branch an argument parser fails to consume the trigger due to missing
        //   value, a different parser in a concurrent branch succeeds without consuming
        //   anything. We want the error from the argument parser, otherwise the whole parser
        //   will fail to make progress and we'll have to produce an error explaining the
        //   unparsed trigger part of the argument.
        // - any sort of autocomplete
        if !consumed
            && matches!(
                &acc,
                Error::Final(_) | Error::Problem(_, _) | Error::CompValue(_) | Error::CompReply(_)
            )
        {
            Err(acc)
        } else {
            val.ok_or(acc)
        }
    }
}
