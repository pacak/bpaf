use crate::*;

#[test]
fn sum_via_construct() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let parser = construct!([a, b, c]).to_options();

    let r = parser.run_inner("-c").unwrap();
    assert_eq!(r, 'c');

    let expected = "Usage: app (-a | -b | -c)

Available options:
    -a
    -b
    -c
    -h, --help  Prints help information
";
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);
}

#[test]
fn sum_via_or_else() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let parser = a.or_else(b).or_else(c).to_options();
    let r = parser.run_inner("-c").unwrap();
    assert_eq!(r, 'c');

    let expected = "Usage: app (-a | -b | -c)

Available options:
    -a
    -b
    -c
    -h, --help  Prints help information
";
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);
}
