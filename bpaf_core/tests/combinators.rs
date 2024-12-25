use bpaf_core::*;

#[test]
fn alt_of_req_flags() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let p = construct!([a, b]);
    assert_eq!("no", run_parser(&p, ["-a", "-b"]).unwrap_err());
}

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

#[test]
fn sum_of_flag_arg2() {
    let a = short('a').argument::<usize>("A");
    let b = short('a').req_flag('a').map(|_| 0);
    let p = construct!(a, b).to_options();
    // items are consumed left to right, so first -a is an argument
    let r = p.run_inner(["-a", "-a", "1"]);

    todo!("{:?}", r);

    //assert_eq!((1, 0), run_parser(&p, ["-a", "1", "-a"]).unwrap());
}
