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
