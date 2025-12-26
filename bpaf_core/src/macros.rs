#[macro_export]
macro_rules! construct {
    // === capture initial shape of the query

    // `construct!(Enum::Cons { a, b, c })`
    ($ns:ident $(:: $con:ident)* { $($rest:tt)* }) =>
        {{ $crate::construct!(@prepare [named $ns $(:: $con)*] [] $($rest)*) }};

    // `construct!(Enum::Cons ( a, b, c ))`
    ($ns:ident $(:: $con:ident)* ( $($rest:tt)* )) =>
        {{ $crate::construct!(@prepare [pos $ns $(:: $con)*] [] $($rest)*) }};

    // construct!( a, b, c )
    ($first:ident $($rest:tt)*) => // first to make sure we have at least one item
        {{ $crate::construct!(@prepare [pos] [] $first $($rest)*) }};

    // construct!([a, b, c])
    ([$first:ident $($rest:tt)*]) => // first - to make sure we have at lest one item
        {{ $crate::construct!(@prepare [alt] [] $first $($rest)*) }};

    // === expand function calls in argument lists, if any
    // this is done for both prod and sum type constructors

    // instantiate field from a function call with possible arguments
    (@prepare $ty:tt [$($fields:tt)*] $field:ident ($($param:tt)*) $(, $($rest:tt)*)? ) => {{
        let $field = $field($($param)*);
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)*)?)
    }};
    // field is already a variable - we can use it as is.
    (@prepare $ty:tt [$($fields:tt)*] $field:ident $(, $($rest:tt)*)? ) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($($rest)* )?)
    }};

    // === fields are done (no 4th argument), can start constructing parsers

    // All the logic for sum parser sits inside of Alt datatype
    (@prepare [alt] [ $($field:ident)*]) => {
        $crate::__private::Alt { items: ::std::vec![ $($crate::Parser::into_rc($field)),*] }
    };

    // For product type the logic is a bit more complicated - do one more step
    (@prepare $ty:tt [$($fields:tt)*]) => {
        $crate::construct!(@fin $ty [ $($fields)* ])
    };

    // === Making a body for the product parser

    // Two special cases where we construct something with no fields, use `Parser::pure` for that
    (@fin [named $($con:tt)+] []) => { $crate::pure($($con)+ { })};
    (@fin [pos   $($con:tt)+] []) => { $crate::pure($($con)+ ( ))};

    (@fin $ty:tt [$($fields:ident)*]) => {{
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
                        ($crate::construct!(@make $ty [$($fields)*]))
                })
            });

        $crate::__private::Con { run, visits }

    }};

    // === Pack parsed results into a constructor
    // this gets called from a step above
    //
    // for named they go into {}
    (@make [named $($con:tt)+] [$($fields:ident)*]) => { $($con)+ {  $($fields: $fields?),* } };
    // for positional - (), if there's no constructor - we are making a tuple
    (@make [pos   $($con:tt)*] [$($fields:ident)*]) => { $($con)* ( $($fields?),* ) };
}
