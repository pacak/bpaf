use bpaf_core::*;

#[test]
fn simple_any() {
    let a = any("SRC", |s: String| {
        s.strip_prefix("if=").map(|r| r.to_owned())
    });

    let b = any("SRC", |s: String| {
        s.strip_prefix("of=").map(|r| r.to_owned())
    });
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(["of=hello", "if=world"]).unwrap();
    assert_eq!(r, ("world".to_owned(), "hello".to_owned()));

    let r = parser.run_inner(["if=hello", "of=world"]).unwrap();
    assert_eq!(r, ("hello".to_owned(), "world".to_owned()));
}
