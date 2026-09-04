use crate::{Ctx, Error, Parser, Visitor};

pub struct Cargo<P> {
    name: &'static str,
    inner: P,
}

pub fn cargo_helper<P>(name: &'static str, inner: P) -> Cargo<P> {
    Cargo { name, inner }
}

impl<P: Parser> Parser for Cargo<P> {
    type Output = P::Output;

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<Self::Output, Error>> {
        if ctx.shared.args.get(0).is_some_and(|v| v == self.name) {
            ctx.cursor().update(|c| if c == 0 { 1 } else { c });
        }
        self.inner.eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor)
    }
}
