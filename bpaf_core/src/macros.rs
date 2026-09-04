/// Combine several `Parser`s into a single `Parser`
///
/// This macro combines several individual [`Parser`](crate::Parser)s into a single parser
/// that produces a single Rust type. For example given `Parser`s for struct fields it can make
/// a `Parser` that produce this `struct`. Additionally it can combine several different `Parser`s
/// that produce values of the same type into a `Parser` that tries them all and picks the best one.
///
/// Value `construct!` returns also implements the `Parser` trait.
///
/// # Usage reference
/// ```rust
/// # use bpaf::*;
/// # { struct Res(bool, bool, bool);
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 1. structs with unnamed fields:
/// let _ = construct!(Res(a, b, c));
/// # }
///
/// # { struct Res { a: bool, b: bool, c: bool }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 2. structs with named fields:
/// let _ = construct!(Res {a, b, c});
/// # }
///
/// # { enum Ty { Res(bool, bool, bool) }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 3. enums with unnamed fields:
/// let _ = construct!(Ty::Res(a, b, c));
/// # }
///
/// # { enum Ty { Res { a: bool, b: bool, c: bool } }
/// # let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 4. enums with named fields:
/// let _ = construct!(Ty::Res {a, b, c});
/// # }
///
/// # { let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 5. tuples (see notes below):
/// let _ = construct!(a, b, c);
/// # }
///
/// # { let a = short('a').switch(); let b = short('b').switch(); let c = short('c').switch();
/// // 6. parallel composition (see notes below):
/// let _ = construct!([a, b, c]);
/// # }
/// ```
///
/// ## Notes:
/// - A tuple of parsers of size up to 12 is a [`Parser`](crate::Parser) for a tuple, e.g.
///   `construct!(a, b, c) ≈ (a, b, c)`.
/// - For parallel composition `bpaf` tries all the parsers in a group and
///   picks one that consumes the leftmost value; if parsers consume the same (or nothing), it picks
///   the leftmost parser in the group.
/// - Parallel composition is accessible using [`Parser::or_else`](crate::Parser::or_else) method.
///   `construct!([a, b, c]) ≈ a.or_else(b).or_else(c)`. Parsers created with `construct!` are as
///   efficient as the equivalent chain of `or_else` calls.
///
/// # Combinatoric usage
/// `construct!` can compose parsers sequentially or in parallel.
///
/// Sequential composition combines individual parsers into a parser object of a new type. The
/// composed parser yields a value of that type only when all of its constituent parsers
/// succeed. Placeholder names for values inside `construct!` macro must correspond to both
/// `struct`/`enum` names and parser names present in scope. In examples below `a` corresponds to a
/// function and `b` corresponds to a variable name. Parentheses in `a()` are required when `a` is
/// a function that produces a parser rather than a variable that holds a parser.
///
/// ```rust
/// # use bpaf::*;
/// // Functions can be shared across multiple `construct!` invocations
/// fn a() -> impl Parser<Output = u32> {
///     short('a').argument::<u32>("N")
/// }
///
/// // Construction of structs or enums with unnamed fields
/// struct Res (u32, u32);
/// fn res() -> impl Parser<Output = Res> {
///     let b = short('b').argument::<u32>("n");
///     construct!(Res ( a(), b ))
/// }
///
/// // Construction of structs or enums with named fields
/// enum Ul {
///     T { a: u32, b: u32 },
/// }
/// fn ult() -> impl Parser<Output = Ul> {
///     let b = short('b').argument::<u32>("n");
///     construct!(Ul::T { a(), b })
/// }
///
/// // Construction of simple tuples
/// fn tuple() -> impl Parser<Output = (u32, u32)> {
///     let b = short('b').argument::<u32>("n");
///     construct!(a(), b)
/// }
///
/// // tuples of size up to 12 work without `construct!`
/// fn short_tuple() -> impl Parser<Output = (u32, u32)> {
///     (a(), short('b').argument::<u32>("n"))
/// }
/// ```
///
/// Parallel composition picks one of several available parsers (result types must match) and returns a
/// parser object of the same type. As in sequential composition, parsers come from variables
/// or functions:
///
/// ```rust
/// # use bpaf::*;
/// fn b() -> impl Parser<Output = u32> {
///     short('b').argument::<u32>("NUM")
/// }
///
/// fn a_or_b() -> impl Parser<Output = u32> {
///     let a = short('a').argument::<u32>("NUM");
/// // equivalent to `a.or_else(b())`
/// construct!([a, b()])
/// }
/// ```
///
/// # Derive usage
///
/// `bpaf` combines fields of struct or enum constructors sequentially and enum
/// variants in parallel.
/// ```rust
/// # use bpaf::*;
/// // to satisfy this parser user needs to pass both `-a` and `-b`
/// #[derive(Debug, Clone, Bpaf)]
/// struct Res {
///     a: u32,
///     b: u32,
/// }
///
/// // to satisfy this parser user needs to pass exactly one of `-a`, `-b`, `-c` or `-d`
/// #[derive(Debug, Clone, Bpaf)]
/// enum Enumeraton {
///     A { a: u32 },
///     B { b: u32 },
///     C { c: u32 },
///     D { d: u32 },
/// }
///
/// // here user needs to pass either both `-a` AND `-b` or both `-c` AND `-d`
/// #[derive(Debug, Clone, Bpaf)]
/// enum Ult {
///     AB { a: u32, b: u32 },
///     CD { c: u32, d: u32 }
/// }
/// ```
#[macro_export]
macro_rules! construct {
    // sadly can't use $name:path around here since it conflicts with `(` in positional items
    // `construct!(Enum::Cons { a, b, c })`
    (    $ns:ident $(::$con:ident)* { $($field:tt)* }) =>
        {{ $crate::prepare!([named    $ns $( ::$con)*] [] $($field)*) }};

    ( :: $ns:ident $(::$con:ident)* { $($field:tt)* }) =>
        {{ $crate::prepare!([named :: $ns $( ::$con)*] [] $($field)*) }};

    // `construct!(Enum::Cons ( a, b, c ))`
    (   $ns:ident $(:: $con:ident)* ( $($field:tt)* )) =>
        {{ $crate::prepare!([pos    $ns $(:: $con)*] [] $($field)*) }};

    ( :: $ns:ident $(:: $con:ident)* ( $($field:tt)* )) =>
        {{ $crate::prepare!([pos :: $ns $(:: $con)*] [] $($field)*) }};

    // construct!([a, b, c])
    ([ $($field:tt)+ ]) => // first - to make sure we have at lest one item
        {{ $crate::prepare!([alt] [] $($field)+) }};

    // construct!( a, b, c )
    ( $($field:tt)+) =>
        {{ $crate::prepare!([pos] [] $($field)+) }};

}

