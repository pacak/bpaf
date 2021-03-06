//! You don't need to import bpaf in order to use it

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Out {
    debug: bool,
    speed: f64,
}

fn main() {
    // A flag, true if used in the command line. Can be required, this one is optional
    let debug = bpaf::short('d')
        .long("debug")
        .help("Activate debug mode")
        .switch();

    // an argument, parsed and with default value
    let speed = bpaf::short('s')
        .long("speed")
        .help("Set speed")
        .argument("SPEED")
        .from_str::<f64>()
        .fallback(42.0);

    // packing things in a struct assumes parser for each field is in scope.
    let parser = bpaf::construct!(Out { debug, speed });
    let opt = bpaf::Info::default().for_parser(parser).run();
    println!("{:#?}", opt);
}
