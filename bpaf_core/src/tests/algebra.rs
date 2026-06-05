use crate::*;

#[test]
fn or_fail() {
    let a = short('a').req_flag(42);
    let parser = a
        .or_exit(|e| fail(format!("this is error, failed with {e}")))
        .to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "this is error, failed with missing '-a'\n");
}

#[test]
fn or_success() {
    let a = short('a').req_flag(42);
    let parser = a
        .or_exit(|e| success(format!("Ok, failed with {e}")))
        .to_options();

    let r = parser.run_inner("-a").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("").unwrap_err().unwrap_stdout();
    assert_eq!(r, "Ok, failed with missing '-a'\n");
}

#[test]
fn then_fail() {
    let a = short('a').req_flag(42);
    let parser = a
        .then_exit::<()>(|c| fail(format!("this is fail of code {c}")))
        .to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stderr();
    assert_eq!(r, "this is fail of code 42\n");

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing '-a'\n");
}

#[test]
fn nested_literal_success() {
    let a = short('a')
        .req_flag(42)
        .then_exit(|v| success::<()>(format!("{v}!")));
    let parser = literal("alpha").nest(a).to_options();
    let r = parser.run_inner("alpha -a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "42!\n");
}

#[test]
fn nested_flag_success() {
    let a = short('a')
        .req_flag(42)
        .then_exit(|v| success::<()>(format!("{v}!")));
    let parser = long("alpha").nest(a).to_options();
    let r = parser.run_inner("--alpha -a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "42!\n");
}

#[test]
fn success_makes_associative_sum_left() {
    let a = short('a').req_flag('a').then_exit(|_| success("ok"));
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "ok\n");

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn success_makes_associative_sum_right() {
    let a = short('a').req_flag('a').then_exit(|_| success("ok"));
    let b = short('b').req_flag('b');
    let parser = construct!([b, a]).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "ok\n");

    let r = parser.run_inner("-b").unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn then_success() {
    let a = short('a').req_flag(42);
    let parser = a.then_exit::<()>(|_| success("this is ok")).to_options();

    let r = parser.run_inner("-a").unwrap_err().unwrap_stdout();
    assert_eq!(r, "this is ok\n");

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "missing '-a'\n");
}

#[test]
fn sum_is_associative_right() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let ab = construct!([a, b]);
    let parser = construct!([ab, c]).to_options();
    assert_eq!(parser.run_inner("-a").unwrap(), 'a');
    assert_eq!(parser.run_inner("-b").unwrap(), 'b');
    assert_eq!(parser.run_inner("-c").unwrap(), 'c');
}

#[test]
fn sum_is_associative_left() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let bc = construct!([b, c]);
    let parser = construct!([a, bc]).to_options();
    assert_eq!(parser.run_inner("-a").unwrap(), 'a');
    assert_eq!(parser.run_inner("-b").unwrap(), 'b');
    assert_eq!(parser.run_inner("-c").unwrap(), 'c');
}

#[test]
fn sum_with_more_than_one_success() {
    let a = positional::<u32>("A").map(|a| a * 10);
    let b = positional::<u32>("b").map(|a| a * 20);
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner("1").unwrap();
    assert_eq!(r, 10);
}
