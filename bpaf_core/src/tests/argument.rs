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
    let parser = short('a')
        .argument::<i32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("-a -42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("-a=-42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("-a -0").unwrap();
    assert_eq!(r, 0);
}

#[test]
fn negative_literal_positive_values_still_work() {
    let parser = short('a')
        .argument::<i32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("-a 42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn negative_literal_with_unsigned_fails() {
    let parser = short('a')
        .argument::<u32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("-a -42").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse '-42': invalid digit found in string\n"
    );
}

#[test]
fn negative_literal_invalid_value() {
    let parser = short('a')
        .argument::<i32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("-a -x").unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse '-x': invalid digit found in string\n"
    );
}

#[test]
fn negative_literal_missing_value() {
    let parser = short('a')
        .argument::<i32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "'-a' expects a value 'N'\n");
}

#[test]
fn negative_literal_with_long() {
    let parser = long("num")
        .argument::<i32>("N")
        .negative_lit()
        .to_options();

    let r = parser.run_inner("--num -42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("--num=-42").unwrap();
    assert_eq!(r, -42);

    let r = parser.run_inner("--num 42").unwrap();
    assert_eq!(r, 42);
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
