use crate::{Ctx, Error, Item, Parser, VKind, Visitor, traits::Leaf, visitors::help::Place};

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Copy, Clone)]
pub enum Block {
    Start(Place),
    EndSection,
}

impl TryFrom<u32> for Block {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            12 => Ok(Block::Start(Place::Named)),
            13 => Ok(Block::Start(Place::Pos)),
            14 => Ok(Block::Start(Place::Command)),
            15 => Ok(Block::Start(Place::Section)),
            16 => Ok(Block::EndSection),
            _ => Err(()),
        }
    }
}

pub const NAMED: &str = "\u{1B}[12m";
pub const POS: &str = "\u{1B}[13m";
pub const CMD: &str = "\u{1B}[14m";
pub const CUSTOM: &str = "\u{1B}[15m";
pub const END: &str = "\u{1B}[16m";

pub struct HelpLiteral<P> {
    pub(crate) inner: P,
    pub(crate) text: &'static str,
}

impl<P: Parser> Parser for HelpLiteral<P> {
    type Output = P::Output;

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<P::Output, Error>> {
        self.inner.eval(ctx)
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        if visitor.identify() == VKind::Help {
            visitor.item(Item::Rendered { text: self.text });
        } else {
            self.inner.visit(visitor)
        }
    }
}

impl<P: Leaf> Leaf for HelpLiteral<P> {}
