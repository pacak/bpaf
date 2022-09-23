//! You don't need to import bpaf in order to use it, it will look ugly though

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
    let speed = bpaf::Parser::fallback(
        bpaf::short('s')
            .long("speed")
            .help("Set speed")
            .argument::<f64>("SPEED"),
        42.0,
    );

    // packing things in a struct assumes parser for each field is in scope.
    let opt = bpaf::Parser::to_options(bpaf::construct!(Out { debug, speed })).run();

    println!("{:#?}", opt);
}
