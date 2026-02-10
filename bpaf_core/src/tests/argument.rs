use crate::*;

#[test]
fn restrict_to_adjacent() {
    let a = short('a').long("aaa").argument::<u32>("A").adjacent();
    let parser = a.to_options();

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    let expected = "Expected value to be adjacent to -a, try -a=42\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("-a=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("--aaa=42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a42").unwrap();
    assert_eq!(r, 42);
}
