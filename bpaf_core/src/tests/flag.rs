use crate::*;

#[test]
fn default_flag_wins_in_sum() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b').default();
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 'b');

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 'a');

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn multiple_aliases() {
    let a = short('a').short('b').short('c').req_flag(());
    let parser = a.to_options();

    let help = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected_help = "\
Usage: app -a

Available options:
    -a
    -h, --help  Prints help information
";
    assert_eq!(expected_help, help);
    parser.run_inner("-a").unwrap();
    parser.run_inner("-b").unwrap();
    parser.run_inner("-c").unwrap();
}
