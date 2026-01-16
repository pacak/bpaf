use crate::*;

pub struct Count<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Count<T> {
    type Output = usize;
    async fn run(&self, ctx: crate::Ctx) -> Result<usize, Error> {
        let many = Many {
            inner: self.inner.clone(),
        };
        Ok(many.run(ctx).await?.len())
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

pub struct Last<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Last<T> {
    type Output = T;

    async fn run(&self, ctx: crate::Ctx) -> Result<Self::Output, Error> {
        let start = ctx.next_free.get();
        let mut prev = None;
        loop {
            let this = try_parse(start, self.inner.clone(), ctx.clone()).await;
            match (prev, this) {
                // keep consuming as long as there are new items
                (_, (true, Ok(v))) => prev = Some(v),
                (None, (_, Err(e))) => return Err(e),
                (None, (false, Ok(v))) => return Ok(v),

                (Some(v), (_, Err(_))) => return Ok(v),
                (Some(v), (false, _)) => return Ok(v),
            }
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

pub struct Many<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Many<T> {
    type Output = Vec<T>;

    fn run(&self, ctx: crate::Ctx) -> impl Future<Output = Result<Vec<T>, Error>> {
        parse_many(self.inner.clone(), ctx, 0, usize::MAX)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

async fn try_parse<T: 'static>(
    start: u32,
    parser: RcParser<T>,
    ctx: Ctx,
) -> (bool, Result<T, Error>) {
    let before = ctx.current_task.borrow().consumed;
    ctx.next_free.set(start);
    let (h, pair) = ctx.spawn_with_early_exit(parser);
    r#yield().await;
    ctx.remove_early_exit(pair);
    let after = ctx.current_task.borrow().consumed;
    (before < after, h.take())
}

async fn parse_many<T: 'static>(
    parser: RcParser<T>,
    ctx: crate::Ctx,
    min: usize,
    max: usize,
) -> Result<Vec<T>, Error> {
    let mut res = Vec::new();
    let start = ctx.next_free.get();
    let mut consumed_before = 0;
    while matches!(&*ctx.wakeup_reason.borrow(), Reason::Pass | Reason::Push) {
        ctx.next_free.set(start);
        let (h, pair) = ctx.spawn_with_early_exit(parser.clone());

        r#yield().await;
        ctx.remove_early_exit(pair);

        let consumed_after = ctx.current_task.borrow().consumed;
        let advanced = consumed_after > consumed_before;
        consumed_before = consumed_after;

        let val = h.take();

        match (advanced, val) {
            (true, Ok(v)) => res.push(v),
            (true, Err(e)) => return Err(e),
            (false, Ok(v)) => {
                if res.is_empty() {
                    res.push(v);
                }
                break;
            }
            (false, Err(e)) if e.can_catch() && res.len() >= min => break,
            (false, Err(e)) => return Err(e),
        }
        if res.len() >= max {
            break;
        }
    }
    Ok(res)
}

pub struct Many1<T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) message: &'static str,
}
impl<T: 'static> Parser for Many1<T> {
    type Output = Vec<T>;
    async fn run(&self, ctx: crate::Ctx) -> Result<Vec<T>, Error> {
        let res = parse_many(self.inner.clone(), ctx, 0, usize::MAX).await?;
        if res.is_empty() {
            Err(Error::Problem(u32::MAX, Problem::Static(self.message)))
        } else {
            Ok(res)
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}
