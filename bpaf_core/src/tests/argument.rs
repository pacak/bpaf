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
