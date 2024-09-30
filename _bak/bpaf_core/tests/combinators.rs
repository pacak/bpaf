use bpaf_core::*;

#[test]
fn alt_of_flag_arg_1() {
    let a = short('a').req_flag(0);
    let b = short('a').argument::<usize>("A");
    let p = construct!([a, b]).to_options();

    assert_eq!(1, p.run_inner(["-a", "1"]).unwrap());
    // assert_eq!(0, p.run_inner(["-a"]).unwrap());
}

#[test]
fn alt_of_flag_arg_2() {
    let a = short('a').req_flag(0);
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

#[test]
fn simple_sum() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner(["-b"]).unwrap();
    assert_eq!(r, 'b');
}

#[test]
fn sum_of_prod1() {
    let a = positional::<String>("B").map(|_| 'B');
    let aa = a.map(|p| (p, false));

    let b = positional::<String>("C").map(|_| 'C');
    let c = short('c').switch();
    let bc = construct!(b, c);

    // parser takes a positional with an optional switch, effectively this
    // [<B> | <C> [-c]]
    // passing `-c` on either side of positional item should force it to parse <C>,
    // otherwise <B> should win since it comes sooner

    let parser = construct!([aa, bc]).map(|p| p.0).to_options();

    // let r = parser.run_inner(["x"]).unwrap();
    // assert_eq!(r, 'B');

    let r = parser.run_inner(["-c", "x"]).unwrap();
    assert_eq!(r, 'C');
    //
    // let r = parser.run_inner(["x", "-c"]).unwrap();
    // assert_eq!(r, 'C');
}
