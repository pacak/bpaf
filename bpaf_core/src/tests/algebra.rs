use crate::*;

#[test]
fn or_fail() {
    let a = short('a').req_flag(42);
    let parser = a.or_exit(fail("this is error")).to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "this is error");
}

#[test]
fn or_success() {
    let a = short('a').req_flag(42);
    let parser = a.or_exit(success("this is ok")).to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("").unwrap_err().unwrap_stdout();
    assert_eq!(r, "this is ok");
}

#[test]
fn then_fail() {
    let a = short('a').req_flag(42);
    let parser = a
        .then_exit::<()>(|c| fail(format!("this is fail of code {c}")))
        .to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "this is fail of code 42");

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `-a`");
}

#[test]
fn then_success() {
    let a = short('a').req_flag(42);
    let parser = a.then_exit::<()>(|_| success("this is ok")).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "this is ok");

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing `-a`");
}
