use bpaf_core::*;

#[test]
fn simple() {
    let a = short('a').argument::<usize>("A").optional();

    let r = run_parser::<_, &&str>(&a, &[]).unwrap();

    assert_eq!(r, None);
}

#[test]
fn nested() {
    let a = positional::<usize>("A");
    let b = short('b').argument::<usize>("B").optional();
    let ab = construct!(a, b).optional();

    let r = run_parser(&ab, ["-b"]).unwrap_err();
    assert_eq!(r, "unexpected item!");

    let r = run_parser(&ab, ["1", "-b"]).unwrap_err();
    assert_eq!(r, "unexpected item!");

    let r = run_parser::<_, &&str>(&ab, []).unwrap();
    assert_eq!(r, None);

    let r = run_parser(&ab, ["-b", "3", "1"]).unwrap();
    assert_eq!(r, Some((1, Some(3))));

    let r = run_parser(&ab, ["1", "-b", "3"]).unwrap();
    assert_eq!(r, Some((1, Some(3))));

    let r = run_parser(&ab, ["1"]).unwrap();
    assert_eq!(r, Some((1, None)));
}

#[test]
fn non_consuming() {
    let a = positional::<usize>("A");
    let b = short('b').switch();
    let ab = construct!(a, b).optional();

    let r = run_parser(&ab, ["-b"]).unwrap_err();
    assert_eq!(r, "Expected <A>");
}

#[test]
fn many() {
    let a = positional::<usize>("A").many::<Vec<_>>();
    let r = run_parser(&a, ["1", "2", "3"]).unwrap();
    assert_eq!(r, &[1, 2, 3,]);
}
