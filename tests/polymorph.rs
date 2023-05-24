use std::str::FromStr;

use bpaf::*;

#[test]
fn generic_argument_field() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Poly<T> {
        field: T,
    }

    fn poly<T>(name: &'static str) -> impl Parser<Poly<T>>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let field = long(name).argument("ARG");
        construct!(Poly { field })
    }

    let a = poly::<usize>("usize").optional();
    let b = poly::<u32>("u32").optional();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(&["--usize", "12"]).unwrap();
    assert_eq!(r, (Some(Poly { field: 12 }), None));

    let r = parser.run_inner(&["--u32", "12"]).unwrap();
    assert_eq!(r, (None, Some(Poly { field: 12 })));

    let r = parser.run_inner(&["--u32", "12", "--usize", "24"]).unwrap();
    assert_eq!(r, (Some(Poly { field: 24 }), Some(Poly { field: 12 })));
}
