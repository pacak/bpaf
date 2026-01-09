#[macro_export]
macro_rules! construct {
    // sadly can't use name::path around here since it conflicts with `(` in positional items
    // `construct!(Enum::Cons { a, b, c })`
    ( $ns:ident $(::$con:ident)* { $($rest:tt)* }) =>
        {{ $crate::macros::prepare!([named $ns $( ::$con)*] [] $($rest)*) }};

    // `construct!(Enum::Cons ( a, b, c ))`
     ( $ns:ident $(:: $con:ident)* ( $($rest:tt)* )) =>
        {{ $crate::macros::prepare!([pos $ns $(:: $con)*] [] $($rest)*) }};

    // construct!([a, b, c])
    ([ $($name:tt)+ ]) => // first - to make sure we have at lest one item
        {{ $crate::macros::prepare!([alt] [] $($name)+) }};

    // construct!( a, b, c )
    ( $($name:tt)+) =>
        {{ $crate::macros::prepare!([pos] [] $($name)+) }};

}

/// Instantiate parsers for fields given by functions
macro_rules! prepare {

    // instantiate field from a function call
    ($ty:tt [$($fields:tt)*] $field:ident() $(, $($rest:tt)*)? ) => {{
        let $field = $field();
        $crate::macros::prepare!($ty [$($fields)* $field] $($($rest)*)?)
    }};
    // otherwise field is already a variable - we can use it as is.
    ($ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::macros::prepare!($ty [$($fields)* $field] $($($rest)* )?)
    }};

    // All the logic for sum parser sits inside of Sum datatype
    ([alt] [ $($field:ident)*]) => {
        $crate::__private::Sum{ items: ::std::vec![ $($field.into_rc()),*] }
    };

    // For product type the logic is a bit more complicated - do one more step
    ($ty:tt [$($fields:tt)*]) => {
        $crate::macros::fin!($ty [ $($fields)* ])
    };
}

// === Making a body for the product parser
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
                        ($crate::macros::make!($ty [$($fields)*]))
                })
            });

        $crate::__private::Prod { run, visits }

    }};
}
macro_rules! make {
    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    ([named $($con:tt)+] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields?),* } };
    // for positional - (), if there's no constructor - we are making a tuple
    ([pos   $($con:tt)*] [$($fields:ident)*]) => { $($con)* ( $($fields?),* ) };

}

pub(crate) use {fin, make, prepare};
