use super::*;

pub trait Tuple {
    type Res;
    fn tparse(self) -> Parser<Self::Res>;
}

pub fn tparse<P>(parsers: P) -> Parser<P::Res>
where
    P: Tuple,
{
    parsers.tparse()
}

impl<P1, P2> Tuple for (Parser<P1>, Parser<P2>)
where
    P1: 'static,
    P2: 'static,
{
    type Res = (P1, P2);
    fn tparse(self) -> Parser<(P1, P2)> {
        let parse = move |rest| {
            let (a, rest) = (self.0.parse)(rest)?;
            let (b, rest) = (self.1.parse)(rest)?;
            Ok(((a, b), rest))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.0.meta.clone().and(self.1.meta.clone()),
        }
    }
}

impl<P1, P2, P3, P4, P5, P6> Tuple
    for (
        Parser<P1>,
        Parser<P2>,
        Parser<P3>,
        Parser<P4>,
        Parser<P5>,
        Parser<P6>,
    )
where
    P1: 'static,
    P2: 'static,
    P3: 'static,
    P4: 'static,
    P5: 'static,
    P6: 'static,
{
    type Res = (P1, P2, P3, P4, P5, P6);
    fn tparse(self) -> Parser<Self::Res> {
        let parse = move |rest| {
            let (a, rest) = (self.0.parse)(rest)?;
            let (b, rest) = (self.1.parse)(rest)?;
            let (c, rest) = (self.2.parse)(rest)?;
            let (d, rest) = (self.3.parse)(rest)?;
            let (e, rest) = (self.4.parse)(rest)?;
            let (f, rest) = (self.5.parse)(rest)?;
            Ok(((a, b, c, d, e, f), rest))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self
                .0
                .meta
                .clone()
                .and(self.1.meta.clone())
                .and(self.2.meta.clone())
                .and(self.3.meta.clone())
                .and(self.4.meta.clone())
                .and(self.5.meta.clone()),
        }
    }
}
