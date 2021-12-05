use bpaf::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
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
        .help("speed in MPH")
        .long("mph")
        .argument("SPEED")
        .build()
        .parse(|s| f64::from_str(&s));
    let k = short('k')
        .long("kph")
        .help("speed in KPH")
        .argument("SPEED")
        .build()
        .parse(|s| f64::from_str(&s).map(|s| s / 0.62));
    m.or_else(k)
}

pub fn main() {
    let info = Info::default().descr("this is a test").version("12");

    let fast = short('f')
        .long("fast")
        .switch()
        .help("Use faster acceleration");
    let acc_parser = Parser::pure(Cmd::Accelerate).ap(fast);
    let cmd = command(
        "accel",
        "command for acceleration",
        info.clone().descr("accelerating").for_parser(acc_parser),
    );

    let a = short('a')
        .long("AAAAA")
        .switch()
        .help("maps to a boolean, is optional");
    let b = long("bbbb").req_flag(()).help("maps to a () and mandatory");
    let c = speed();

    let parser = construct!(Foo: a, b, c, cmd);
    //    let mk = Parser::pure(curry!(|a, b, c, cmd| Foo { a, b, c, cmd }));
    //  let x = mk.ap(a).ap(b).ap(speed()).ap(acc);
    let y = info.for_parser(parser);

    let xx = run(y);
    println!("{:?}", xx);
}
