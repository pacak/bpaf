/// Blurb
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
