use crate::*;

#[test]
fn basic_two_item_arg() {
    let key = positional::<String>("KEY").help("Name of an option to set");
    let value = positional::<String>("VAL").help("Value to set");
    let n = long("set")
        .short('s')
        .nest(construct!(key, value))
        .help("help for nest");
    let l = short('l').long("long").help("with some help").switch();
    let parser = construct!(l, n).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: [-l] -s { KEY VAL }

Available options:
    -l, --long         with some help
    -s, --set KEY VAL  help for nest
    KEY                Name of an option to set
    VAL                Value to set
    -h, --help         Prints help information
";
    assert_eq!(r, expected);

    let (rl, (rk, rv)) = parser.run_inner("--set key value").unwrap();
    assert_eq!(rk, "key");
    assert_eq!(rv, "value");
    assert!(!rl);
}
