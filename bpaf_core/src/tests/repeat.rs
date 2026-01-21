use crate::*;

#[test]
fn last_works() {
    let p = positional::<u32>("P").last();
    let parser = p.to_options();

    let r = parser.run_inner("1 2 3 4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `P`");
}

#[test]
fn last_fallback() {
    let p = positional::<u32>("P").last().fallback(42);
    let parser = p.to_options();

    let r = parser.run_inner("1 2 3 4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn no_losing_data() {
    let a = positional::<u32>("A");
    let b = positional::<u32>("B");
    let parser = construct!(a, b).many().to_options();

    let r = parser.run_inner("1 2 3 4").unwrap();
    assert_eq!(r, &[(1, 2), (3, 4)]);

    let r = parser.run_inner("1 2 3").unwrap_err().unwrap_stderr();
    let expected = "missing `B`";
    assert_eq!(r, expected);
}
