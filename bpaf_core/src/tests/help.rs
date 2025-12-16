use crate::*;

#[test]
fn simple_flag() {
    let a = short('a').help("A simple argument").req_flag(());
    let parser = a.to_options();

    let r = parser.run_inner("-a --help").unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}
