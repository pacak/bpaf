use bpaf::*;

#[test]
fn test_adjacent() {
    let a = short('a').req_flag(());
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c).adjacent().many().to_options();

    let r = parser
        .run_inner(&["-a", "-c", "-a", "-b", "-a", "-b", "-c"])
        .unwrap();
    // adjacent groups here argument
    // -a [-b] -c  | -a -b [-c] | -a -b -c
    assert_eq!(r, &[((), false, true), ((), true, false), ((), true, true)]);

    let r = parser.run_inner(&["-a"]).unwrap();
    assert_eq!(r, &[((), false, false)]);

    let r = parser.run_inner(&["-a", "-c"]).unwrap();
    assert_eq!(r, &[((), false, true)]);

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, &[]);
}

#[test]
fn test_adjacent_prefix() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("X");
    let ab = construct!(a, b).adjacent().optional();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(&["-c"]).unwrap();
    assert_eq!(r, (None, true));

    let r = parser.run_inner(&["-a", "1"]).unwrap();
    assert_eq!(r, (Some(((), 1)), false));

    let r = parser.run_inner(&["-c", "-a", "1"]).unwrap();
    assert_eq!(r, (Some(((), 1)), true));

    let r = parser.run_inner(&["-a", "1", "-c"]).unwrap();
    assert_eq!(r, (Some(((), 1)), true));
}

#[test]
fn adjacent_error_message_pos_single() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent();
    let parser = construct!(adj, d).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `C`, pass `--help` for usage information");
}

#[test]
fn adjacent_error_message_arg_single() {
    let a = short('a').req_flag(());
    let b = short('b').argument::<usize>("B");
    let c = short('c').argument::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent();
    let parser = construct!(adj, d).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "expected `-b=B`, got `10`. Pass `--help` for usage information"
    );
}

#[test]
fn adjacent_error_message_pos_many() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let c = positional::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent().many();
    let parser = construct!(adj, d).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap_err().unwrap_stderr();
    assert_eq!(r, "expected `C`, pass `--help` for usage information");
}

#[test]
fn adjacent_error_message_arg_many() {
    let a = short('a').req_flag(());
    let b = short('b').argument::<usize>("B");
    let c = short('c').argument::<usize>("C");
    let d = short('d').switch();
    let adj = construct!(a, b, c).adjacent().many();
    let parser = construct!(adj, d).to_options();

    let r = parser.run_inner(&["-a", "10"]).unwrap_err().unwrap_stderr();
    // this should ask for -b or -c and complain on 10...
    assert_eq!(
        r,
        "expected `-b=B`, got `10`. Pass `--help` for usage information"
    );
}

#[test]
fn adjacent_is_adjacent() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let parser = construct!(a, b).adjacent().map(|t| t.1).many().to_options();

    let r = parser
        .run_inner(&["-a", "-a", "10", "20"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "expected `B`, pass `--help` for usage information");

    let r = parser.run_inner(&["-a", "10", "-a", "20"]).unwrap();
    assert_eq!(r, [10, 20]);
}

#[test]
fn adjacent_with_switch() {
    let a = short('a').req_flag(());
    let b = positional::<usize>("B");
    let ab = construct!(a, b).adjacent().map(|t| t.1).many();
    let c = short('c').switch();
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(&["-a", "10", "-c"]).unwrap();
    assert_eq!(r, (vec![10], true));

    let r = parser.run_inner(&["-a", "10", "-c", "-a", "20"]).unwrap();
    assert_eq!(r, (vec![10, 20], true));

    let r = parser.run_inner(&["-c", "-a", "10", "-a", "20"]).unwrap();
    assert_eq!(r, (vec![10, 20], true));
}

#[test]
fn adjacent_limits_commands() {
    let x = pure(()).to_options().command("a").adjacent();
    let s = short('s').switch();
    let parser = construct!(s, x).to_options();

    let r = parser.run_inner(&["a", "-s"]).unwrap();
    assert_eq!(r, (true, ()));
}

#[test]
fn commands_and_adjacent() {
    let eat = positional::<String>("FOOD")
        .to_options()
        .command("eat")
        .help("eat something")
        .adjacent();

    let sleep = long("time")
        .argument::<String>("HOURS")
        .to_options()
        .command("sleep")
        .help("sleep for a bit")
        .adjacent();

    let cmds = construct!([eat, sleep]);
    let switch = short('s').switch();

    let parser = construct!(switch, cmds).to_options();

    let r = parser.run_inner(&["sleep", "--time", "12", "-s"]).unwrap();
    assert_eq!(r, (true, "12".to_owned()));

    let r = parser.run_inner(&["--help"]).unwrap_err().unwrap_stdout();

    // TODO - this is ugly
    let expected = "\
Usage: [-s] COMMAND ...

Available options:
    -s
    -h, --help  Prints help information

Available commands:
    eat         eat something
    sleep       sleep for a bit
";

    assert_eq!(r, expected);
}

#[test]
fn two_adjacent_args() {
    let x = short('x').argument::<usize>("X");
    let y = short('y').argument::<usize>("Y");
    let c = short('c').switch();
    let point = construct!(x, y).adjacent();
    let parser = construct!(point, c).to_options();

    let r = parser.run_inner(&["-x", "3", "-y", "4", "-c"]).unwrap();
    assert_eq!(r, ((3, 4), true));

    // they are adjacent to each other, but the way it is coded currently - they must be adjacent
    // to the first element.
    // Proper fix is to split "adjacent" into "adjacent to" and "adjacent block"

    // let r = parser.run_inner(&["-y", "3", "-x", "4", "-c"]).unwrap();
    // assert_eq!(r, ((4, 3), true));

    let r = parser
        .run_inner(&["-y", "3", "-c", "-x", "4"])
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "expected `-y=Y`, pass `--help` for usage information");
}

#[test]
fn start_adjacent_args1() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let d = any::<String, String, _>("D", Some).many();

    let abc = construct!([a, b, c]).last().start_adjacent();
    let parser = construct!(abc, d).to_options();

    let r = parser.run_inner(&["-a", "-b", "potat", "-c"]).unwrap();

    todo!("{:?}", r);
}

#[test]
fn start_adjacent_args2() {
    let a = short('a').req_flag('a');
    let b = short('b').req_flag('b');
    let c = short('c').req_flag('c');
    let d = any::<String, String, _>("D", Some).many();

    let abc = construct!([a, b, c]).start_adjacent().last();
    let parser = construct!(abc, d).to_options();

    let r = parser.run_inner(&["-a", "-b", "potat", "-c"]).unwrap();

    todo!("{:?}", r);
}

#[test]
fn start_adjacent_args3() {
    let a = short('a').flag('a', '?').start_adjacent();
    let d = any::<String, String, _>("D", Some).many();

    let parser = construct!(a, d).to_options();

    let r = parser.run_inner(&["x", "-a"]).unwrap();
    todo!("{:?}", r);
}
