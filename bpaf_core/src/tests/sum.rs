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

#[test]
fn either_of_two_required_flags_and_one_optional() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').flag('c', 'C');
    let p = a.or_else(b).or_else(c);
    let decorated = p.to_options().version("1.0");

    let ver = decorated.run_inner("-V").unwrap_err().unwrap_stdout();
    assert_eq!("Version: 1.0\n", ver);

    // help is always generated
    let help = decorated.run_inner("-h").unwrap_err().unwrap_stdout();
    let expected_help = "\
Usage: app [-a | -b | -c]

Available options:
    -a
    -b
    -c
    -h, --help     Prints help information
    -V, --version  Prints version information
";
    assert_eq!(expected_help, help);

    // fallback to default (from C)
    let res = decorated.run_inner("").unwrap();
    assert_eq!(res, 'C');
}