/// Instantiate parsers for fields given by functions
#[doc(hidden)]
#[macro_export]
macro_rules! prepare {

    // instantiate field from a function call
    ($ty:tt [$($fields:tt)*] $field:ident() $(, $($rest:tt)*)? ) => {{
        let $field = $field();
        $crate::prepare!($ty [$($fields)* $field] $($($rest)*)?)
    }};
    // otherwise, field is already a variable - we can use it as is.
    ($ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::prepare!($ty [$($fields)* $field] $($($rest)* )?)
    }};

    // All the logic for sum parser sits inside of Sum datatype
    ([alt] [$($f:ident)+]) => {
        $crate::__private::Sum{ items: ::std::vec![ $( $crate::Parser::into_box($f) ),+] }
    };


    // this block is for debugging of prod only
    // ($ty:tt [$($f:tt)+]) => {
    //     $crate::prod!($ty [$($f)+])
    // };

    // 13+ fields in a product - generate new dummy structure and a parser for that
    ($ty:tt [$a:tt $b:tt $c:tt $d:tt $e:tt $f:tt $g:tt $h:tt $i:tt $j:tt $k:tt $l:tt $($m:tt)+]) => {
        $crate::prod!($ty [ $a $b $c $d $e $f $g $h $i $j $k $l $($m)+])
    };

    // reuse tuple logic
    ($ty:tt $fs:tt) => { $crate::via_tuple!($ty $fs) }
}

#[doc(hidden)]
#[macro_export]
macro_rules! via_tuple {
    // single item positional and named - can use directly with a `map`
    ([pos   $($con:tt)+] [$f:ident]) => { $crate::Parser::map($f, |$f| $($con)+ ($f)) };
    ([named $($con:tt)+] [$f:ident]) => { $crate::Parser::map($f, |$f| $($con)+ {$f}) };

    // tuple below 13 items - use tuple instance directly
    ([pos] [$($f:ident)+]) => { ( $($f),+) };


    // for named/positional below 13 items - go via tuple
    ([pos   $($con:tt)+] [$($f:ident)+]) => { $crate::Parser::map( ($($f),+), |($($f),+)|  $($con)+ ($($f),+)) };
    ([named $($con:tt)+] [$($f:ident)+]) => { $crate::Parser::map( ($($f),+), |($($f),+)|  $($con)+ {$($f),+}) };

    ([named $($con:tt)+] []) => { $crate::pure( $($con)+ {} ) };
}

