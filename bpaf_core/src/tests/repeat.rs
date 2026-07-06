use crate::*;

#[test]
fn last_works() {
    let p = positional::<u32>("P").last();
    let parser = p.to_options();

    let r = parser.run_inner("1 2 3 4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected 'P'\n");
}

#[test]
fn last_single_item() {
    let p = positional::<u32>("P").last();
    let parser = p.to_options();

    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("10 20 30").unwrap();
    assert_eq!(r, 30);
}

#[test]
fn count_positionals() {
    let p = positional::<u32>("P").count();
    let parser = p.to_options();

    let r = parser.run_inner("10 20 30").unwrap();
    assert_eq!(r, 3usize);

    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 0);
}

#[test]
fn count_req_flags() {
    let v = short('v').req_flag(true).count();
    let parser = v.to_options();

    let r = parser.run_inner("-v -v -v").unwrap();
    assert_eq!(r, 3);

    let r = parser.run_inner("-v").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 0);
}

#[test]
fn count_in_construct() {
    let a = positional::<u32>("A");
    let b = positional::<u32>("B").count();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("10 20 30 40").unwrap();
    assert_eq!(r, (10, 3usize));

    let r = parser.run_inner("42").unwrap();
    assert_eq!(r, (42, 0));
}

#[test]
fn last_in_construct() {
    let a = positional::<u32>("A");
    let b = positional::<u32>("B").last();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("10 20 30 40").unwrap();
    assert_eq!(r, (10, 40));

    let r = parser.run_inner("10 20").unwrap();
    assert_eq!(r, (10, 20));
}

#[test]
fn last_of_req_flag() {
    let v = short('v').req_flag(true).last();
    let parser = v.to_options();

    let r = parser.run_inner("-v -v -v").unwrap();
    assert!(r);

    let r = parser.run_inner("-v").unwrap();
    assert!(r);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '-v'\n");
}

#[test]
fn last_with_parse_filter() {
    let p = positional::<u32>("P")
        .parse(|v| if v > 10 { Ok(v) } else { Err("too small") })
        .last();
    let parser = p.to_options();

    let r = parser.run_inner("20 30 40").unwrap();
    assert_eq!(r, 40);

    let r = parser.run_inner("5 20 30").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '5': too small\n");
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
    let expected = "expected 'B'\n";
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
fn can_handle_missing_in_many() {
    let parser = positional::<u32>("A")
        .many()
        .map(|items| if items.is_empty() { vec![42] } else { items })
        .to_options();

    let r = parser.run_inner("1").unwrap();
    assert_eq!(r, &[1]);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, &[42]);
}

#[test]
fn parse_some_catch() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum A {
        U32(u32),
        S(String),
    }
    let a1 = short('a').argument("N").map(A::U32).some("A").hide();
    let a2 = short('a').argument("S").map(A::S).some("A").hide();
    let parser = construct!([a1, a2]).to_options();

    let r = parser.run_inner("-a 10").unwrap();
    assert_eq!(r, vec![A::U32(10)]);

    let r = parser.run_inner("-a x").unwrap();
    assert_eq!(r, vec![A::S("x".to_string())]);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "A\n");
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
    let expected = "expected '-b=B'\n";
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

#[test]
fn catch_on_option() {
    let a = positional::<u32>("A")
        .parse(|v| if v < 10 { Ok(v) } else { Err("too big") })
        .optional()
        .catch();
    let b = positional::<u32>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner("20").unwrap();
    assert_eq!(r, (None, 20));

    let r = parser.run_inner("1 42").unwrap();
    assert_eq!(r, (Some(1), 42));
}

#[test]
fn many_many_switch() {
    let a = short('a').switch().many().many();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, vec![vec![true, true]]);

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, vec![vec![true, true]]);
}

#[test]
fn many_many_req() {
    let a = short('a').req_flag(true).many().many();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, vec![vec![true, true]]);

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, vec![vec![true, true]]);
}

#[test]
fn many_opt_switch() {
    let a = short('a').switch().many().optional();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, Some(vec![true, true]));

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, Some(vec![true, true]));
}

#[test]
fn many_opt_req() {
    let a = short('a').req_flag(true).many().optional();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, Some(vec![true, true]));

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, Some(vec![true, true]));
}

#[test]
fn opt_many_switch() {
    let a = short('a').switch().optional().many();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, vec![Some(true), Some(true)]);

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, vec![Some(true), Some(true)]);
}

#[test]
fn opt_many_req() {
    let a = short('a').req_flag(true).optional().many();
    let parser = a.to_options();

    let r = parser.run_inner("-a -a").unwrap();
    assert_eq!(r, vec![Some(true), Some(true)]);

    let r = parser.run_inner("-aa").unwrap();
    assert_eq!(r, vec![Some(true), Some(true)]);
}
#[test]
fn parse_many_errors_positional() {
    let p = positional::<u32>("N").many().to_options();

    let r = p.run_inner("1 2 3").unwrap();
    assert_eq!(r, vec![1, 2, 3]);

    let r = p.run_inner("1 2 x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'x': invalid digit found in string\n");
}

#[test]
fn parse_collect_flag() {
    let p = short('p')
        .argument::<u32>("N")
        .collect::<Vec<_>>()
        .to_options();

    let r = p.run_inner("-p 1 -p 2").unwrap();
    assert_eq!(r, vec![1, 2]);

    let r = p.run_inner("-p 1 -p x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'x': invalid digit found in string\n");
}

#[test]
fn parse_many_errors_flag() {
    let p = short('p').argument::<u32>("N").many().to_options();

    let r = p.run_inner("-p 1 -p 2").unwrap();
    assert_eq!(r, vec![1, 2]);

    let r = p.run_inner("-p 1 -p x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'x': invalid digit found in string\n");
}

#[test]
fn optional_bool_states() {
    let parser = short('a').switch().optional().to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, Some(true));

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, Some(false));
}

#[test]
fn many_with_adjacent_leaf_leaves_no_leftovers() {
    let b = positional::<usize>("x");
    let ab = short('a').nest(b).many();
    let bc = short('a').switch();
    let parser = construct!(ab, bc).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected '<x>'\n");

    let r = parser.run_inner("-a 10").unwrap();
    assert_eq!(r, (vec![10], false));

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, (Vec::new(), false));
}
