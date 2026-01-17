use crate::*;

#[test]
fn from_any_str_works() {
    let a = any_from_str::<i32>("I");
    let parser = a.to_options();

    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-42").unwrap();
    assert_eq!(r, -42);
}

#[test]
fn simple_parse_any_works() {
    let parser = any("A", |s: &str| s.parse::<i32>().ok()).to_options();

    let r = parser.run_inner("-42").unwrap();
    assert_eq!(r, -42);
    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn with_flag_parse_any_works() {
    let a = any("A", |s: &str| s.parse::<i32>().ok());
    let b = short('b').switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("-b -10").unwrap();
    assert_eq!(r, (-10, true));
    let r = parser.run_inner("-10 -b").unwrap();
    assert_eq!(r, (-10, true));
}

#[test]
fn multiple_any_parsers() {
    let a = any("A", |s: &str| s.parse::<i32>().ok().filter(|v| *v < -100));
    let b = any("B", |s: &str| {
        s.parse::<i32>().ok().filter(|v| (-100..100).contains(v))
    });
    let c = any("C", |s: &str| s.parse::<i32>().ok().filter(|v| *v > 100));
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner("-1000 1000 0").unwrap();
    assert_eq!(r, (-1000, 0, 1000));

    let r = parser.run_inner("1000 -1000 0").unwrap();
    assert_eq!(r, (-1000, 0, 1000));

    let r = parser.run_inner("0 -1000 1000").unwrap();
    assert_eq!(r, (-1000, 0, 1000));
}

#[test]
fn any_from_str_works() {
    let a = any_from_str::<i32>("A").help("simple lit");
    let parser = a.to_options();
    let r = parser.run_inner("-42").unwrap();
    assert_eq!(r, -42);
}
