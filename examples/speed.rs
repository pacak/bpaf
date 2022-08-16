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

fn speed() -> impl Parser<f64> {
    let m = short('m')
        .long("mph")
        .help("speed in MPH")
        .argument("SPEED")
        .from_str::<f64>();

    let k = short('k')
        .help("speed in KPH")
        .long("kph")
        .argument("SPEED")
        .parse(|s| f64::from_str(&s).map(|s| s / 0.62));

    construct!([m, k])
}

pub fn main() {
    let fast = short('f')
        .long("fast")
        .help("Use faster acceleration")
        .switch();

    let acc_parser = construct!(Cmd::Accelerate(fast));

    let cmd = command("accel", acc_parser.to_options().descr("this is a test"))
        .help("command for acceleration");

    let a = short('a')
        .long("AAAAA")
        .help("maps to a boolean, is optional")
        .switch();

    let b = long("bbbb")
        .req_flag(())
        .group_help("maps to a () and mandatory");

    let c = speed();

    let opts = construct!(Foo { a, b, c, cmd })
        .to_options()
        .version("12")
        .run();

    println!("{:?}", opts);
}
