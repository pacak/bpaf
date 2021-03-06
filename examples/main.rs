use bpaf::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Foo {
    a: bool,
    b: (),
    c: f64,
    cmd: Cmd,
}

#[derive(Debug, Clone)]
enum Cmd {
    Accelerate(bool),
}

fn speed() -> Parser<f64> {
    let m = short('m')
        .long("mph")
        .help("speed in MPH")
        .argument("SPEED")
        .parse(|s| f64::from_str(&s));

    let k = short('k')
        .help("speed in KPH")
        .long("kph")
        .argument("SPEED")
        .parse(|s| f64::from_str(&s).map(|s| s / 0.62));

    m.or_else(k)
}

pub fn main() {
    let info = Info::default().descr("this is a test").version("12");

    let fast = short('f')
        .long("fast")
        .help("Use faster acceleration")
        .switch();

    let acc_parser = construct!(Cmd::Accelerate(fast));

    let cmd = command(
        "accel",
        Some("command for acceleration"),
        info.clone().descr("accelerating").for_parser(acc_parser),
    );

    let a = short('a')
        .long("AAAAA")
        .help("maps to a boolean, is optional")
        .switch();

    let b = long("bbbb")
        .req_flag(())
        .group_help("maps to a () and mandatory");

    let c = speed();

    let parser = construct!(Foo { a, b, c, cmd });
    let opts = info.for_parser(parser);

    println!("{:?}", opts.run());
}
