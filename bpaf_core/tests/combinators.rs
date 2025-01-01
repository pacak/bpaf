use bpaf_core::*;

#[test]
fn alt_of_flag_arg_1() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");

    let p = construct!([a, b]).to_options();
    assert_eq!(1, p.run_inner(["-a", "1"]).unwrap());
    assert_eq!(0, p.run_inner(["-a"]).unwrap());
}

#[test]
fn alt_of_flag_arg_2() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");

    let p = construct!([b, a]).to_options();
    assert_eq!(1, p.run_inner(["-a", "1"]).unwrap());
    assert_eq!(0, p.run_inner(["-a"]).unwrap());
}

#[test]
fn sum_of_flag_arg1() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");
    let p = construct!(a, b).to_options();

    assert_eq!((0, 1), p.run_inner(["-a", "-a", "1"]).unwrap());
    // assert_eq!((1, 0), run_parser(&p, ["-a", "1", "-a"]).unwrap());
}
