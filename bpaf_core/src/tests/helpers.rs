use crate::*;

#[test]
fn cargo_helper_works() {
    let a = short('a').switch().help("opt a");
    let b = short('b').argument::<usize>("B").help("opt b");
    let parser = cargo_helper("asm", (a, b)).to_options();

    let r = parser.run_inner("asm -a -b 3").unwrap();
    assert_eq!(r, (true, 3));

    let r = parser.run_inner("-a -b 3").unwrap();
    assert_eq!(r, (true, 3));

    let expected = "Usage: app [-a] -b=B

Available options:
    -a          opt a
    -b=B        opt b
    -h, --help  Prints help information
";

    let r = parser.run_inner("asm --help").unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    assert_eq!(r, expected);
}
