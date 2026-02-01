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
    assert_eq!(r, "couldn't parse '-42': invalid digit found in string\n");
}

#[test]
fn negative_literal_invalid_value() {
    let parser = short('a').argument::<i32>("N").negative_lit().to_options();

    let r = parser.run_inner("-a -x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '-x': invalid digit found in string\n");
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
        .then_exit(|_| fail("-a requires a numeric value"));
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
