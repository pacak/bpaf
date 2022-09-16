/// A very basic example with excessive documentation
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
        .argument::<f64>("SPEED") // it's an argument with metavar
        .map(|s| s / 0.62); // and converted to mph

    let distance = short('d')
        .long("distance")
        .help("distance in miles")
        .argument("DISTANCE");

    (construct!(Opts { speed, distance }))
        .to_options()
        .descr("Accept speed and distance, print them.")
        .run()
}

fn main() {
    let opts = opts();
    println!("Options: {:?}", opts);
}
