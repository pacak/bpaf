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