#[doc(hidden)]
#[macro_export]
macro_rules! prod {
    ($ty:tt [$($f:ident)+]) => {{
        mod ty {
            #![allow(non_camel_case_types, unused_parens, clippy::double_parens, unused_imports)]
            use $crate::__private::*;
            pub(super) struct Ty<$($f),+> {
                $( pub $f: $f, )+
            }
            impl <$($f: Parser + 'static),+> Parser for Ty<$( $f ),+> {
                type Output = ($($f::Output),+);

                async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
                    $( let $f = ctx.spawn(Kind::Prod, &self.$f); )+
                    ctx.wait_for_children().await;
                    let mut err = None;

                    $( let $f = $f.take().map_err(|e| e.append_to(&mut err)); )+
                    if let Some(err) = err {
                        Err(err)
                    } else {
                        Ok(($( $f? ),+))
                    }
                }

                fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
                    visitor.push_group(VisitGroup::Prod);
                    $( self.$f.visit(visitor); )+
                    visitor.pop_group();

                }
            }
        }

        #[allow(non_camel_case_types, unused_parens)]
        $crate::Parser::into_box(
            $crate::Parser::map(
                ty::Ty { $($f: $f),+},
                |($($f),+)| $crate::make!($ty [ $($f)+ ])
            )
        )
    }}

}

#[doc(hidden)]
#[macro_export]
/// Pack parsed results into a constructor
macro_rules! make {
    // this gets called from prod!
    //
    // for named they go into {}
    ([named $($con:tt)+] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields),* } };
    // for positional - (), if there's no constructor - we are making a tuple
    ([pos   $($con:tt)*] [$($fields:ident)*]) => { $($con)* ( $($fields),* ) };
}

#[cfg(feature = "extradocs")]
macro_rules! example_encase {
    ($title:literal, $inner:expr) => {
        concat!(
            "<details><summary>",
            $title,
            "</summary>\n\n",
            $inner,
            "\n\n</details>"
        )
    };
}

#[cfg(feature = "extradocs")]
macro_rules! example_derive {
    ($name:literal) => {
        $crate::macros::example_encase!(
            "Derive example",
            concat!(
                "```no_run\n",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/examples/",
                    $name,
                    "/derive.rs"
                )),
                "\n```"
            )
        )
    };
}

#[cfg(feature = "extradocs")]
macro_rules! example_combo {
    ($name:literal) => {
        $crate::macros::example_encase!(
            "Combinatoric example",
            concat!(
                "```no_run\n",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/examples/",
                    $name,
                    "/combo.rs"
                )),
                "\n```"
            )
        )
    };
}

#[cfg(feature = "extradocs")]
macro_rules! example_readme {
    ($name:literal) => {
        concat!(
            "\n\n",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/",
                $name,
                "/README.md"
            ))
        )
    };
}

#[cfg(feature = "extradocs")]
macro_rules! example_output {
    ($name:literal) => {
        $crate::macros::example_encase!(
            "Sample output",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/",
                $name,
                "/OUTPUT.md"
            ))
        )
    };
}

#[allow(unused_macros)]
#[cfg(feature = "extradocs")]
macro_rules! example_c {
    ($name:literal) => {
        concat!(
            $crate::macros::example_combo!($name),
            $crate::macros::example_output!($name)
        )
    };
}

#[allow(unused_macros)]
#[cfg(feature = "extradocs")]
macro_rules! example_d {
    ($name:literal) => {
        concat!(
            $crate::macros::example_readme!($name),
            $crate::macros::example_derive!($name),
            $crate::macros::example_outupt!($name)
        )
    };
}

#[cfg(feature = "extradocs")]
macro_rules! example_cd {
    ($name:literal) => {
        concat!(
            $crate::macros::example_readme!($name),
            $crate::macros::example_combo!($name),
            $crate::macros::example_derive!($name),
            $crate::macros::example_output!($name)
        )
    };
}
#[cfg(not(feature = "extradocs"))]
#[allow(unused_macros)]
macro_rules! example_c {
    ($dummy:literal) => {
        ""
    };
}

#[cfg(not(feature = "extradocs"))]
#[allow(unused_macros)]
macro_rules! example_d {
    ($dummy:literal) => {
        ""
    };
}

#[cfg(not(feature = "extradocs"))]
macro_rules! example_cd {
    ($dummy:literal) => {
        ""
    };
}

#[allow(unused_imports)]
pub(crate) use {example_c, example_cd, example_d};
#[cfg(feature = "extradocs")]
pub(crate) use {example_combo, example_derive, example_encase, example_output, example_readme};
