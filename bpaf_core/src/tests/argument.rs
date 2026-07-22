use crate::*;

#[test]
fn restrict_to_adjacent() {
    let a = short('a').long("aaa").argument::<u32>("A").adjacent();
    let parser = a.to_options();

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    let expected = "expected value to be adjacent to -a, try -a=42\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("--aaa=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn negative_literal_works() {
    let parser = short('a').argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a -42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("-a=-42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("-a -0").unwrap();
    assert_eq!(r, 0);
}

#[test]
fn negative_literal_positive_values_still_work() {
    let parser = short('a').argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a 42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn negative_literal_with_unsigned_fails() {
    let parser = short('a').argument::<u32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '-42', try '-a=-42' to use it as an argument\n";
    assert_eq!(r, expected);
}

#[test]
fn negative_literal_invalid_value() {
    let parser = short('a').argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a -x").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '-x', try '-a=-x' to use it as an argument\n";
    assert_eq!(r, expected);
}

#[test]
fn negative_literal_missing_value() {
    let parser = short('a').argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' expects a value 'N'\n");
}

#[test]
fn negative_literal_with_long() {
    let parser = long("num").argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("--num -42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("--num=-42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("--num 42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn mix_of_all_arg_like_methods() {
    fn missing() -> Result<u32, String> {
        Ok(42)
    }
    let p1 = long("num")
        .argument::<u32>("N")
        .adjacent()
        .negative_lit()
        .on_missing_value(missing)
        .into_box();

    let p2 = long("num")
        .argument::<u32>("N")
        .negative_lit()
        .adjacent()
        .on_missing_value(missing)
        .into_box();

    let p3 = long("num")
        .argument::<u32>("N")
        .on_missing_value(missing)
        .negative_lit()
        .adjacent()
        .into_box();

    for p in [p1, p2, p3] {
        let parser = p.to_options();

        let r = parser.run_inner("--num").unwrap();
        assert_eq!(r, 42);

        let r = parser.run_inner("--num=69").unwrap();
        assert_eq!(r, 69);

        let r = parser
            .run_inner("--num 131313")
            .unwrap_err()
            .unwrap_stderr();
        let expected = "expected value to be adjacent to --num, try --num=131313\n";
        assert_eq!(r, expected);
    }
}

#[test]
fn parse_errors() {
    let parser = short('a').argument::<i32>("ARG").to_options();

    let r = parser.run_inner("-a 123x").unwrap_err().unwrap_stderr();
    let expected = "couldn't parse '123x': invalid digit found in string\n";
    assert_eq!(expected, r);

    let r = parser.run_inner("-b 123x").unwrap_err().unwrap_stderr();

    let expected = "expected '-a=ARG', got '-b'\n";
    assert_eq!(expected, r);

    let r = parser.run_inner("-a 123 -b").unwrap_err().unwrap_stderr();
    let expected = "'-b' is not expected in this context\n";
    assert_eq!(expected, r);
}

#[test]
fn missing_value_handler_provides_default() {
    let a = short('o')
        .long("output")
        .argument::<String>("OUTPUT")
        .on_missing_value(|| Ok(String::from("-")));
    let parser = a.to_options();

    let r = parser.run_inner("--output").unwrap();
    assert_eq!(r, "-");

    let r = parser.run_inner("--output file.txt").unwrap();
    assert_eq!(r, "file.txt");
}

#[test]
fn missing_value_handler_provides_default_alt() {
    let a = short('o').long("output").req_flag(String::from("-")).hide();
    let a = short('o')
        .long("output")
        .argument::<String>("OUTPUT")
        .or_else(a);

    let parser = a.to_options();

    let r = parser.run_inner("--output").unwrap();
    assert_eq!(r, "-");

    let r = parser.run_inner("--output file.txt").unwrap();
    assert_eq!(r, "file.txt");
}

#[test]
fn missing_value_handler_returns_error() {
    let a = short('a')
        .argument::<u32>("NUM")
        .on_missing_value(|| Err(String::from("-a requires a numeric value")));
    let parser = a.to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    let expected = "-a requires a numeric value\n";
    assert_eq!(r, expected);
}

#[test]
fn missing_value_handler_returns_error_alt() {
    let a = short('a')
        .req_flag(())
        .hide()
        .then_exit(|_| Exit::failure("-a requires a numeric value"));
    let a = short('a').argument::<u32>("A").or_else(a);
    let parser = a.to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    let expected = "-a requires a numeric value\n";
    assert_eq!(r, expected);
}

#[test]
fn missing_value_handler_does_not_mask_missing() {
    let a = short('o')
        .argument::<String>("OUTPUT")
        .on_missing_value(|| Ok(String::from("-")));
    let parser = a.to_options();

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    let expected = "expected '-o=OUTPUT'\n";
    assert_eq!(r, expected);
}

#[test]
fn missing_value_handler_does_not_mask_parse_error() {
    let a = short('n')
        .argument::<u32>("NUM")
        .on_missing_value(|| Ok(42));
    let parser = a.to_options();

    let r = parser.run_inner("-n abc").unwrap_err().unwrap_stderr();
    let expected = "couldn't parse 'abc': invalid digit found in string\n";
    assert_eq!(r, expected);
}

#[test]
fn missing_value_handler_adjacent_interaction() {
    let a = short('a')
        .long("aaa")
        .argument::<u32>("A")
        .adjacent()
        .on_missing_value(|| Ok(99));
    let parser = a.to_options();

    let r = parser.run_inner("-a=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 99);
}

#[test]
fn strange_short_option() {
    use crate::*;
    let parser = short('O').argument::<String>("ARG").to_options();
    let r = parser.run_inner("-Obits=2048").unwrap();
    assert_eq!(r, "bits=2048");
}

#[test]
fn generic_argument_field() {
    use std::str::FromStr;
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Poly<T> {
        field: T,
    }

    fn poly<T>(name: &'static str) -> impl Parser<Output = Poly<T>>
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

    let r = parser.run_inner("--usize 12").unwrap();
    assert_eq!(r, (Some(Poly { field: 12 }), None));

    let r = parser.run_inner("--u32 12").unwrap();
    assert_eq!(r, (None, Some(Poly { field: 12 })));

    let r = parser.run_inner("--u32 12 --usize 24").unwrap();
    assert_eq!(r, (Some(Poly { field: 24 }), Some(Poly { field: 12 })));
}

#[test]
fn no_argument_problematic() {
    let a = short('a').argument::<i32>("N").negative_lit();
    let b = short('2').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -42").unwrap();
    assert_eq!(r, (-42, false));

    // this here shows why negative_lit must be an opt in
    let r = parser.run_inner("-a -2").unwrap();
    assert_eq!(r, (-2, false));
}

#[test]
fn no_argument() {
    let a = short('a').argument::<i32>("N").negative_lit();
    let b = short('k').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-a -42").unwrap();
    assert_eq!(r, (-42, false));

    let r = parser.run_inner("-a -k").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '-k', try '-a=-k' to use it as an argument\n";
    assert_eq!(r, expected);
}

#[test]
fn strict_positional_argument() {
    let a = short('a').argument::<String>("N");
    let parser = a.to_options();

    let r = parser.run_inner("-a -- 10").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '--', try '-a=--' to use it as an argument\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a --").unwrap_err().unwrap_stderr();
    let expected =
        "'-a' requires an argument 'N', got '--', try '-a=--' to use it as an argument\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=--").unwrap();
    assert_eq!(r, "--");
}
