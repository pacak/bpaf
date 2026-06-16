use std::ops::RangeBounds;

use crate::{
    adapters::{Optionality, optional},
    error::MissingItem,
    *,
};

pub struct Count<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Count<T> {
    type Output = usize;
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<usize, Error> {
        let mut cnt = 0;

        let start = ctx.next_free.get();
        loop {
            ctx.next_free.set(start);
            match optional(ctx.clone(), &self.inner).await {
                Optionality::Parsed(_) => cnt += 1,
                Optionality::Summoned(_) => return Ok(cnt.max(1)),
                Optionality::Missing(_) => return Ok(cnt),
                Optionality::Failed(e) => return Err(e),
            }
        }
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

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        let start = ctx.next_free.get();
        let mut prev = None;
        loop {
            ctx.next_free.set(start);
            let this = optional(ctx.clone(), &self.inner).await;
            match (prev, this) {
                (_, Optionality::Parsed(v)) => prev = Some(v),
                (_, Optionality::Summoned(v)) => return Ok(v),
                (None, Optionality::Missing(e) | Optionality::Failed(e)) => return Err(e),
                (Some(v), Optionality::Missing(_)) => return Ok(v),
                (Some(_), Optionality::Failed(e)) => return Err(e),
            }
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        self.inner.visit(visitor);
        visitor.pop_group();
    }
}

pub struct Collect<C, T> {
    pub(crate) inner: RcParser<T>,
    pub(crate) ctx: PhantomData<C>,
    pub(crate) min: u32,
    pub(crate) max: u32,
}

impl<T: 'static, C: FromIterator<T> + 'static> Parser for Collect<C, T> {
    type Output = C;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        Ok(parse_many(&self.inner, ctx, self.min, self.max)
            .await?
            .into_iter()
            .collect())
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        if self.min > 1 {
            self.inner.visit(visitor);
        } else {
            visitor.push_group(VisitGroup::Optional);
            self.inner.visit(visitor);
            visitor.pop_group();
        }
        visitor.pop_group();
    }
}

impl<C, T> Collect<C, T> {
    pub fn range<B: RangeBounds<usize>>(mut self, r: B) -> Self {
        use std::ops::Bound;
        self.min = match r.start_bound().map(|v| (*v).try_into().unwrap_or(u32::MAX)) {
            Bound::Included(b) => b,
            Bound::Excluded(b) => b.saturating_add(1),
            Bound::Unbounded => 0,
        };

        self.max = match r.end_bound().map(|v| (*v).try_into().unwrap_or(u32::MAX)) {
            Bound::Included(b) => b,
            Bound::Excluded(b) => b.saturating_add(1),
            Bound::Unbounded => u32::MAX,
        };
        self
    }
}

pub struct Many<T> {
    pub(crate) inner: RcParser<T>,
}

impl<T: 'static> Parser for Many<T> {
    type Output = Vec<T>;

    fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> impl Future<Output = Result<Vec<T>, Error>> + 'p {
        parse_many(&self.inner, ctx, 0, u32::MAX)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        visitor.push_group(VisitGroup::Many);
        visitor.push_group(VisitGroup::Optional);
        self.inner.visit(visitor);
        visitor.pop_group();
        visitor.pop_group();
    }
}

/// run `parser` several times
/// -
async fn parse_many<'p, T: 'static>(
    parser: &'p impl Parser<Output = T>,
    ctx: crate::Ctx<'p>,
    min: u32,
    max: u32,
) -> Result<Vec<T>, Error> {
    let mut res = Vec::new();
    let start = ctx.next_free.get();
    while matches!(&*ctx.wakeup_reason.borrow(), Reason::Pass | Reason::Push) {
        ctx.next_free.set(start);
        match optional(ctx.clone(), parser).await {
            Optionality::Parsed(v) => res.push(v),

            Optionality::Summoned(v) if res.is_empty() => res.push(v),
            // if value was produced without consuming anything - values
            // past the first one are not helpful (and they will never stop)
            Optionality::Summoned(_) => break,

            // - no more data available
            // - got enough parses to satisfy the constraint
            // - no data is lost
            Optionality::Missing(_e) if res.len() >= min as usize => break,
            Optionality::Missing(e) | Optionality::Failed(e) => return Err(e),
        }

        if res.len() >= max as usize {
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
    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Vec<T>, Error> {
        let res = parse_many(&self.inner, ctx, 0, u32::MAX).await?;
        if res.is_empty() {
            Err(Error::missing(MissingItem::Some {
                item: self.message.to_string(),
            }))
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
