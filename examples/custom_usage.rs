/// you can refer to generated usage inside custom usage
use bpaf::*;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Opts {
    speed: u32,
    distance: u32,
}

fn opts() -> Opts {
    let speed = short('k')
        .long("speed")
        .help("speed in km/h")
        .argument("SPEED")
        .from_str::<u32>();

    let distance = short('d')
        .long("distance")
        .help("distance in km")
        .argument("DISTANCE")
        .from_str::<u32>();

    (construct!(Opts { speed, distance }))
        .to_options()
        .descr("Accept speed and distance, print them.")
        .usage("custom {usage}")
        .run()
}

fn main() {
    let opts = opts();
    println!("Options: {:?}", opts);
}
