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
    assert_eq!(r, "couldn't parse `3`: Value must be even\n");

    let r = parser.run_inner("-a3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `3`: Value must be even\n");

    let r = parser.run_inner("-a=3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse `3`: Value must be even\n");
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
    assert_eq!(r, "couldn't parse `3`: Value must be even\n");
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

    assert_eq!(r, "couldn't parse: Value must be even\n");
}
