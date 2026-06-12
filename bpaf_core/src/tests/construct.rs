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
fn empty_struct() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Foo {}
    let parser = construct!(Foo {}).to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, Foo {});

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: app\n\nAvailable options:\n    -h, --help  Prints help information\n"
    );
}

#[test]
fn empty_tuple() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Foo();
    // can't be 'Foo ()' - it's not a function, but {} works.
    //
    // Whole things is a moot point. If you want a parser that does
    // nothing but produces something - use 'pure(Foo())'
    let parser = construct!(Foo {}).to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, Foo());

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "Usage: app\n\nAvailable options:\n    -h, --help  Prints help information\n"
    );
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

#[test]
fn make_pure_named_enum() {
    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    enum Foo {
        Bar {},
    }

    let parser = construct!(Foo::Bar {}).to_options();
    let r = parser.run_inner("").unwrap();

    assert_eq!(r, Foo::Bar {});
}

#[test]
fn with_absolute_path() {
    let a = short('a').switch();
    let parser = construct!(::std::option::Option::Some(a)).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Some(true));
}

#[test]
fn with_long_rel_path() {
    let a = short('a').switch();
    let parser = construct!(std::option::Option::Some(a)).to_options();
    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Some(true));
}

#[test]
fn make_struct2_pos() {
    let a = short('a').switch();
    let b = short('b').switch();
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct Ab {
        a: bool,
        b: bool,
    }

    let parser = construct!(Ab { a, b }).to_options();
    let r = parser.run_inner("-a -b").unwrap();
    assert_eq!(r, Ab { a: true, b: true });
}

#[test]
fn make_struct2_named() {
    let a = short('a').switch();
    let b = short('b').switch();
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct Ab(bool, bool);

    let parser = construct!(Ab(a, b)).to_options();
    let r = parser.run_inner("-a -b").unwrap();
    assert_eq!(r, Ab(true, true));
}

#[test]
fn into_box_basic() {
    let a = short('a').switch();
    let boxed = a.into_box();
    let parser = boxed.to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn into_box_no_flag() {
    let a = short('a').switch();
    let boxed = a.into_box();
    let parser = boxed.to_options();
    let r = parser.run_inner("").unwrap();
    assert!(!r);
}

#[test]
fn into_box_then_into_rc() {
    let a = short('a').switch();
    let boxed = a.into_box();
    let rc = boxed.into_rc();
    let parser = rc.to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn into_rc_then_into_box() {
    let a = short('a').switch();
    let rc = a.into_rc();
    let boxed = rc.into_box();
    let parser = boxed.to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn into_box_idempotent() {
    let a = short('a').switch();
    let boxed = a.into_box();
    let boxed2 = boxed.into_box();
    let parser = boxed2.to_options();
    let r = parser.run_inner("-a").unwrap();
    assert!(r);
}

#[test]
fn into_box_in_construct() {
    struct MkFoo {
        a: BoxParser<bool>,
        b: BoxParser<bool>,
    }

    impl Parser for MkFoo {
        type Output = (bool, bool);

        async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
            let a = ctx.spawn(Kind::Prod, &self.a);
            let b = ctx.spawn(Kind::Prod, &self.b);
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
            visitor.push_group(VisitGroup::Prod);
            self.a.visit(visitor);
            self.b.visit(visitor);
            visitor.pop_group();
        }
    }

    let a = short('a').switch().into_box();
    let b = short('b').switch().into_box();
    let parser = MkFoo { a, b }.to_options();
    let r = parser.run_inner("-a -b").unwrap();
    assert_eq!(r, (true, true));
}
