use bpaf_core::*;

#[test]
fn alt_of_flag_arg_1() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");

    let p = construct!([a, b]);
    assert_eq!(1, run_parser(&p, ["-a", "1"]).unwrap());
    assert_eq!(0, run_parser(&p, ["-a"]).unwrap());
}

#[test]
fn alt_of_flag_arg_2() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");

    let p = construct!([b, a]);
    assert_eq!(1, run_parser(&p, ["-a", "1"]).unwrap());
    assert_eq!(0, run_parser(&p, ["-a"]).unwrap());
}

#[test]
fn sum_of_flag_arg1() {
    let a = short('a').req_flag('a').map(|_| 0);
    let b = short('a').argument::<usize>("A");
    let p = construct!(a, b);

    assert_eq!((0, 1), run_parser(&p, ["-a", "-a", "1"]).unwrap());
    // assert_eq!((1, 0), run_parser(&p, ["-a", "1", "-a"]).unwrap());
}
