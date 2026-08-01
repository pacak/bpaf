use crate::{Ctx, Error, Parser, error::CvKind};

/// Wrapper that overrides completion help text for a parser.
///
/// Replaces help message for the item that is inside of it. This is useful when the item's help is
/// too verbose for shell completions.
///
/// The replacement is applied when the item name or its metavar is displayed as a completion
/// candidate.
///
/// Created with [`Parser::comp_help`].
pub struct CompHelp<P> {
    pub(crate) inner: P,
    pub(crate) help: &'static str,
}

impl<P: Parser> Parser for CompHelp<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<P::Output, Error> {
        match self.inner.eval(ctx).await {
            Ok(v) => Ok(v),
            Err(Error::CompValue(mut cv)) if matches!(cv.kind, CvKind::Item) || cv.meta_only => {
                cv.help = Some(self.help);
                Err(Error::CompValue(cv))
            }
            Err(e) => Err(e),
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

impl<P: crate::traits::Leaf> crate::traits::Leaf for CompHelp<P> {}
