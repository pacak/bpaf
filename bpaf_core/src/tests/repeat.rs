use crate::*;

#[test]
fn last_works() {
    let p = positional::<u32>("P").last();
    let parser = p.to_options();

    let r = parser.run_inner("1 2 3 4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `P`\n");
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
    let expected = "missing `B`\n";
    assert_eq!(r, expected);
}

#[test]
fn error_msg_from_some_is_retained() {
    let a = short('a').argument::<u32>("A");
    let parser = a.some("needs some").to_options();

    let r = parser.run_inner("-a 1").unwrap();
    assert_eq!(r, &[1]);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    let expected = "needs some\n";
    assert_eq!(r, expected);
}

#[test]
fn can_catch_missing_in_some() {
    let a = short('a').argument::<u32>("A");
    let parser = a.some("needs some").fallback(vec![42]).to_options();

    let r = parser.run_inner("-a 1").unwrap();
    assert_eq!(r, &[1]);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, &[42]);
}

#[test]
fn no_dataloss_with_some() {
    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b)
        .some("need some")
        .fallback(vec![(1, 2)])
        .to_options();

    let r = parser.run_inner("-a 42").unwrap_err().unwrap_stderr();
    let expected = "missing `-b B`\n";
    assert_eq!(r, expected);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, &[(1, 2)]);

    let r = parser.run_inner("-b 3 -a4").unwrap();
    assert_eq!(r, &[(4, 3)]);
}

#[test]
fn pairpos_1() {
    let a = positional::<u32>("A").optional();
    let b = positional::<u32>("B");

    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("1 42").unwrap();
    assert_eq!(r, (Some(1), 42));
}

#[test]
fn pairpos_2() {
    let a = positional::<u32>("A");
    let b = positional::<u32>("B").optional();

    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("1 42").unwrap();
    assert_eq!(r, (1, Some(42)));
}
