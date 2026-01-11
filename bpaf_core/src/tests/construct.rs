use crate::*;

#[test]
fn make_tuple_var() {
    let a = short('a').switch();
    let parser = construct!(a).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn make_tuple_func_no_arg() {
    fn a() -> impl Parser<Output = bool> {
        short('a').switch()
    }
    fn b() -> impl Parser<Output = bool> {
        short('b').switch()
    }
    let parser = construct!(a(), b()).to_options();
    let r = parser.run_inner("-a -b").unwrap();
    assert_eq!(r, (true, true));
}

#[test]
fn make_struct_var() {
    struct A {
        a: bool,
    }
    let a = short('a').switch();
    let parser = construct!(A { a }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A { a: true }));
}

#[test]
fn make_struct_func_no_arg() {
    struct A {
        a: bool,
    }
    fn a() -> impl Parser<Output = bool> {
        short('a').switch()
    }
    let parser = construct!(A { a() }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A { a: true }));
}

#[test]
fn make_enum_var() {
    enum A {
        A { a: bool },
    }
    let a = short('a').switch();
    let parser = construct!(A::A { a }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A::A { a: true }));
}

#[test]
fn make_enum_func_no_arg() {
    enum A {
        A { a: bool },
    }
    fn a() -> impl Parser<Output = bool> {
        short('a').switch()
    }
    let parser = construct!(A::A { a() }).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(matches!(r, A::A { a: true }));
}

#[test]
fn make_alt_var() {
    let a = short('a').switch();
    let parser = construct!([a]).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn make_alt_func_no_arg() {
    fn a() -> impl Parser<Output = bool> {
        short('a').switch()
    }
    let parser = construct!([a()]).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Foo {
    a: bool,
    b: bool,
}

fn foo2() -> impl Parser<Output = Foo> {
    struct Fo<A, B> {
        a: A,
        b: B,
    }

    impl<A: Parser + Clone + 'static, B: Parser + Clone + 'static> Parser for Fo<A, B> {
        type Output = (A::Output, B::Output);

        async fn run(&self, ctx: Ctx) -> Result<Self::Output, Error> {
            let a = ctx.spawn(Kind::Prod, self.a.clone().into_rc());
            let b = ctx.spawn(Kind::Prod, self.b.clone().into_rc());
            r#yield().await;

            let mut err = None;
            let a = a.take().map_err(|e| e.append_to(&mut err));
            let b = b.take().map_err(|e| e.append_to(&mut err));
            if let Some(err) = err {
                Err(err)
            } else {
                Ok((a?, b?))
            }
        }

        fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
            todo!()
        }
    }

    let a = short('a').switch().into_rc();
    let b = short('b').switch().into_rc();
    Fo { a, b }.map(|(a, b)| Foo { a, b })
}

fn foo() -> impl Parser<Output = Foo> {
    mod x {
        use super::*;

        pub(super) struct MkFoo {
            pub(super) a: RcParser<bool>,
            pub(super) b: RcParser<bool>,
        }

        impl Parser for MkFoo {
            type Output = Foo;

            async fn run(&self, ctx: Ctx) -> Result<Self::Output, Error> {
                let a = ctx.spawn(Kind::Prod, self.a.clone());
                let b = ctx.spawn(Kind::Prod, self.b.clone());
                r#yield().await;

                let mut err = None;
                let a = a.take().map_err(|e| e.append_to(&mut err));
                let b = b.take().map_err(|e| e.append_to(&mut err));
                if let Some(err) = err {
                    Err(err)
                } else {
                    Ok(Foo { a: a?, b: b? })
                }
            }

            fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
                visitor.push_group(VisitGroup::Prod);
                self.a.visit(visitor);
                self.b.visit(visitor);
                visitor.pop_group();
            }
        }
    }
    use x::MkFoo;

    let a = short('a').switch().into_rc();
    let b = short('b').switch().into_rc();

    MkFoo { a, b }
}

#[test]
fn asdf() {
    let parser = foo2().to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Foo { a: true, b: false });
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Foo {
    a: bool,
    b: bool,
}

fn foo2() -> impl Parser<Output = Foo> {
    struct Fo<A, B> {
        a: A,
        b: B,
    }

    impl<A: Parser + Clone + 'static, B: Parser + Clone + 'static> Parser for Fo<A, B> {
        type Output = (A::Output, B::Output);

        async fn run(&self, ctx: Ctx) -> Result<Self::Output, Error> {
            let a = ctx.spawn(Kind::Prod, self.a.clone().into_rc());
            let b = ctx.spawn(Kind::Prod, self.b.clone().into_rc());
            r#yield().await;

            let mut err = None;
            let a = a.take().map_err(|e| e.append_to(&mut err));
            let b = b.take().map_err(|e| e.append_to(&mut err));
            if let Some(err) = err {
                Err(err)
            } else {
                Ok((a?, b?))
            }
        }

        fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
            todo!()
        }
    }

    let a = short('a').switch().into_rc();
    let b = short('b').switch().into_rc();
    Fo { a, b }.map(|(a, b)| Foo { a, b })
}

fn foo() -> impl Parser<Output = Foo> {
    mod x {
        use super::*;

        pub(super) struct MkFoo {
            pub(super) a: RcParser<bool>,
            pub(super) b: RcParser<bool>,
        }

        impl Parser for MkFoo {
            type Output = Foo;

            async fn run(&self, ctx: Ctx) -> Result<Self::Output, Error> {
                let a = ctx.spawn(Kind::Prod, self.a.clone());
                let b = ctx.spawn(Kind::Prod, self.b.clone());
                r#yield().await;

                let mut err = None;
                let a = a.take().map_err(|e| e.append_to(&mut err));
                let b = b.take().map_err(|e| e.append_to(&mut err));
                if let Some(err) = err {
                    Err(err)
                } else {
                    Ok(Foo { a: a?, b: b? })
                }
            }

            fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
                visitor.push_group(VisitGroup::Prod);
                self.a.visit(visitor);
                self.b.visit(visitor);
                visitor.pop_group();
            }
        }
    }
    use x::MkFoo;

    let a = short('a').switch().into_rc();
    let b = short('b').switch().into_rc();

    MkFoo { a, b }
}

#[test]
fn asdf() {
    let parser = foo2().to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Foo { a: true, b: false });
}

#[test]
fn plain_tuple() {
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, (true, false));
}

#[test]
fn plain_pos() {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Foo(bool, bool);
    let a = short('a').switch();
    let b = short('b').switch();
    let parser = construct!(Foo(a, b)).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Foo(true, false));
}
