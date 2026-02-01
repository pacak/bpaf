use crate::*;

#[test]
fn is_odd_arg() {
    let parser = short('a')
        .argument::<u32>("A")
        .parse::<_, u32, &'static str>(|v| {
            if v % 2 == 0 {
                Ok(v)
            } else {
                Err("Value must be even")
            }
        })
        .to_options();

    let r = parser.run_inner("-a 4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("-a 3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '3': Value must be even\n");

    let r = parser.run_inner("-a3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '3': Value must be even\n");

    let r = parser.run_inner("-a=3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '3': Value must be even\n");
}

#[test]
fn is_odd_pos() {
    let parser = positional::<u32>("A")
        .parse::<_, u32, &'static str>(|v| {
            if v % 2 == 0 {
                Ok(v)
            } else {
                Err("Value must be even")
            }
        })
        .to_options();

    let r = parser.run_inner("4").unwrap();
    assert_eq!(r, 4);

    let r = parser.run_inner("3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '3': Value must be even\n");
}

#[test]
fn leaf_only() {
    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b)
        .parse::<_, u32, &'static str>(|(a, b)| {
            let v = a + b;

            if v % 2 == 0 {
                Ok(v)
            } else {
                Err("Value must be even")
            }
        })
        .to_options();

    let r = parser.run_inner("-a 1 -b 2").unwrap_err().unwrap_stderr();

    assert_eq!(r, "parse error: Value must be even\n");
}

#[test]
fn fallback_with_ok() {
    let parser = short('a')
        .argument("ARG")
        .fallback_with::<_, &str>(|| Ok(10u32))
        .to_options();

    let r = parser.run_inner("-a 1").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("").unwrap();
    assert_eq!(r, 10);
}

#[test]
fn fallback_with_err() {
    let parser = short('a')
        .argument::<u32>("ARG")
        .fallback_with::<_, &str>(|| Err("nope"))
        .to_options();

    let r = parser.run_inner("-a 1").unwrap();
    assert_eq!(r, 1);

    let r = parser.run_inner("-a x").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse 'x': invalid digit found in string\n");

    let r = parser.run_inner("").unwrap_err().unwrap_stderr();
    assert_eq!(r, "nope\n");
}
