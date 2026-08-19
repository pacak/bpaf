use crate::{BoxParser, Ctx, Error, Parser, VisitGroup, Visitor};

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
