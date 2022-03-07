use bpaf::*;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Opts {
    speed: f64,
    distance: f64,
}

fn opts() -> Opts {
    let speed = short('k')
        .long("speed") // give it a name
        .help("speed in KPH") // and help message
        .argument("SPEED") // it's an argument with metavar
        .from_str() // that is parsed from string as f64
        .map(|s: f64| s / 0.62); // and converted to mph

    let distance = short('d')
        .long("distance")
        .help("distance in miles")
        .argument("DISTANCE")
        .from_str();

    let parser = construct!(Opts { speed, distance });

    let info = Info::default().descr("Accept speed and distance, print them.");
    info.for_parser(parser).run()
}

fn main() {
    let opts = opts();
    println!("Options: {opts:?}");
}
