use bpaf::*;

#[test]
fn ambiguity() {
    #[derive(Debug, Clone)]
    enum A {
        V(Vec<bool>),
        W(String),
    }

    let a0 = short('a').switch().many().map(A::V);
    let a1 = short('a').argument("AAAAAA").map(A::W);
    let a = construct!([a0, a1]).to_options();

    let r = a
        .run_inner(Args::from(&["-aaaaaa"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "");
    todo!();
}
