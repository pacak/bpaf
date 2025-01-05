use bpaf_core::*;

#[test]
fn simple() {
    let a = short('a').argument::<usize>("A").optional().to_options();

    let r = a.run_inner([]).unwrap();

    assert_eq!(r, None);
}

#[test]
fn repeated() {
    let a = positional::<usize>("A");
    let b = short('b').argument::<usize>("B").optional();
    let parser = construct!(a, b).many::<Vec<_>>().to_options();

    let r = parser.run_inner(["1", "2", "-b=3"]).unwrap();
    assert_eq!(r, &[(1, None), (2, Some(3))]);

    let r = parser
        .run_inner(["1", "2", "-b=3", "4", "-b", "5", "6"])
        .unwrap();
    assert_eq!(r, &[(1, None), (2, Some(3)), (4, Some(5)), (6, None)]);

    let r = parser.run_inner(["4", "-b", "5", "6"]).unwrap();
    assert_eq!(r, &[(4, Some(5)), (6, None)]);
}

#[test]
fn nested() {
    let a = positional::<usize>("A");
    let b = short('b').argument::<usize>("B").optional();
    let ab = construct!(a, b).optional().to_options();

    let r = ab.run_inner(["-b"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "unexpected item!");

    let r = ab.run_inner(["1", "-b"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "unexpected item!");

    let r = ab.run_inner([]).unwrap();
    assert_eq!(r, None);

    let r = ab.run_inner(["-b", "3", "1"]).unwrap();
    assert_eq!(r, Some((1, Some(3))));

    let r = ab.run_inner(["1", "-b", "3"]).unwrap();
    assert_eq!(r, Some((1, Some(3))));

    let r = ab.run_inner(["1"]).unwrap();
    assert_eq!(r, Some((1, None)));
}

#[test]
fn non_consuming() {
    let a = positional::<usize>("A");
    let b = short('b').switch();
    let ab = construct!(a, b).optional().to_options();

    let r = ab.run_inner(["-b"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "Expected <A>");
}

#[test]
fn many() {
    let a = positional::<usize>("A").many::<Vec<_>>().to_options();
    let r = a.run_inner(["1", "2", "3"]).unwrap();
    assert_eq!(r, &[1, 2, 3,]);
}
