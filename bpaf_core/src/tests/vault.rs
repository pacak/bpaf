use crate::*;

key!(MyKey: u32);

#[test]
fn vault_basic() {
    let parser = short('a')
        .argument::<u32>("A")
        .vault(|storage, v| {
            storage.set::<MyKey>(v);
            Ok::<_, &'static str>(v)
        })
        .to_options();

    let r = parser.run_inner("-a 42").unwrap();
    assert_eq!(r, 42);
}

#[test]
fn vault_error() {
    let parser = short('a')
        .argument::<u32>("A")
        .vault(|_, v| if v > 100 { Err("too large") } else { Ok(v) })
        .to_options();

    let r = parser.run_inner("-a 42").unwrap();
    assert_eq!(r, 42);

    let r = parser.run_inner("-a 200").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '200': too large\n");
}

#[test]
fn vault_storage_shared() {
    key!(A: u32);
    key!(B: u32);

    let a = short('a').argument::<u32>("A");
    let b = short('b').argument::<u32>("B");
    let parser = construct!(a, b)
        .vault(|storage, (a, b)| {
            storage.set::<A>(a);
            storage.set::<B>(b);
            let sum = a + b;
            Ok::<_, &'static str>(sum)
        })
        .to_options();

    let r = parser.run_inner("-a 1 -b 2").unwrap();
    assert_eq!(r, 3);
}

#[test]
fn vault_cross_parser_interaction() {
    key!(Limit: u32);

    let limit = short('l').argument::<u32>("LIMIT").vault(|storage, v| {
        storage.set::<Limit>(v);
        Ok::<_, &'static str>(v)
    });

    let val = short('v').argument::<u32>("VAL").vault(|storage, v| {
        let limit = storage.get::<Limit>().copied().unwrap_or(100);
        if v > limit {
            Err("exceeds limit")
        } else {
            Ok(v)
        }
    });

    let parser = construct!(limit, val).to_options();

    let r = parser.run_inner("-l 10 -v 5").unwrap();
    assert_eq!(r, (10, 5));

    let r = parser.run_inner("-l 10 -v 20").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '20': exceeds limit\n");
}

#[test]
fn vault_pure_sum() {
    key!(A: u32);
    key!(B: u32);

    let a = short('a').argument::<u32>("A").vault(|storage, v| {
        storage.set::<A>(v);
        Ok::<_, &'static str>(v)
    });
    let b = short('b').argument::<u32>("B").vault(|storage, v| {
        storage.set::<B>(v);
        Ok::<_, &'static str>(v)
    });
    let sum = pure(()).vault(|storage, ()| {
        let a = storage.get::<A>().copied().unwrap_or(0);
        let b = storage.get::<B>().copied().unwrap_or(0);
        Ok::<_, &'static str>(a + b)
    });

    let parser = construct!(a, b, sum).to_options();

    let r = parser.run_inner("-a 3 -b 7").unwrap();
    assert_eq!(r, (3, 7, 10));
}

#[test]
fn vault_parse_chain() {
    let parser = short('a')
        .argument::<u32>("A")
        .parse::<_, u32, &'static str>(|v| {
            if v % 2 == 0 {
                Ok(v)
            } else {
                Err("must be even")
            }
        })
        .vault(|storage, v| {
            storage.set::<MyKey>(v);
            Ok::<_, &'static str>(v / 2)
        })
        .to_options();

    let r = parser.run_inner("-a 4").unwrap();
    assert_eq!(r, 2);

    let r = parser.run_inner("-a 3").unwrap_err().unwrap_stderr();
    assert_eq!(r, "couldn't parse '3': must be even\n");
}
