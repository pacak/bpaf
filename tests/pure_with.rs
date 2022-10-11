use bpaf::*;

#[test]
fn default_value_using_pure_with_ok() {
    // ~ this is a bit artifactial (it's better to use fallback_with instead actually),
    // but this example demonstrates the concept
    let a = short('t').argument::<u32>("N");
    let b = pure_with::<_, _, String>(|| Ok(42u32));

    let opts = construct!([a, b]).to_options();

    let r = opts.run_inner(Args::from(&["-t", "87"]));
    assert_eq!(87, r.unwrap());

    let r = opts.run_inner(Args::from(&[]));
    assert_eq!(42, r.unwrap());
}

#[test]
fn default_value_using_pure_with_err() {
    let a = short('t').argument::<u32>("N");
    let b = pure_with::<_, _, String>(|| Err(String::from("some-err")));

    let opts = construct!([a, b]).to_options();
    let r = opts.run_inner(Args::from(&[]));
    let e = r.unwrap_err().unwrap_stderr();
    assert_eq!("some-err", e);
}

#[test]
fn default_value_using_pure_with_ok_for_some() {
    let user_seeds = positional::<u32>("SEED").some("at least one required");
    let last_seeds = pure_with::<_, _, String>(|| {
        // ~ trying to lookup the last used seeds
        Ok(vec![3, 5, 7, 11])
    });
    let seeds = construct!([user_seeds, last_seeds]).to_options();

    let r = seeds.run_inner(Args::from(&["23", "59"]));
    assert_eq!(vec![23, 59], r.unwrap());

    let r = seeds.run_inner(Args::from(&[]));
    assert_eq!(vec![3, 5, 7, 11], r.unwrap());
}

#[test]
fn default_value_using_pure_with_err_for_some() {
    let user_seeds = positional::<u32>("SEED").some("at least one required");
    let last_seeds = pure_with(|| {
        // ~ trying to lookup the last used seeds but failing - parser fails, fallback works
        Err("oh, no!")
    });
    let default_seeds = pure(vec![1, 2]);
    let seeds = construct!([user_seeds, last_seeds, default_seeds]).to_options();

    let r = seeds.run_inner(Args::from(&["23", "59"]));
    assert_eq!(vec![23, 59], r.unwrap());

    let r = seeds.run_inner(Args::from(&[]));
    assert_eq!(r.unwrap(), vec![1, 2]);
}
