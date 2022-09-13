use bpaf::*;

#[test]
fn get_any_simple() {
    let a = short('a').switch();
    let b = any("REST").help("any help").str();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap().1;
    assert_eq!(r, "-b");

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap().1;
    assert_eq!(r, "-b");

    //let r = parser.run_inner(Args::from(&["-b=foo", "-a"])).unwrap().1;
    //assert_eq!(r, "-b=foo");
    //    todo!("{:?}", Args::from(&["-b=foo", "-a"]));
}

#[test]
fn get_any_many() {
    let a = short('a').switch();
    let b = any("REST").help("any help").str().many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(Args::from(&["-a", "-b"])).unwrap().1;
    assert_eq!(r, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a"])).unwrap().1;
    assert_eq!(r, &["-b"]);

    let r = parser.run_inner(Args::from(&["-b", "-a", "-b"])).unwrap().1;
    assert_eq!(r, &["-b", "-b"]);
}
