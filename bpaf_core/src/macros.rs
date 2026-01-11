/// Blurb
#[macro_export]
macro_rules! construct {
    // sadly can't use name::path around here since it conflicts with `(` in positional items
    // `construct!(Enum::Cons { a, b, c })`
    ( $ns:ident $(::$con:ident)* { $($field:tt)* }) =>
        {{ $crate::prepare!([named $ns $( ::$con)*] [] $($field)*) }};

    // `construct!(Enum::Cons ( a, b, c ))`
     ( $ns:ident $(:: $con:ident)* ( $($field:tt)* )) =>
        {{ $crate::prepare!([pos $ns $(:: $con)*] [] $($field)*) }};

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
    // otherwise field is already a variable - we can use it as is.
    ($ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::prepare!($ty [$($fields)* $field] $($($rest)* )?)
    }};

    // All the logic for sum parser sits inside of Sum datatype
    ([alt] [ $($field:ident)*]) => {
        $crate::__private::Sum{ items: ::std::vec![ $($field.into_rc()),*] }
    };

    // // For product type the logic is a bit more complicated - do one more step
    // ($ty:tt [$($fields:tt)*]) => {
    //     $crate::fin!($ty [ $($fields)* ])
    // };

    // 13+ fields in a product - generate new dummy structure and a parser for that
    ($ty:tt [$a:tt $b:tt $c:tt $d:tt $e:tt $f:tt $g:tt $h:tt $i:tt $j:tt $k:tt $l:tt $m:tt]) => {
        $crate::fin!($ty [ $a $b $c $d $e $f $g $h $i $j $k $l $m])
    };

    // reuse tuple logic
    ($ty:tt $fs:tt) => { $crate::via_tuple!($ty $fs) }
}

#[doc(hidden)]
#[macro_export]
macro_rules! via_tuple {
    // single item positional and named - can use directly with a `map`
    ([pos   $($con:tt)+] [$f:ident]) => { $f.map(|$f| $($con)+ ($f)) };
    ([named $($con:tt)+] [$f:ident]) => { $f.map(|$f| $($con)+ {$f}) };

    // tuple below 13 items - use tuple instance directly
    ([pos] [$($f:ident)+]) => { ( $($f.into_rc()),+) };

    // for named/positional below 13 items - go via tuple
    ([pos   $($con:tt)+] [$($f:ident)+]) => { ( $($f.into_rc()),+).map(|($($f),+)|  $($con)+ ($($f),+)) };
    ([named $($con:tt)+] [$($f:ident)+]) => { ( $($f.into_rc()),+).map(|($($f),+)|  $($con)+ {$($f),+}) };
}

/// Making a body for the product parser
#[doc(hidden)]
#[macro_export]
macro_rules! fin {

    ($ty:tt [$($fields:ident)*]) => {{
        $( let $fields = $fields.into_rc(); )*

        // This allocates...
        let visits = vec![ $($crate::Visited::into_box($fields.clone()) ),* ];

        let run:  ::std::boxed::Box<dyn ::std::ops::Fn($crate::__private::Ctx) ->
        ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::std::result::Result<_, $crate::__private::Error>>> =
            ::std::boxed::Box::new(move |ctx: $crate::__private::Ctx| {
                $( let $fields = ctx.spawn($crate::__private::Kind::Prod, $fields.clone());)*
                ::std::boxed::Box::new(||{
                    let mut err = ::std::option::Option::None;
                    $( let $fields = $fields.take().map_err(|e| e.append_to(&mut err)); )*

                    if let ::std::option::Option::Some(err) = err {
                        return ::std::result::Result::Err(err);
                    }

                    ::std::result::Result::Ok::<_, $crate::__private::Error>
                        ($crate::make!($ty [$($fields)*]))
                })
            });

        $crate::__private::Prod { run, visits }

    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! make {
    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    ([named $($con:tt)+] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields?),* } };
    // for positional - (), if there's no constructor - we are making a tuple
    ([pos   $($con:tt)*] [$($fields:ident)*]) => { $($con)* ( $($fields?),* ) };

}
